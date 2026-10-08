//! ADR-029 DEPS-V0 — load/validate `arita.toml` `[deps]` (whitelist + bridges).
//! ADR-035 — also load `[host-bridges]` (separate from `[deps]`).
//!
//! Manifest form (only): `arita.toml` next to the entry `.arita` or package root.
//! Embedded `deps { }` in `.arita` = OUT v0 (illegal_top / keyword).

use arita_codegen::{
    validate_declared_deps, validate_host_bridges, DeclaredDep, DeclaredHostBridge,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Locate `arita.toml` beside the entry `.arita`, else walk up parents looking for one
/// that sits next to a package root cue (`arita.toml` itself is the cue).
pub fn find_arita_toml(arita_file: &Path) -> Option<PathBuf> {
    let mut dir = arita_file.parent()?.to_path_buf();
    for _ in 0..8 {
        let candidate = dir.join("arita.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
    None
}

/// Load `[deps]` from `arita.toml`. Missing file → empty list (async path still works).
/// Malformed / unknown crate → **E0260**.
pub fn load_deps_for_arita(arita_file: &Path) -> Result<Vec<DeclaredDep>, String> {
    let Some(toml_path) = find_arita_toml(arita_file) else {
        return Ok(Vec::new());
    };
    let text = fs::read_to_string(&toml_path).map_err(|e| {
        format!(
            "E0260: dependency not in ARITA whitelist (cannot read {}: {e})",
            toml_path.display()
        )
    })?;
    let deps = parse_arita_toml_deps(&text)?;
    validate_declared_deps(&deps)?;
    Ok(deps)
}

/// Minimal TOML slice parser for `[deps]` only (no full TOML crate required).
///
/// Accepted line form:
/// `tokio = { bridge = "async-runtime" }`
///
/// Rejects demo-key style names (`secret`, `api_key`, `token`, `password`) as E0260.
pub fn parse_arita_toml_deps(text: &str) -> Result<Vec<DeclaredDep>, String> {
    let mut in_deps = false;
    let mut deps = Vec::new();
    for (lineno, raw) in text.lines().enumerate() {
        let line_no = lineno + 1;
        let mut line = raw.trim();
        if let Some(hash) = line.find('#') {
            line = line[..hash].trim();
        }
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_deps = line == "[deps]";
            continue;
        }
        if !in_deps {
            continue;
        }
        // name = { bridge = "id" }
        let Some((name_part, rest)) = line.split_once('=') else {
            return Err(format!(
                "E0260: dependency not in ARITA whitelist (bad [deps] line {line_no})"
            ));
        };
        let name = name_part.trim();
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(format!(
                "E0260: dependency not in ARITA whitelist (bad crate name on line {line_no})"
            ));
        }
        let lower = name.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "secret" | "api_key" | "apikey" | "token" | "password" | "demo_key" | "demokey"
        ) {
            return Err(
                "E0260: dependency not in ARITA whitelist (demo keys / secrets not allowed)"
                    .to_string(),
            );
        }
        let rest = rest.trim();
        let bridge = parse_bridge_table(rest).ok_or_else(|| {
            format!(
                "E0260: dependency not in ARITA whitelist (expected `{{ bridge = \"…\" }}` on line {line_no})"
            )
        })?;
        deps.push(DeclaredDep {
            name: name.to_string(),
            bridge,
        });
    }
    Ok(deps)
}

fn parse_bridge_table(rest: &str) -> Option<String> {
    let rest = rest.trim();
    if !(rest.starts_with('{') && rest.ends_with('}')) {
        return None;
    }
    let inner = rest[1..rest.len() - 1].trim();
    // bridge = "…"
    let mut bridge_val: Option<String> = None;
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (k, v) = part.split_once('=')?;
        let k = k.trim();
        let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
        if k == "bridge" {
            if v.is_empty() {
                return None;
            }
            bridge_val = Some(v);
        } else {
            // unknown keys in dep table → reject (keep whitelist strict)
            return None;
        }
    }
    bridge_val
}

/// Load `[host-bridges]` from `arita.toml`. Missing file / section → empty.
pub fn load_host_bridges_for_arita(arita_file: &Path) -> Result<Vec<DeclaredHostBridge>, String> {
    let Some(toml_path) = find_arita_toml(arita_file) else {
        return Ok(Vec::new());
    };
    let text = fs::read_to_string(&toml_path)
        .map_err(|e| format!("cannot read {}: {e}", toml_path.display()))?;
    let bridges = parse_arita_toml_host_bridges(&text)?;
    validate_host_bridges(&bridges)?;
    Ok(bridges)
}

/// Minimal TOML slice parser for `[host-bridges]`.
///
/// Accepted line form:
/// `demo = { crate = "arita-host-demo", path = "crates/arita-host-demo" }`
pub fn parse_arita_toml_host_bridges(text: &str) -> Result<Vec<DeclaredHostBridge>, String> {
    let mut in_host = false;
    let mut bridges = Vec::new();
    for (lineno, raw) in text.lines().enumerate() {
        let line_no = lineno + 1;
        let mut line = raw.trim();
        if let Some(hash) = line.find('#') {
            line = line[..hash].trim();
        }
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_host = line == "[host-bridges]";
            continue;
        }
        if !in_host {
            continue;
        }
        let Some((name_part, rest)) = line.split_once('=') else {
            return Err(format!("bad [host-bridges] line {line_no}"));
        };
        let id = name_part.trim();
        if id.is_empty()
            || !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(format!("bad host-bridge id on line {line_no}"));
        }
        let lower = id.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "secret" | "api_key" | "apikey" | "token" | "password" | "demo_key" | "demokey"
        ) {
            return Err(
                "E0260: dependency not in ARITA whitelist (demo keys / secrets not allowed)"
                    .to_string(),
            );
        }
        let rest = rest.trim();
        let (crate_name, path) = parse_host_bridge_table(rest).ok_or_else(|| {
            format!("expected `{{ crate = \"…\", path = \"…\" }}` on [host-bridges] line {line_no}")
        })?;
        bridges.push(DeclaredHostBridge {
            id: id.to_string(),
            crate_name,
            path,
        });
    }
    Ok(bridges)
}

fn parse_host_bridge_table(rest: &str) -> Option<(String, String)> {
    let rest = rest.trim();
    if !(rest.starts_with('{') && rest.ends_with('}')) {
        return None;
    }
    let inner = rest[1..rest.len() - 1].trim();
    let mut crate_name: Option<String> = None;
    let mut path: Option<String> = None;
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (k, v) = part.split_once('=')?;
        let k = k.trim();
        let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
        if v.is_empty() {
            return None;
        }
        match k {
            "crate" => crate_name = Some(v),
            "path" => path = Some(v),
            _ => return None,
        }
    }
    Some((crate_name?, path?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_deps_tokio_ok() {
        let t = r#"
[package]
name = "demo"

[deps]
tokio = { bridge = "async-runtime" }
"#;
        let deps = parse_arita_toml_deps(t).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "tokio");
        assert_eq!(deps[0].bridge, "async-runtime");
        validate_declared_deps(&deps).unwrap();
    }

    #[test]
    fn parse_deps_serde_e0260() {
        let t = r#"
[deps]
serde = { bridge = "json" }
"#;
        let deps = parse_arita_toml_deps(t).unwrap();
        let err = validate_declared_deps(&deps).unwrap_err();
        assert!(err.contains("E0260"));
        assert!(err.contains("dependency not in ARITA whitelist"));
    }

    #[test]
    fn find_arita_toml_beside_entry() {
        let dir = std::env::temp_dir().join(format!("arita_deps_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let arita = dir.join("prog.arita");
        let toml = dir.join("arita.toml");
        fs::write(&arita, "module x\n").unwrap();
        let mut f = fs::File::create(&toml).unwrap();
        writeln!(f, "[deps]").unwrap();
        writeln!(f, "tokio = {{ bridge = \"async-runtime\" }}").unwrap();
        let found = find_arita_toml(&arita).expect("find");
        assert_eq!(found, toml);
        let deps = load_deps_for_arita(&arita).unwrap();
        assert_eq!(deps.len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_host_bridges_demo_ok() {
        let t = r#"
[package]
name = "demo"

[host-bridges]
demo = { crate = "arita-host-demo", path = "crates/arita-host-demo" }
"#;
        let b = parse_arita_toml_host_bridges(t).unwrap();
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].id, "demo");
        assert_eq!(b[0].crate_name, "arita-host-demo");
        assert_eq!(b[0].path, "crates/arita-host-demo");
        validate_host_bridges(&b).unwrap();
    }

    #[test]
    fn host_bridges_not_mixed_with_deps_section() {
        let t = r#"
[deps]
tokio = { bridge = "async-runtime" }

[host-bridges]
demo = { crate = "arita-host-demo", path = "crates/arita-host-demo" }
"#;
        let deps = parse_arita_toml_deps(t).unwrap();
        let hosts = parse_arita_toml_host_bridges(t).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(hosts.len(), 1);
    }
}
