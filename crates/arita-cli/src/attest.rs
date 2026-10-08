//! ATTEST-V0 — source sha256 + contract sidecar (ADR-018 / CUT ATTEST-V0-20260913).
//! Hash = SHA-256 of raw source file bytes (hex lowercase). Signatures / remote / CA IR OUT.

use crate::contract::{parse_contract_json, run_contract_spec, ContractExpect, ContractSpec};
use crate::measure::{OracleResult, Verdict};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub const ATTEST_VERSION: &str = "ATTEST-V0";
pub const TOOL_VERSION: &str = "0.1.0-f1";

/// Hex-encode bytes lowercase (no extra deps).
pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

/// SHA-256 hex of raw bytes.
pub fn sha256_hex(data: &[u8]) -> String {
    let dig = Sha256::digest(data);
    hex_encode(&dig)
}

/// Read file as raw bytes and hash. Err on IO.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("cannot read source for hash: {e}"))?;
    Ok(sha256_hex(&bytes))
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn expect_kind_payload(expect: &ContractExpect) -> (&'static str, String) {
    match expect {
        ContractExpect::Stdout(lines) => {
            let parts: Vec<String> = lines
                .iter()
                .map(|l| format!("\"{}\"", json_escape(l)))
                .collect();
            ("stdout", format!("[{}]", parts.join(",")))
        }
        ContractExpect::Reject(code) => ("reject", format!("\"{}\"", json_escape(code))),
    }
}

/// Sidecar path: `target/arita-attest/<contract_id>.attest.json` under workspace root.
pub fn sidecar_path(root: &Path, contract_id: &str) -> PathBuf {
    root.join("target/arita-attest")
        .join(format!("{contract_id}.attest.json"))
}

/// Also try next to the contract JSON: `<stem>.attest.json`.
pub fn sidecar_beside_contract(contract_json: &Path) -> PathBuf {
    let stem = contract_json
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("contract");
    contract_json
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{stem}.attest.json"))
}

/// Write attestation sidecar after a successful contract run.
pub fn write_attestation_sidecar(
    root: &Path,
    contract_json: Option<&Path>,
    spec: &ContractSpec,
    source_sha256: &str,
) -> Result<PathBuf, String> {
    let (kind, payload) = expect_kind_payload(&spec.expect);
    let body = format!(
        "{{\n  \"version\": \"{}\",\n  \"contract_id\": \"{}\",\n  \"source_path\": \"{}\",\n  \"source_sha256\": \"{}\",\n  \"expect_kind\": \"{}\",\n  \"expect_payload\": {},\n  \"tool_version\": \"{}\"\n}}\n",
        ATTEST_VERSION,
        json_escape(&spec.id),
        json_escape(&spec.source),
        json_escape(source_sha256),
        kind,
        payload,
        TOOL_VERSION,
    );

    let dir = root.join("target/arita-attest");
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create attest dir: {e}"))?;
    let primary = sidecar_path(root, &spec.id);
    fs::write(&primary, &body).map_err(|e| format!("cannot write attest sidecar: {e}"))?;
    let _ = contract_json; // beside-contract optional; prefer target/arita-attest only (keep ejemplos clean)
    Ok(primary)
}

/// Load `source_sha256` from a sidecar JSON (minimal field extract).
pub fn load_sidecar_hash(path: &Path) -> Result<String, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("cannot read sidecar: {e}"))?;
    extract_string_field(&text, "source_sha256")
        .ok_or_else(|| format!("sidecar missing source_sha256: {}", path.display()))
}

fn extract_string_field(json_text: &str, key: &str) -> Option<String> {
    let v = crate::contract::parse_json_value(json_text.trim()).ok()?;
    match v {
        crate::contract::Json::Object(m) => match m.get(key) {
            Some(crate::contract::Json::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// Resolve expected hash: embedded field, else sidecar under target/arita-attest or beside JSON.
pub fn resolve_expected_hash(
    root: &Path,
    contract_json: &Path,
    spec: &ContractSpec,
) -> Result<String, String> {
    if let Some(ref h) = spec.source_sha256 {
        if h.is_empty() {
            return Err("source_sha256 is empty".into());
        }
        return Ok(h.clone());
    }
    let primary = sidecar_path(root, &spec.id);
    if primary.is_file() {
        return load_sidecar_hash(&primary);
    }
    let beside = sidecar_beside_contract(contract_json);
    if beside.is_file() {
        return load_sidecar_hash(&beside);
    }
    Err(format!(
        "missing expected hash: no source_sha256 in contract and no sidecar at {} or {}",
        primary.display(),
        beside.display()
    ))
}

/// Verify: expected hash must match file bytes AND contract expect must pass.
pub fn run_attest_verify(root: &Path, json_path: &Path) -> OracleResult {
    let label = json_path
        .strip_prefix(root)
        .unwrap_or(json_path)
        .to_string_lossy()
        .into_owned();

    if !json_path.is_file() {
        return OracleResult {
            id: "attest-verify".into(),
            path: label,
            verdict: Verdict::Inconclusive,
            detail: format!("missing contract JSON: {}", json_path.display()),
        };
    }

    let text = match fs::read_to_string(json_path) {
        Ok(t) => t,
        Err(e) => {
            return OracleResult {
                id: "attest-verify".into(),
                path: label,
                verdict: Verdict::Inconclusive,
                detail: format!("cannot read contract JSON: {e}"),
            };
        }
    };

    let mut spec = match parse_contract_json(&text) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: "attest-verify".into(),
                path: label,
                verdict: Verdict::Inconclusive,
                detail: e,
            };
        }
    };

    let expected = match resolve_expected_hash(root, json_path, &spec) {
        Ok(h) => h,
        Err(e) => {
            return OracleResult {
                id: spec.id.clone(),
                path: label,
                verdict: Verdict::Inconclusive,
                detail: format!("attest verify: {e}"),
            };
        }
    };
    // Force hash check inside run_contract_spec.
    spec.source_sha256 = Some(expected);

    let mut r = run_contract_spec(root, &label, &spec);
    if r.verdict == Verdict::Accepted {
        r.detail = format!("attest verify ok: {}", r.detail);
    }
    r
}

pub fn cmd_attest_verify(json_arg: &str) -> ExitCode {
    let Some(root) = crate::measure::find_workspace_root() else {
        eprintln!("arita attest verify: cannot locate workspace root (Cargo.toml [workspace])");
        return ExitCode::from(2);
    };
    if let Err(e) = std::env::set_current_dir(&root) {
        eprintln!("arita attest verify: cannot cd to {}: {e}", root.display());
        return ExitCode::from(2);
    }

    let path = PathBuf::from(json_arg);
    let full = if path.is_absolute() {
        path
    } else {
        root.join(&path)
    };

    let result = run_attest_verify(&root, &full);
    match result.verdict {
        Verdict::Accepted => {
            println!("ok: {} — {}", result.id, result.detail);
            ExitCode::SUCCESS
        }
        Verdict::Rejected => {
            eprintln!(
                "arita attest verify rejected: {} — {}",
                result.id, result.detail
            );
            ExitCode::FAILURE
        }
        Verdict::Inconclusive => {
            eprintln!(
                "arita attest verify inconclusive: {} — {}",
                result.id, result.detail
            );
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod attest_tests {
    use super::*;

    #[test]
    fn sha256_hello_world_known() {
        // RFC / common test vector
        let h = sha256_hex(b"hello world");
        assert_eq!(
            h,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn hex_encode_empty() {
        assert_eq!(hex_encode(&[]), "");
        assert_eq!(hex_encode(&[0x0f, 0xf0]), "0ff0");
    }
}
