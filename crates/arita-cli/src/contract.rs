//! `arita contract` — JSON (ADR-017/018) + language-level `.arita` contracts (ADR-019/020).
//! ATTEST-V0: optional `source_sha256` on JSON; sidecar on success; hash mismatch → reject.
//! LANG-CONTRACT: parse `contract { }` from AST; same expect pipeline; not emitted to Rust.
//! CONTRACT-TARGET-FN: optional `target` entry rewrite (ADR-020); expect_reject ignores target.
//! Real E2E via build/parse_lower_check; skip ≠ PASS; invalid schema / missing → inconclusive.

use crate::measure::{normalize_stdout, OracleResult, Verdict};
use crate::parse_lower_check;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Declared measure contract oracles (JSON paths under ejemplos/contracts/).
pub struct ContractOracle {
    pub id: &'static str,
    pub path: &'static str,
    /// When true, measure accepted iff the contract check **rejects** (neg-style, e.g. bad hash).
    pub expect_fail: bool,
}

pub const CONTRACT_ORACLES: &[ContractOracle] = &[
    ContractOracle {
        id: "contract-hello",
        path: "ejemplos/contracts/contract-hello.json",
        expect_fail: false,
    },
    ContractOracle {
        id: "contract-f23-break",
        path: "ejemplos/contracts/contract-f23-break.json",
        expect_fail: false,
    },
    ContractOracle {
        id: "contract-neg-e0224",
        path: "ejemplos/contracts/contract-neg-e0224.json",
        expect_fail: false,
    },
    ContractOracle {
        id: "contract-hello-attested",
        path: "ejemplos/contracts/contract-hello-attested.json",
        expect_fail: false,
    },
    ContractOracle {
        id: "contract-hello-bad-hash",
        path: "ejemplos/contracts/contract-hello-bad-hash.json",
        expect_fail: true,
    },
    ContractOracle {
        id: "core01-ref-json-cli",
        path: "ejemplos/core01/ref/arita-ref-json-cli.arita",
        expect_fail: false,
    },
    ContractOracle {
        id: "core01-scenario-hello",
        path: "ejemplos/core01/11-scenario-hello.arita",
        expect_fail: false,
    },
    ContractOracle {
        id: "lang-contract-hello",
        path: "ejemplos/contracts/lang-hello.arita",
        expect_fail: false,
    },
    ContractOracle {
        id: "lang-contract-neg-e0224",
        path: "ejemplos/contracts/lang-neg-e0224.arita",
        expect_fail: false,
    },
    ContractOracle {
        id: "lang-contract-target-helper",
        path: "ejemplos/contracts/lang-target-helper.arita",
        expect_fail: false,
    },
    ContractOracle {
        id: "contract-target-helper",
        path: "ejemplos/contracts/contract-target-helper.json",
        expect_fail: false,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractExpect {
    Stdout(Vec<String>),
    Reject(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractSpec {
    pub id: String,
    pub source: String,
    pub expect: ContractExpect,
    /// Optional ATTEST-V0 pin: SHA-256 hex of raw source bytes.
    pub source_sha256: Option<String>,
    /// ADR-020: entry fn for expect_stdout (`None` ⇒ `"main"`). Ignored for expect_reject.
    pub target: Option<String>,
}

/// Parse contract JSON. Invalid schema → Err (caller maps to inconclusive).
pub fn parse_contract_json(text: &str) -> Result<ContractSpec, String> {
    let v = parse_json_value(text.trim())?;
    let obj = match v {
        Json::Object(m) => m,
        _ => return Err("invalid schema: root must be a JSON object".into()),
    };

    let id = match obj.get("id") {
        Some(Json::String(s)) if !s.is_empty() => s.clone(),
        Some(_) => return Err("invalid schema: id must be a non-empty string".into()),
        None => return Err("invalid schema: missing id".into()),
    };
    let source = match obj.get("source") {
        Some(Json::String(s)) if !s.is_empty() => s.clone(),
        Some(_) => return Err("invalid schema: source must be a non-empty string".into()),
        None => return Err("invalid schema: missing source".into()),
    };

    let source_sha256 = match obj.get("source_sha256") {
        None => None,
        Some(Json::String(s)) if !s.is_empty() => Some(s.clone()),
        Some(Json::String(_)) => {
            return Err("invalid schema: source_sha256 must be a non-empty string".into())
        }
        Some(_) => return Err("invalid schema: source_sha256 must be a string".into()),
    };

    let target = match obj.get("target") {
        None => None,
        Some(Json::String(s)) if !s.is_empty() => Some(s.clone()),
        Some(Json::String(_)) => {
            return Err("invalid schema: target must be a non-empty string".into())
        }
        Some(_) => return Err("invalid schema: target must be a string".into()),
    };

    let has_stdout = obj.contains_key("expect_stdout");
    let has_reject = obj.contains_key("expect_reject");
    match (has_stdout, has_reject) {
        (true, true) => {
            Err("invalid schema: exactly one of expect_stdout or expect_reject required".into())
        }
        (false, false) => {
            Err("invalid schema: exactly one of expect_stdout or expect_reject required".into())
        }
        (true, false) => {
            let lines = match obj.get("expect_stdout") {
                Some(Json::Array(arr)) => {
                    let mut out = Vec::with_capacity(arr.len());
                    for (i, el) in arr.iter().enumerate() {
                        match el {
                            Json::String(s) => out.push(s.clone()),
                            _ => {
                                return Err(format!(
                                    "invalid schema: expect_stdout[{i}] must be a string"
                                ))
                            }
                        }
                    }
                    out
                }
                _ => return Err("invalid schema: expect_stdout must be an array of strings".into()),
            };
            Ok(ContractSpec {
                id,
                source,
                expect: ContractExpect::Stdout(lines),
                source_sha256,
                target,
            })
        }
        (false, true) => {
            let code = match obj.get("expect_reject") {
                Some(Json::String(s)) if !s.is_empty() => s.clone(),
                Some(_) => {
                    return Err("invalid schema: expect_reject must be a non-empty string".into())
                }
                None => unreachable!(),
            };
            Ok(ContractSpec {
                id,
                source,
                expect: ContractExpect::Reject(code),
                source_sha256,
                target,
            })
        }
    }
}

/// Run a contract against workspace root. Returns OracleResult (accepted/rejected/inconclusive).
/// If `source_sha256` is set, recompute SHA-256 of raw source bytes and compare first.
pub fn run_contract_spec(root: &Path, json_label: &str, spec: &ContractSpec) -> OracleResult {
    let source_full = root.join(&spec.source);
    if !source_full.is_file() {
        return OracleResult {
            id: spec.id.clone(),
            path: json_label.into(),
            verdict: Verdict::Inconclusive,
            detail: format!("missing source file: {}", source_full.display()),
        };
    }

    let computed = match crate::attest::sha256_file(&source_full) {
        Ok(h) => h,
        Err(e) => {
            return OracleResult {
                id: spec.id.clone(),
                path: json_label.into(),
                verdict: Verdict::Inconclusive,
                detail: e,
            };
        }
    };

    if let Some(ref expected) = spec.source_sha256 {
        if expected.is_empty() {
            return OracleResult {
                id: spec.id.clone(),
                path: json_label.into(),
                verdict: Verdict::Inconclusive,
                detail: "source_sha256 is empty (missing hash when required — never PASS)".into(),
            };
        }
        if computed != *expected {
            return OracleResult {
                id: spec.id.clone(),
                path: json_label.into(),
                verdict: Verdict::Rejected,
                detail: format!("source_sha256 mismatch: got={computed} expected={expected}"),
            };
        }
    }

    let mut result = match &spec.expect {
        ContractExpect::Stdout(expected) => {
            run_expect_stdout(json_label, spec, &source_full, expected)
        }
        ContractExpect::Reject(code) => run_expect_reject(json_label, spec, &source_full, code),
    };
    // Always surface computed hash in detail for operators / Mac sync.
    if result.verdict == Verdict::Accepted {
        result.detail = format!("{}; source_sha256={computed}", result.detail);
    } else if spec.source_sha256.is_none() {
        result.detail = format!("{}; computed_source_sha256={computed}", result.detail);
    }
    result
}

/// Compute source sha256 for a spec (helper for CLI print / sidecar).
pub fn compute_source_sha256(root: &Path, spec: &ContractSpec) -> Result<String, String> {
    crate::attest::sha256_file(&root.join(&spec.source))
}

fn run_expect_stdout(
    json_label: &str,
    spec: &ContractSpec,
    source_full: &Path,
    expected: &[String],
) -> OracleResult {
    let path_str = source_full.to_str().unwrap_or(spec.source.as_str());
    let entry = spec.target.as_deref().unwrap_or("main");
    let bin = match crate::build_with_entry(path_str, entry) {
        Ok(p) => p,
        Err(e) => {
            return OracleResult {
                id: spec.id.clone(),
                path: json_label.into(),
                verdict: Verdict::Rejected,
                detail: format!("build failed: {e}"),
            };
        }
    };

    let run = match Command::new(&bin).output() {
        Ok(o) => o,
        Err(e) => {
            return OracleResult {
                id: spec.id.clone(),
                path: json_label.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot spawn binary {}: {e}", bin.display()),
            };
        }
    };

    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        return OracleResult {
            id: spec.id.clone(),
            path: json_label.into(),
            verdict: Verdict::Rejected,
            detail: format!(
                "non-zero exit {}: {}",
                run.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ),
        };
    }

    let got = normalize_stdout(&String::from_utf8_lossy(&run.stdout));
    if got == expected {
        OracleResult {
            id: spec.id.clone(),
            path: json_label.into(),
            verdict: Verdict::Accepted,
            detail: "contract expect_stdout: build ok, exit 0, stdout match".into(),
        }
    } else {
        OracleResult {
            id: spec.id.clone(),
            path: json_label.into(),
            verdict: Verdict::Rejected,
            detail: format!("stdout mismatch: got={got:?} expected={expected:?}"),
        }
    }
}

fn run_expect_reject(
    json_label: &str,
    spec: &ContractSpec,
    source_full: &Path,
    code: &str,
) -> OracleResult {
    let src = match fs::read_to_string(source_full) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: spec.id.clone(),
                path: json_label.into(),
                verdict: Verdict::Inconclusive,
                detail: format!("cannot read source: {e}"),
            };
        }
    };

    match parse_lower_check(&src) {
        Ok(_) => OracleResult {
            id: spec.id.clone(),
            path: json_label.into(),
            verdict: Verdict::Rejected,
            detail: format!("expected reject with {code}, but parse_lower_check succeeded"),
        },
        Err(msg) => {
            if msg.contains(code) {
                OracleResult {
                    id: spec.id.clone(),
                    path: json_label.into(),
                    verdict: Verdict::Accepted,
                    detail: format!("contract expect_reject: rejected with {code}: {msg}"),
                }
            } else {
                OracleResult {
                    id: spec.id.clone(),
                    path: json_label.into(),
                    verdict: Verdict::Rejected,
                    detail: format!("failed without expected {code}: {msg}"),
                }
            }
        }
    }
}

/// Convert AST language contract expect → CLI ContractExpect.
fn lang_expect_to_cli(e: &arita_syntax::ContractExpect) -> ContractExpect {
    match e {
        arita_syntax::ContractExpect::Stdout(lines) => ContractExpect::Stdout(lines.clone()),
        arita_syntax::ContractExpect::Reject(code) => ContractExpect::Reject(code.clone()),
    }
}

/// Parse `.arita` and extract contracts as ContractSpecs (source = this file, relative to root).
pub fn specs_from_arita(
    root: &Path,
    arita_path: &Path,
    src: &str,
) -> Result<Vec<ContractSpec>, String> {
    // ADR-283 (Ingeniero addendum 2, cond. 1): this path never runs `arita_hir::check`, so an
    // AST carrying the parser's `<arita:deferred-shape>` marker must be rejected here with the
    // exact deferred parser error (E0001/E0006) — `parse_standalone` does that, never panics.
    let module = arita_syntax::parse_standalone(src).map_err(|e| e.to_string())?;
    if module.contracts.is_empty() {
        return Err("no contract { } in .arita (skip ≠ PASS)".into());
    }
    let rel = arita_path
        .strip_prefix(root)
        .unwrap_or(arita_path)
        .to_string_lossy()
        .replace("\\", "/");
    let mut out = Vec::with_capacity(module.contracts.len());
    for c in &module.contracts {
        out.push(ContractSpec {
            id: c.name.clone(),
            source: rel.clone(),
            expect: lang_expect_to_cli(&c.expect),
            source_sha256: None,
            target: c.target.clone(),
        });
    }
    Ok(out)
}

/// Run all language contracts in an `.arita` file. All must accept for Accepted;
/// any Rejected → Rejected; any Inconclusive / empty → Inconclusive.
pub fn run_lang_contract_file(root: &Path, arita_path: &Path) -> OracleResult {
    let label = arita_path
        .strip_prefix(root)
        .unwrap_or(arita_path)
        .to_string_lossy()
        .into_owned();

    if !arita_path.is_file() {
        return OracleResult {
            id: "lang-contract".into(),
            path: label,
            verdict: Verdict::Inconclusive,
            detail: format!("missing .arita: {}", arita_path.display()),
        };
    }

    let text = match fs::read_to_string(arita_path) {
        Ok(t) => t,
        Err(e) => {
            return OracleResult {
                id: "lang-contract".into(),
                path: label,
                verdict: Verdict::Inconclusive,
                detail: format!("cannot read .arita: {e}"),
            };
        }
    };

    let specs = match specs_from_arita(root, arita_path, &text) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: "lang-contract".into(),
                path: label,
                verdict: Verdict::Inconclusive,
                detail: e,
            };
        }
    };

    let mut details = Vec::new();
    let mut worst = Verdict::Accepted;
    let mut last_id = specs[0].id.clone();
    for spec in &specs {
        last_id = spec.id.clone();
        let r = run_contract_spec(root, &label, spec);
        details.push(format!("{}: {}", r.id, r.detail));
        match (worst, r.verdict) {
            (_, Verdict::Inconclusive) | (Verdict::Inconclusive, _) => {
                worst = Verdict::Inconclusive;
            }
            (Verdict::Accepted, Verdict::Rejected) | (Verdict::Rejected, Verdict::Rejected) => {
                worst = Verdict::Rejected;
            }
            (Verdict::Rejected, Verdict::Accepted) => {
                worst = Verdict::Rejected;
            }
            (Verdict::Accepted, Verdict::Accepted) => {}
        }
    }

    OracleResult {
        id: last_id,
        path: label,
        verdict: worst,
        detail: details.join("; "),
    }
}

/// Dispatch JSON vs language `.arita` contract path.
pub fn run_contract_path(root: &Path, path: &Path) -> OracleResult {
    let is_arita = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("arita"))
        .unwrap_or(false);
    if is_arita {
        run_lang_contract_file(root, path)
    } else {
        run_contract_file(root, path)
    }
}

/// Load JSON from path and run. Missing/unreadable/invalid → inconclusive.
pub fn run_contract_file(root: &Path, json_path: &Path) -> OracleResult {
    let label = json_path
        .strip_prefix(root)
        .unwrap_or(json_path)
        .to_string_lossy()
        .into_owned();

    if !json_path.is_file() {
        return OracleResult {
            id: "contract".into(),
            path: label,
            verdict: Verdict::Inconclusive,
            detail: format!("missing contract JSON: {}", json_path.display()),
        };
    }

    let text = match fs::read_to_string(json_path) {
        Ok(t) => t,
        Err(e) => {
            return OracleResult {
                id: "contract".into(),
                path: label,
                verdict: Verdict::Inconclusive,
                detail: format!("cannot read contract JSON: {e}"),
            };
        }
    };

    let spec = match parse_contract_json(&text) {
        Ok(s) => s,
        Err(e) => {
            return OracleResult {
                id: "contract".into(),
                path: label,
                verdict: Verdict::Inconclusive,
                detail: e,
            };
        }
    };

    run_contract_spec(root, &label, &spec)
}

pub fn run_contract_oracle(root: &Path, oracle: &ContractOracle) -> OracleResult {
    let full = root.join(oracle.path);
    let mut r = run_contract_path(root, &full);
    // Measure pin id is always authoritative for the oracle row.
    r.id = oracle.id.into();
    if oracle.expect_fail {
        match r.verdict {
            Verdict::Rejected => {
                r.verdict = Verdict::Accepted;
                r.detail = format!("expect_fail: correctly rejected — {}", r.detail);
            }
            Verdict::Accepted => {
                r.verdict = Verdict::Rejected;
                r.detail = format!("expect_fail: contract unexpectedly accepted — {}", r.detail);
            }
            Verdict::Inconclusive => {
                // stay inconclusive — never flip to PASS
            }
        }
    }
    r
}

/// `arita contract --verify-attest <json>` → same as `arita attest verify`.
pub fn cmd_contract_verify_attest(json_arg: &str) -> ExitCode {
    crate::attest::cmd_attest_verify(json_arg)
}

pub fn cmd_contract(path_arg: &str) -> ExitCode {
    let Some(root) = crate::measure::find_workspace_root() else {
        eprintln!("arita contract: cannot locate workspace root (Cargo.toml [workspace])");
        return ExitCode::from(2);
    };
    if let Err(e) = std::env::set_current_dir(&root) {
        eprintln!("arita contract: cannot cd to {}: {e}", root.display());
        return ExitCode::from(2);
    }

    let path = PathBuf::from(path_arg);
    let full = if path.is_absolute() {
        path
    } else {
        root.join(&path)
    };

    let is_arita = full
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("arita"))
        .unwrap_or(false);

    if is_arita {
        return cmd_contract_lang(&root, &full);
    }

    // JSON path (ADR-017/018).
    let text = match fs::read_to_string(&full) {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "arita contract inconclusive: cannot read {}: {e}",
                full.display()
            );
            return ExitCode::from(2);
        }
    };
    let spec = match parse_contract_json(&text) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("arita contract inconclusive: {e}");
            return ExitCode::from(2);
        }
    };

    let label = full
        .strip_prefix(&root)
        .unwrap_or(&full)
        .to_string_lossy()
        .into_owned();
    let result = run_contract_spec(&root, &label, &spec);

    match result.verdict {
        Verdict::Accepted => {
            let hash = match compute_source_sha256(&root, &spec) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("arita contract: accepted but cannot hash source: {e}");
                    return ExitCode::from(2);
                }
            };
            match crate::attest::write_attestation_sidecar(&root, Some(&full), &spec, &hash) {
                Ok(p) => {
                    println!("ok: {} — {}", result.id, result.detail);
                    println!("source_sha256={hash}");
                    println!("attest_sidecar={}", p.display());
                }
                Err(e) => {
                    println!("ok: {} — {}", result.id, result.detail);
                    println!("source_sha256={hash}");
                    eprintln!("arita contract: warning: could not write attest sidecar: {e}");
                }
            }
            ExitCode::SUCCESS
        }
        Verdict::Rejected => {
            eprintln!("arita contract rejected: {} — {}", result.id, result.detail);
            ExitCode::FAILURE
        }
        Verdict::Inconclusive => {
            eprintln!(
                "arita contract inconclusive: {} — {}",
                result.id, result.detail
            );
            ExitCode::from(2)
        }
    }
}

fn cmd_contract_lang(root: &Path, full: &Path) -> ExitCode {
    let text = match fs::read_to_string(full) {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "arita contract inconclusive: cannot read {}: {e}",
                full.display()
            );
            return ExitCode::from(2);
        }
    };
    let specs = match specs_from_arita(root, full, &text) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("arita contract inconclusive: {e}");
            return ExitCode::from(2);
        }
    };

    let label = full
        .strip_prefix(root)
        .unwrap_or(full)
        .to_string_lossy()
        .into_owned();

    let mut any_reject = false;
    let mut any_inconclusive = false;
    for spec in &specs {
        let result = run_contract_spec(root, &label, spec);
        match result.verdict {
            Verdict::Accepted => {
                let hash = match compute_source_sha256(root, spec) {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!("arita contract: accepted but cannot hash source: {e}");
                        return ExitCode::from(2);
                    }
                };
                // Sidecar for lang contracts (no JSON path); print computed hash only (ADR-019 OUT for surface attest fields).
                match crate::attest::write_attestation_sidecar(root, None, spec, &hash) {
                    Ok(p) => {
                        println!("ok: {} — {}", result.id, result.detail);
                        println!("source_sha256={hash}");
                        println!("attest_sidecar={}", p.display());
                    }
                    Err(e) => {
                        println!("ok: {} — {}", result.id, result.detail);
                        println!("source_sha256={hash}");
                        eprintln!("arita contract: warning: could not write attest sidecar: {e}");
                    }
                }
            }
            Verdict::Rejected => {
                eprintln!("arita contract rejected: {} — {}", result.id, result.detail);
                any_reject = true;
            }
            Verdict::Inconclusive => {
                eprintln!(
                    "arita contract inconclusive: {} — {}",
                    result.id, result.detail
                );
                any_inconclusive = true;
            }
        }
    }

    if any_inconclusive {
        ExitCode::from(2)
    } else if any_reject {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

// --- Minimal JSON value parser (contract schema only; no serde) ---

#[derive(Debug, Clone)]
pub(crate) enum Json {
    String(String),
    Array(Vec<Json>),
    Object(std::collections::BTreeMap<String, Json>),
    /// Numbers/bools/null accepted in parser but rejected by schema checks.
    Other,
}

pub(crate) fn parse_json_value(input: &str) -> Result<Json, String> {
    let mut p = Parser {
        chars: input.chars().collect(),
        i: 0,
    };
    let v = p.parse_value()?;
    p.skip_ws();
    if p.i != p.chars.len() {
        return Err("invalid JSON: trailing input after value".into());
    }
    Ok(v)
}

struct Parser {
    chars: Vec<char>,
    i: usize,
}

impl Parser {
    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.i += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.i += 1;
        Some(c)
    }

    fn expect_char(&mut self, want: char) -> Result<(), String> {
        self.skip_ws();
        match self.bump() {
            Some(c) if c == want => Ok(()),
            Some(c) => Err(format!("invalid JSON: expected '{want}', got '{c}'")),
            None => Err(format!("invalid JSON: expected '{want}', got EOF")),
        }
    }

    fn parse_value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.peek() {
            Some('"') => Ok(Json::String(self.parse_string()?)),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('t') => {
                self.consume_lit("true")?;
                Ok(Json::Other)
            }
            Some('f') => {
                self.consume_lit("false")?;
                Ok(Json::Other)
            }
            Some('n') => {
                self.consume_lit("null")?;
                Ok(Json::Other)
            }
            Some(c) if c == '-' || c.is_ascii_digit() => {
                self.parse_number()?;
                Ok(Json::Other)
            }
            Some(c) => Err(format!("invalid JSON: unexpected '{c}'")),
            None => Err("invalid JSON: unexpected EOF".into()),
        }
    }

    fn consume_lit(&mut self, lit: &str) -> Result<(), String> {
        for ch in lit.chars() {
            match self.bump() {
                Some(c) if c == ch => {}
                _ => return Err(format!("invalid JSON: expected `{lit}`")),
            }
        }
        Ok(())
    }

    fn parse_number(&mut self) -> Result<(), String> {
        if self.peek() == Some('-') {
            self.bump();
        }
        let mut saw = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                saw = true;
                self.bump();
            } else {
                break;
            }
        }
        if !saw {
            return Err("invalid JSON: expected number".into());
        }
        Ok(())
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect_char('"')?;
        let mut out = String::new();
        loop {
            match self.bump() {
                Some('"') => return Ok(out),
                Some('\\') => {
                    let esc = self
                        .bump()
                        .ok_or_else(|| "invalid JSON: truncated escape".to_string())?;
                    match esc {
                        '"' | '\\' | '/' => out.push(esc),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let mut hex = String::new();
                            for _ in 0..4 {
                                let h = self
                                    .bump()
                                    .ok_or_else(|| "invalid JSON: truncated \\u".to_string())?;
                                hex.push(h);
                            }
                            let code = u32::from_str_radix(&hex, 16)
                                .map_err(|_| format!("invalid JSON: bad \\u{hex}"))?;
                            let ch = char::from_u32(code)
                                .ok_or_else(|| format!("invalid JSON: bad \\u{hex}"))?;
                            out.push(ch);
                        }
                        c => return Err(format!("invalid JSON: unknown escape \\{c}")),
                    }
                }
                Some(c) => out.push(c),
                None => return Err("invalid JSON: unterminated string".into()),
            }
        }
    }

    fn parse_array(&mut self) -> Result<Json, String> {
        self.expect_char('[')?;
        self.skip_ws();
        let mut items = Vec::new();
        if self.peek() == Some(']') {
            self.bump();
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                }
                Some(']') => {
                    self.bump();
                    return Ok(Json::Array(items));
                }
                Some(c) => return Err(format!("invalid JSON: expected ',' or ']', got '{c}'")),
                None => return Err("invalid JSON: unterminated array".into()),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Json, String> {
        self.expect_char('{')?;
        self.skip_ws();
        let mut map = std::collections::BTreeMap::new();
        if self.peek() == Some('}') {
            self.bump();
            return Ok(Json::Object(map));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some('"') {
                return Err("invalid JSON: object key must be a string".into());
            }
            let key = self.parse_string()?;
            self.expect_char(':')?;
            let val = self.parse_value()?;
            map.insert(key, val);
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                }
                Some('}') => {
                    self.bump();
                    return Ok(Json::Object(map));
                }
                Some(c) => return Err(format!("invalid JSON: expected ',' or '}}', got '{c}'")),
                None => return Err("invalid JSON: unterminated object".into()),
            }
        }
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    use crate::measure::find_workspace_root;

    fn workspace() -> PathBuf {
        find_workspace_root().expect("workspace root for tests")
    }

    #[test]
    fn schema_error_both_expects() {
        let err = parse_contract_json(
            r#"{
            "id": "bad",
            "source": "ejemplos/01-hello.arita",
            "expect_stdout": ["hello"],
            "expect_reject": "E0224"
        }"#,
        )
        .expect_err("both expects must fail schema");
        assert!(err.contains("exactly one"), "detail={err}");
    }

    #[test]
    fn schema_error_neither_expect() {
        let err = parse_contract_json(
            r#"{
            "id": "bad",
            "source": "ejemplos/01-hello.arita"
        }"#,
        )
        .expect_err("neither expect must fail schema");
        assert!(err.contains("exactly one"), "detail={err}");
    }

    #[test]
    fn schema_error_missing_id() {
        let err = parse_contract_json(
            r#"{
            "source": "ejemplos/01-hello.arita",
            "expect_stdout": ["hello"]
        }"#,
        )
        .expect_err("missing id");
        assert!(err.contains("missing id"), "detail={err}");
    }

    #[test]
    fn parse_stdout_contract_ok() {
        let s = parse_contract_json(
            r#"{
            "id": "contract-hello",
            "source": "ejemplos/01-hello.arita",
            "expect_stdout": ["hello"]
        }"#,
        )
        .expect("parse");
        assert_eq!(s.id, "contract-hello");
        assert_eq!(s.expect, ContractExpect::Stdout(vec!["hello".into()]));
    }

    #[test]
    fn parse_reject_contract_ok() {
        let s = parse_contract_json(
            r#"{
            "id": "contract-neg-e0224",
            "source": "ejemplos/f2.3/neg/e0224-break-outside.arita",
            "expect_reject": "E0224"
        }"#,
        )
        .expect("parse");
        assert_eq!(s.expect, ContractExpect::Reject("E0224".into()));
    }

    #[test]
    fn stdout_match_hello_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd");
        let o = &CONTRACT_ORACLES[0];
        assert_eq!(o.id, "contract-hello");
        let r = run_contract_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "contract-hello must accept; detail={}",
            r.detail
        );
    }

    #[test]
    fn reject_match_e0224_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd");
        let o = CONTRACT_ORACLES
            .iter()
            .find(|o| o.id == "contract-neg-e0224")
            .expect("neg oracle");
        let r = run_contract_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "contract-neg-e0224 must accept; detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0224"), "detail={}", r.detail);
    }

    #[test]
    fn stdout_mismatch_rejected() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd");
        let spec = ContractSpec {
            id: "contract-bad-stdout".into(),
            source: "ejemplos/01-hello.arita".into(),
            expect: ContractExpect::Stdout(vec!["NOT_HELLO".into()]),
            source_sha256: None,
            target: None,
        };
        let r = run_contract_spec(&root, "inline-bad-stdout", &spec);
        assert_eq!(
            r.verdict,
            Verdict::Rejected,
            "mismatch must reject; detail={}",
            r.detail
        );
    }

    #[test]
    fn missing_source_inconclusive() {
        let root = workspace();
        let spec = ContractSpec {
            id: "contract-missing".into(),
            source: "ejemplos/__no_such_contract_src__.arita".into(),
            expect: ContractExpect::Stdout(vec!["x".into()]),
            source_sha256: None,
            target: None,
        };
        let r = run_contract_spec(&root, "inline-missing", &spec);
        assert_eq!(
            r.verdict,
            Verdict::Inconclusive,
            "missing source must be inconclusive; detail={}",
            r.detail
        );
    }

    #[test]
    fn invalid_schema_file_inconclusive() {
        let root = workspace();
        let dir = root.join("target/arita-contract-test");
        fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("bad-schema.json");
        fs::write(&path, r#"{"id":"x","source":"ejemplos/01-hello.arita"}"#).expect("write");
        let r = run_contract_file(&root, &path);
        assert_eq!(
            r.verdict,
            Verdict::Inconclusive,
            "bad schema must be inconclusive; detail={}",
            r.detail
        );
        assert!(
            r.detail.contains("exactly one") || r.detail.contains("invalid schema"),
            "detail={}",
            r.detail
        );
    }

    #[test]
    fn attested_hello_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd");
        let o = CONTRACT_ORACLES
            .iter()
            .find(|o| o.id == "contract-hello-attested")
            .expect("attested oracle");
        let r = run_contract_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "contract-hello-attested must accept; detail={}",
            r.detail
        );
        assert!(r.detail.contains("source_sha256="), "detail={}", r.detail);
    }

    #[test]
    fn bad_hash_measure_accepted_via_expect_fail() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd");
        let o = CONTRACT_ORACLES
            .iter()
            .find(|o| o.id == "contract-hello-bad-hash")
            .expect("bad-hash oracle");
        assert!(o.expect_fail);
        let r = run_contract_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "bad-hash expect_fail must accept; detail={}",
            r.detail
        );
        assert!(
            r.detail.contains("mismatch") || r.detail.contains("correctly rejected"),
            "detail={}",
            r.detail
        );
    }

    #[test]
    fn bad_hash_raw_contract_rejected() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd");
        let full = root.join("ejemplos/contracts/contract-hello-bad-hash.json");
        let r = run_contract_file(&root, &full);
        assert_eq!(
            r.verdict,
            Verdict::Rejected,
            "wrong sha must reject; detail={}",
            r.detail
        );
    }

    #[test]
    fn parse_optional_source_sha256() {
        let s = parse_contract_json(
            r#"{
            "id": "x",
            "source": "ejemplos/01-hello.arita",
            "source_sha256": "abc",
            "expect_stdout": ["hello"]
        }"#,
        )
        .expect("parse");
        assert_eq!(s.source_sha256.as_deref(), Some("abc"));
    }

    #[test]
    fn lang_hello_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd");
        let o = CONTRACT_ORACLES
            .iter()
            .find(|o| o.id == "lang-contract-hello")
            .expect("lang-hello oracle");
        let r = run_contract_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "lang-contract-hello must accept; detail={}",
            r.detail
        );
    }

    #[test]
    fn lang_neg_e0224_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd");
        let o = CONTRACT_ORACLES
            .iter()
            .find(|o| o.id == "lang-contract-neg-e0224")
            .expect("lang-neg oracle");
        let r = run_contract_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "lang-contract-neg-e0224 must accept; detail={}",
            r.detail
        );
        assert!(r.detail.contains("E0224"), "detail={}", r.detail);
    }

    #[test]
    fn parse_optional_target() {
        let s = parse_contract_json(
            r#"{
            "id": "x",
            "source": "ejemplos/contracts/lang-target-helper.arita",
            "target": "helper",
            "expect_stdout": ["from-helper"]
        }"#,
        )
        .expect("parse");
        assert_eq!(s.target.as_deref(), Some("helper"));
    }

    #[test]
    fn lang_target_helper_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd");
        let o = CONTRACT_ORACLES
            .iter()
            .find(|o| o.id == "lang-contract-target-helper")
            .expect("lang-target oracle");
        let r = run_contract_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "lang-contract-target-helper must accept; detail={}",
            r.detail
        );
    }

    #[test]
    fn json_target_helper_accepted() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd");
        let o = CONTRACT_ORACLES
            .iter()
            .find(|o| o.id == "contract-target-helper")
            .expect("json-target oracle");
        let r = run_contract_oracle(&root, o);
        assert_eq!(
            r.verdict,
            Verdict::Accepted,
            "contract-target-helper must accept; detail={}",
            r.detail
        );
    }

    #[test]
    fn missing_target_fn_rejected_e0230() {
        let root = workspace();
        std::env::set_current_dir(&root).expect("cd");
        let spec = ContractSpec {
            id: "missing-target".into(),
            source: "ejemplos/01-hello.arita".into(),
            expect: ContractExpect::Stdout(vec!["hello".into()]),
            source_sha256: None,
            target: Some("no_such_fn".into()),
        };
        let r = run_contract_spec(&root, "inline-missing-target", &spec);
        assert_eq!(
            r.verdict,
            Verdict::Rejected,
            "missing target must reject; detail={}",
            r.detail
        );
        assert!(
            r.detail.contains("E0230"),
            "must surface E0230; detail={}",
            r.detail
        );
    }
}

#[cfg(test)]
mod adr283_contract_deferred_shape {
    //! ADR-283 (Ingeniero addendum 2, cond. 1): `specs_from_arita` rejects (Err, no panic) any
    //! AST with the deferred-shape marker; valid contract files are unchanged.
    use super::*;

    #[test]
    fn adr283_specs_from_arita_rejects_deferred_shape_marker() {
        let root = Path::new("/r");
        let p = Path::new("/r/x.arita");
        // main without print + `v[0] = 2` → parser defers E0001 behind the marker.
        let bad = "module t\nfn main() -> Io<()> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  v[0] = 2\n}\ncontract c {\n  expect_reject \"E0344\"\n}\n";
        let ast = arita_syntax::parse(bad).expect("plain parse carries the marker");
        assert!(ast.functions.iter().any(|f| f.body.iter().any(|s| matches!(
            s,
            arita_syntax::Stmt::Expr(arita_syntax::Expr::Call(c))
                if c.callee == arita_syntax::DEFERRED_SHAPE_MARKER
        ))));
        let r = std::panic::catch_unwind(|| specs_from_arita(root, p, bad));
        let Ok(res) = r else {
            unreachable!("specs_from_arita panicked on deferred-shape marker");
        };
        let err = res.expect_err("marker AST must be rejected");
        assert!(err.starts_with("E0001"), "{err}");
        assert!(!err.contains("<arita:deferred-shape>"), "{err}");
    }

    #[test]
    fn adr283_specs_from_arita_valid_contracts_unchanged() {
        let root = Path::new("/r");
        let p = Path::new("/r/ejemplos/contracts/x.arita");
        let ok = "module t\nfn main() -> Io<()> {\n  print(\"hello\")\n}\ncontract hello_c {\n  expect_stdout \"hello\"\n}\n";
        let specs = specs_from_arita(root, p, ok).expect("valid contract file");
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].id, "hello_c");
        assert_eq!(specs[0].source, "ejemplos/contracts/x.arita");
        // Valid Vec-assign program in a Result fn carries no marker → same as plain parse.
        let vec_ok = "module t\nfn f() -> Result<Int, Int> {\n  let mut v: Vec<Int> = Vec::new()\n  v.push(1)\n  v[0] = 2\n  let n: Int = v.len()\n  Ok(n)\n}\nfn main() -> Io<()> {\n  print(\"x\")\n}\ncontract x_c {\n  expect_stdout \"x\"\n}\n";
        assert_eq!(specs_from_arita(root, p, vec_ok).expect("valid").len(), 1);
    }
}
