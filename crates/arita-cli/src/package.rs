//! ADR-255 / CORE-0.4-PACKAGE-MANIFEST — `arita.toml` package + workspace emit `{lib,bin}`;
//! forbid crates.io (**E0330**); bad toml (**E0331**).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use arita_codegen::BuildProfile;

use crate::deps;
use crate::parse_lower_check;

/// Parsed `[package]` + `[workspace]` from `arita.toml` (v0).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub members: Vec<String>,
    pub path: PathBuf,
}

/// Load package manifest beside `entry` (walk up). No `[workspace]` → Ok(None).
pub fn load_package_manifest(entry: &Path) -> Result<Option<PackageManifest>, String> {
    let Some(toml_path) = deps::find_arita_toml(entry) else {
        return Ok(None);
    };
    let text = fs::read_to_string(&toml_path).map_err(|e| {
        format!(
            "E0331: cannot read package manifest {}: {e}",
            toml_path.display()
        )
    })?;
    reject_crates_io_deps(&text)?;
    let parsed = parse_package_toml(&text)?;
    let Some((name, version, members)) = parsed else {
        return Ok(None);
    };
    if members.is_empty() {
        return Err("E0331: [workspace] members must be non-empty".into());
    }
    Ok(Some(PackageManifest {
        name,
        version,
        members,
        path: toml_path,
    }))
}

/// Reject crates.io / versioned remote deps (**E0330**).
/// Allows ADR-029 `[deps] name = { bridge = "…" }` and path-only tables.
pub fn reject_crates_io_deps(text: &str) -> Result<(), String> {
    let mut in_deps = false;
    let mut in_dependencies = false;
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
            let sec = line;
            in_deps = sec == "[deps]";
            in_dependencies = sec == "[dependencies]" || sec.starts_with("[dependencies.");
            continue;
        }
        if !in_deps && !in_dependencies {
            continue;
        }
        let Some((_, rest)) = line.split_once('=') else {
            continue;
        };
        let rest = rest.trim();
        if rest.starts_with('"') || rest.starts_with('\'') {
            return Err(format!(
                "E0330: crates.io / versioned remote deps forbidden in arita.toml (line {line_no})"
            ));
        }
        if rest.starts_with('{') {
            let lower = rest.to_ascii_lowercase();
            if lower.contains("version")
                || lower.contains("registry")
                || lower.contains("git")
                || lower.contains("crates.io")
            {
                return Err(format!(
                    "E0330: crates.io / versioned remote deps forbidden in arita.toml (line {line_no})"
                ));
            }
        }
    }
    Ok(())
}

/// Parse `[package]` + `[workspace] members`. Ok(None) if no `[workspace]`.
pub fn parse_package_toml(text: &str) -> Result<Option<(String, String, Vec<String>)>, String> {
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    let mut members: Option<Vec<String>> = None;
    let mut section = String::new();
    let mut saw_workspace = false;

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
            section = line[1..line.len() - 1].to_string();
            if section == "workspace" {
                saw_workspace = true;
            }
            continue;
        }
        match section.as_str() {
            "package" => {
                let Some((k, v)) = line.split_once('=') else {
                    return Err(format!("E0331: bad [package] line {line_no}"));
                };
                let k = k.trim();
                let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                if v.is_empty() {
                    return Err(format!("E0331: empty [package].{k} on line {line_no}"));
                }
                match k {
                    "name" => name = Some(v),
                    "version" => version = Some(v),
                    _ => {}
                }
            }
            "workspace" => {
                let Some((k, v)) = line.split_once('=') else {
                    return Err(format!("E0331: bad [workspace] line {line_no}"));
                };
                if k.trim() != "members" {
                    continue;
                }
                let v = v.trim();
                if !(v.starts_with('[') && v.ends_with(']')) {
                    return Err(format!(
                        "E0331: workspace.members must be an array on line {line_no}"
                    ));
                }
                let inner = &v[1..v.len() - 1];
                let mut ms = Vec::new();
                for part in inner.split(',') {
                    let p = part.trim().trim_matches('"').trim_matches('\'');
                    if !p.is_empty() {
                        ms.push(p.to_string());
                    }
                }
                members = Some(ms);
            }
            _ => {}
        }
    }

    if !saw_workspace {
        return Ok(None);
    }
    let name = name.ok_or_else(|| "E0331: missing [package].name".to_string())?;
    let version = version.unwrap_or_else(|| "0.1.0".into());
    let members = members.ok_or_else(|| "E0331: missing [workspace].members".to_string())?;
    Ok(Some((name, version, members)))
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Emit root workspace `Cargo.toml` (members under `crates/`).
///
/// ADR-292: Cargo only reads `[profile.*]` from the workspace root, and the member manifests
/// carry none, so a release build gets `overflow-checks = true` from here (uniform panic on Int
/// overflow, like debug). Debug output is unchanged (Cargo `dev` already checks overflow).
pub fn emit_workspace_cargo_toml(manifest: &PackageManifest, profile: BuildProfile) -> String {
    let pkg = sanitize(&manifest.name);
    let mut out = String::from("[workspace]\nresolver = \"2\"\nmembers = [\n");
    for m in &manifest.members {
        let dir = if m == "lib" {
            format!("crates/lib_{pkg}")
        } else if m == "bin" {
            format!("crates/bin_{pkg}")
        } else {
            format!("crates/{}", sanitize(m))
        };
        out.push_str(&format!("  \"{dir}\",\n"));
    }
    out.push_str("]\n");
    if matches!(profile, BuildProfile::Release) {
        out.push_str("\n[profile.release]\n");
        out.push_str(arita_codegen::RELEASE_OVERFLOW_CHECKS_LINE);
        out.push('\n');
    }
    out
}

/// Resolve member → `.arita` under package root.
pub fn member_arita_path(pkg_root: &Path, member: &str) -> Result<PathBuf, String> {
    let candidates = [
        pkg_root.join(member).join(format!("{member}.arita")),
        pkg_root.join(member).join("mod.arita"),
        pkg_root.join(member).join("main.arita"),
        pkg_root.join(member).join("lib.arita"),
        pkg_root.join(format!("{member}.arita")),
    ];
    for c in &candidates {
        if c.is_file() {
            return Ok(c.clone());
        }
    }
    Err(format!(
        "E0331: workspace member `{member}` has no .arita under {}",
        pkg_root.display()
    ))
}

/// ADR-287 D1/D2/D3: `input` must be a member of the workspace `arita build` resolved.
///
/// Membership = canonical path of `input` equals the canonical path of the `.arita` of some
/// `members` entry (`member_arita_path`, unchanged candidate rules). Returns the member name.
///
/// Read-only: it never writes, removes or spawns anything, so it can run before any mutation.
/// * `input` unreadable / missing → plain I/O error (same text as the loose-file path), no E code.
/// * a member without `.arita` → that member's **E0331** is propagated (not hidden).
/// * not a member → **E0332**.
pub fn resolve_input_member(input: &Path, manifest: &PackageManifest) -> Result<String, String> {
    let input_canon = fs::canonicalize(input).map_err(|e| e.to_string())?;
    let pkg_root = manifest
        .path
        .parent()
        .ok_or("E0331: manifest has no parent")?;
    let mut found: Option<String> = None;
    for m in &manifest.members {
        let p = member_arita_path(pkg_root, m)?;
        let canon = fs::canonicalize(&p).unwrap_or(p);
        if found.is_none() && canon == input_canon {
            found = Some(m.clone());
        }
    }
    found.ok_or_else(|| {
        format!(
            "E0332: input {} is not a member of workspace {} (members: {})",
            input.display(),
            manifest.path.display(),
            manifest.members.join(", ")
        )
    })
}

/// ADR-287 D4: artifact printed after `ok:` for `member`. Deterministic (no `read_dir` order).
/// `crates` = `(member, crate dir name)` as emitted by [`build_package_workspace_crates`].
/// A `bin_*` crate of that member that exists as a plain file wins; otherwise (lib member)
/// the profile dir itself — never another member's `bin_*`.
pub fn select_member_artifact(
    profile_dir: &Path,
    crates: &[(String, String)],
    member: &str,
) -> PathBuf {
    for (m, crate_name) in crates {
        if m == member && crate_name.starts_with("bin_") {
            let p = profile_dir.join(crate_name);
            if p.is_file() {
                return p;
            }
        }
    }
    profile_dir.to_path_buf()
}

/// Emit + `cargo build` package workspace. Returns emit root path.
/// ADR-263: `cargo build` (not check) so `target/debug/bin_*` is runnable for contract/CLI.
pub fn build_package_workspace(entry: &Path, profile: BuildProfile) -> Result<PathBuf, String> {
    build_package_workspace_crates(entry, profile).map(|(root, _)| root)
}

/// Same as [`build_package_workspace`], also returning `(member, crate dir name)` per member.
pub fn build_package_workspace_crates(
    entry: &Path,
    profile: BuildProfile,
) -> Result<(PathBuf, Vec<(String, String)>), String> {
    let mut built_crates: Vec<(String, String)> = Vec::new();
    let manifest = load_package_manifest(entry)?
        .ok_or_else(|| "E0331: no [workspace] in arita.toml".to_string())?;
    let pkg_root = manifest
        .path
        .parent()
        .ok_or("E0331: manifest has no parent")?
        .to_path_buf();

    let out_root = pkg_root.join("target").join("arita-pkg");
    let _ = fs::remove_dir_all(&out_root);
    fs::create_dir_all(out_root.join("crates")).map_err(|e| e.to_string())?;
    fs::write(
        out_root.join("Cargo.toml"),
        emit_workspace_cargo_toml(&manifest, profile),
    )
    .map_err(|e| e.to_string())?;

    // ADR-275: workspace packages may declare [host-bridges] (host 238 I/O in lib/bin).
    let hosts = deps::load_host_bridges_for_arita(entry)?;
    let mut host_path_deps: Vec<(String, String)> = Vec::new();
    if !hosts.is_empty() {
        let root = crate::measure::find_workspace_root()
            .ok_or_else(|| "cannot locate workspace root for host-bridges path deps".to_string())?;
        for b in &hosts {
            let abs = root.join(&b.path);
            if !abs.join("Cargo.toml").is_file() {
                return Err(format!(
                    "host bridge path missing Cargo.toml: {}",
                    abs.display()
                ));
            }
            host_path_deps.push((
                b.crate_name.clone(),
                abs.canonicalize().unwrap_or(abs).display().to_string(),
            ));
        }
    }

    let pkg = sanitize(&manifest.name);
    let mut lib_crate_name: Option<String> = None;

    // Process lib members first so bin can depend on them.
    let mut ordered: Vec<&String> = Vec::new();
    for m in &manifest.members {
        if m == "lib" || m.starts_with("lib") {
            ordered.push(m);
        }
    }
    for m in &manifest.members {
        if !(m == "lib" || m.starts_with("lib")) {
            ordered.push(m);
        }
    }

    for member in ordered {
        let src_path = member_arita_path(&pkg_root, member)?;
        let src = fs::read_to_string(&src_path).map_err(|e| e.to_string())?;
        let mut module = arita_syntax::parse(&src).map_err(|e| e.to_string())?;
        // ADR-256: inject `pub record` from workspace lib into bin before HIR check.
        if !module.uses.is_empty() {
            inject_pub_items_from_uses(&pkg_root, &mut module)?;
        }
        let hir = arita_hir::lower_ast(&module);
        arita_hir::check(&hir).map_err(|e| e.to_string())?;
        let is_lib = member == "lib"
            || member.starts_with("lib")
            || module.functions.iter().all(|f| f.name != "main");

        let crate_dir_name = if member == "lib" {
            format!("lib_{pkg}")
        } else if member == "bin" {
            format!("bin_{pkg}")
        } else if is_lib {
            format!("lib_{}", sanitize(member))
        } else {
            format!("bin_{}", sanitize(member))
        };
        built_crates.push((member.clone(), crate_dir_name.clone()));
        let crate_dir = out_root.join("crates").join(&crate_dir_name);
        fs::create_dir_all(crate_dir.join("src")).map_err(|e| e.to_string())?;

        if is_lib {
            lib_crate_name = Some(crate_dir_name.clone());
            let mut rust = arita_codegen::emit_rust_lib(&module)?;
            rust = arita_codegen::rewrite_host_calls(&rust, &hosts)?;
            fs::write(crate_dir.join("src/lib.rs"), &rust).map_err(|e| e.to_string())?;
            let mut cargo = format!(
                "[package]\nname = \"{crate_dir_name}\"\nversion = \"{}\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
                manifest.version
            );
            if rust.contains("arita_host_") && !host_path_deps.is_empty() {
                cargo.push_str("\n[dependencies]\n");
                for (crate_name, abs_path) in &host_path_deps {
                    let path_esc = abs_path.replace('\\', "/").replace('"', "");
                    cargo.push_str(&format!("{crate_name} = {{ path = \"{path_esc}\" }}\n"));
                }
            }
            fs::write(crate_dir.join("Cargo.toml"), cargo).map_err(|e| e.to_string())?;
        } else {
            // Bin may `use lib::item` — for workspace, rewrite to `use lib_pkg::item` via path dep
            // when member name matches UseDecl.module == "lib" or the lib module name.
            let mut rust = if module.uses.is_empty() {
                arita_codegen::emit_rust_with_entry(&module, "main")?
            } else if let Some(ref lib_name) = lib_crate_name {
                // Load sibling deps via multi-module emit, then rewrite `mod X` → empty and use path crate
                let dir = src_path.parent().unwrap_or(pkg_root.as_path());
                let mut deps = Vec::new();
                for u in &module.uses {
                    let dep_path = dir.join(format!("{}.arita", u.module));
                    // also try ../lib/lib.arita layout
                    let alt = pkg_root.join("lib").join("lib.arita");
                    let path = if dep_path.is_file() {
                        dep_path
                    } else if u.module == "lib" && alt.is_file() {
                        alt
                    } else {
                        return Err(format!(
                            "E0404: module file not found for `use {}::{}`",
                            u.module, u.item
                        ));
                    };
                    let dsrc = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                    let dep_mod = parse_lower_check(&dsrc)?;
                    check_pub_export(&dep_mod, &u.item)?;
                    deps.push(dep_mod);
                }
                // Prefer path-dep crate: emit entry only + `use lib_pkg::item`
                // Strip injected lib records so we don't re-define them in the bin crate.
                let mut entry_only = module.clone();
                entry_only.uses.clear();
                let imported: std::collections::HashSet<&str> =
                    module.uses.iter().map(|u| u.item.as_str()).collect();
                entry_only
                    .records
                    .retain(|r| !imported.contains(r.name.as_str()));
                // ADR-263: strip injected lib fns — they live in the path-dep crate.
                // ADR-286 S1b: the stripped lib fns still have to be visible to the codegen
                // guards (`join(spawn(lib_fn()))` needs `lib_fn`'s async/ret), so pass their
                // signatures as externs instead of letting them vanish with the strip.
                let extern_sigs: Vec<arita_codegen::ExternFnSig> = module
                    .functions
                    .iter()
                    .filter(|f| imported.contains(f.name.as_str()))
                    .map(arita_codegen::ExternFnSig::from_function)
                    .collect();
                entry_only
                    .functions
                    .retain(|f| !imported.contains(f.name.as_str()));
                let mut body = arita_codegen::emit_rust_with_entry_and_sigs(
                    &entry_only,
                    "main",
                    &extern_sigs,
                )?;
                // Prepend uses of path crate items
                let mut use_block = String::new();
                for u in &module.uses {
                    use_block.push_str(&format!("use {}::{};\n", lib_name, u.item));
                }
                if let Some(pos) = body.find("#![forbid(unsafe_code)]\n") {
                    let insert_at = pos + "#![forbid(unsafe_code)]\n".len();
                    body.insert_str(insert_at, &use_block);
                } else {
                    body = format!("{use_block}{body}");
                }
                let _ = deps; // validated via parse_lower_check
                body
            } else {
                arita_codegen::emit_rust_with_entry(&module, "main")?
            };
            rust = arita_codegen::rewrite_host_calls(&rust, &hosts)?;
            fs::write(crate_dir.join("src/main.rs"), &rust).map_err(|e| e.to_string())?;
            let mut cargo = format!(
                "[package]\nname = \"{crate_dir_name}\"\nversion = \"{}\"\nedition = \"2021\"\n\n[[bin]]\nname = \"{crate_dir_name}\"\npath = \"src/main.rs\"\n",
                manifest.version
            );
            let need_deps = lib_crate_name.is_some()
                || (rust.contains("arita_host_") && !host_path_deps.is_empty());
            if need_deps {
                cargo.push_str("\n[dependencies]\n");
                if let Some(ref lib_name) = lib_crate_name {
                    cargo.push_str(&format!("{lib_name} = {{ path = \"../{lib_name}\" }}\n"));
                }
                if rust.contains("arita_host_") {
                    for (crate_name, abs_path) in &host_path_deps {
                        let path_esc = abs_path.replace('\\', "/").replace('"', "");
                        cargo.push_str(&format!("{crate_name} = {{ path = \"{path_esc}\" }}\n"));
                    }
                }
            }
            fs::write(crate_dir.join("Cargo.toml"), cargo).map_err(|e| e.to_string())?;
        }
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("build").current_dir(&out_root);
    if matches!(profile, BuildProfile::Release) {
        cmd.arg("--release");
    }
    let out = cmd
        .output()
        .map_err(|e| format!("E0100: cargo spawn: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "E0100: cargo build failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok((out_root, built_crates))
}

/// ADR-256: `use` across crate must name a `pub` item (**E0331** if private/missing).
/// ADR-256/263: bring `pub` records + `pub` fns from `use` targets into the bin module for HIR.
fn inject_pub_items_from_uses(
    pkg_root: &Path,
    module: &mut arita_syntax::Module,
) -> Result<(), String> {
    let dir = pkg_root; // members resolve under root
    for u in module.uses.clone() {
        let path = member_arita_path(dir, &u.module).or_else(|_| {
            let alt = dir.join("lib").join("lib.arita");
            if u.module == "lib" && alt.is_file() {
                Ok(alt)
            } else {
                Err(format!(
                    "E0404: module file not found for `use {}::{}`",
                    u.module, u.item
                ))
            }
        })?;
        let dsrc = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let dep = arita_syntax::parse(&dsrc).map_err(|e| e.to_string())?;
        check_pub_export(&dep, &u.item)?;
        if let Some(rec) = dep.records.iter().find(|r| r.name == u.item) {
            if !module.records.iter().any(|r| r.name == rec.name) {
                module.records.push(rec.clone());
            }
        }
        // ADR-263: inject pub fns so HIR can type cross-crate calls (Result/Option rets).
        if let Some(f) = dep.functions.iter().find(|f| f.name == u.item) {
            if !module.functions.iter().any(|x| x.name == f.name) {
                module.functions.push(f.clone());
            }
        }
    }
    Ok(())
}

fn check_pub_export(lib: &arita_syntax::Module, item: &str) -> Result<(), String> {
    if let Some(f) = lib.functions.iter().find(|f| f.name == item) {
        if f.is_pub {
            return Ok(());
        }
        return Err(format!(
            "E0331: item `{item}` is private; only `pub` crosses crate boundary"
        ));
    }
    if let Some(r) = lib.records.iter().find(|r| r.name == item) {
        if r.is_pub {
            return Ok(());
        }
        return Err(format!(
            "E0331: record `{item}` is private; only `pub` crosses crate boundary"
        ));
    }
    Err(format!(
        "E0406: item `{item}` not found in module `{}`",
        lib.name
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_workspace_members() {
        let t = r#"
[package]
name = "demo"
version = "0.1.0"

[workspace]
members = ["lib", "bin"]
"#;
        let (n, v, m) = parse_package_toml(t).unwrap().unwrap();
        assert_eq!(n, "demo");
        assert_eq!(v, "0.1.0");
        assert_eq!(m, vec!["lib", "bin"]);
    }

    #[test]
    fn e0330_version_string_dep() {
        let t = r#"
[package]
name = "demo"
[dependencies]
serde = "1.0"
"#;
        let e = reject_crates_io_deps(t).unwrap_err();
        assert!(e.contains("E0330"), "{e}");
    }

    #[test]
    fn bridge_dep_ok() {
        let t = r#"
[package]
name = "demo"
[deps]
tokio = { bridge = "async-runtime" }
"#;
        assert!(
            reject_crates_io_deps(t).is_ok(),
            "bridge dep must be accepted"
        );
    }

    #[test]
    fn bad_toml_missing_name() {
        let t = r#"
[workspace]
members = ["lib"]
"#;
        let e = parse_package_toml(t).unwrap_err();
        assert!(e.contains("E0331"), "{e}");
        assert!(e.contains("name"), "{e}");
    }

    #[test]
    fn emit_lists_lib_and_bin() {
        let m = PackageManifest {
            name: "demo".into(),
            version: "0.1.0".into(),
            members: vec!["lib".into(), "bin".into()],
            path: PathBuf::from("arita.toml"),
        };
        let t = emit_workspace_cargo_toml(&m, BuildProfile::Debug);
        assert!(t.contains("crates/lib_demo"), "{t}");
        assert!(t.contains("crates/bin_demo"), "{t}");
    }

    fn adr292_manifest() -> PackageManifest {
        PackageManifest {
            name: "demo".into(),
            version: "0.1.0".into(),
            members: vec!["lib".into(), "bin".into()],
            path: PathBuf::from("arita.toml"),
        }
    }

    #[test]
    fn adr292_workspace_root_release_has_profile_with_overflow_checks() {
        let t = emit_workspace_cargo_toml(&adr292_manifest(), BuildProfile::Release);
        assert_eq!(t.matches("[profile.release]").count(), 1, "{t}");
        assert_eq!(t.matches("overflow-checks = true").count(), 1, "{t}");
        // After the member list (the profile section is the last thing in the root manifest).
        let members_end = t.find("]\n").expect("members close");
        let prof = t.find("[profile.release]").expect("profile");
        assert!(prof > members_end, "{t}");
        assert!(
            t.ends_with("[profile.release]\noverflow-checks = true\n"),
            "{t}"
        );
        // Members and workspace header untouched.
        assert!(
            t.starts_with("[workspace]\nresolver = \"2\"\nmembers = [\n"),
            "{t}"
        );
        assert!(
            t.contains("crates/lib_demo") && t.contains("crates/bin_demo"),
            "{t}"
        );
        // No other profile knobs are introduced here.
        assert!(!t.contains("opt-level") && !t.contains("lto"), "{t}");
    }

    #[test]
    fn adr292_workspace_root_debug_is_unchanged() {
        let t = emit_workspace_cargo_toml(&adr292_manifest(), BuildProfile::Debug);
        assert_eq!(
            t,
            "[workspace]\nresolver = \"2\"\nmembers = [\n  \"crates/lib_demo\",\n  \"crates/bin_demo\",\n]\n"
        );
        assert!(!t.contains("[profile"), "{t}");
    }
}

#[cfg(test)]
mod adr287_member_tests {
    //! ADR-287 §4 unit tests (no cargo/rustc): membership by canonical path + D4 selector.
    use super::*;

    fn tree(tag: &str, members: &[&str]) -> (PathBuf, PackageManifest) {
        let root =
            std::env::temp_dir().join(format!("arita_adr287_unit_{tag}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for d in ["lib", "bin", "bin2"] {
            fs::create_dir_all(root.join(d)).unwrap();
        }
        fs::write(root.join("lib/lib.arita"), "module lib\n").unwrap();
        fs::write(root.join("bin/main.arita"), "module a\n").unwrap();
        fs::write(root.join("bin/other.arita"), "module o\n").unwrap();
        fs::write(root.join("bin2/main.arita"), "module b\n").unwrap();
        fs::write(root.join("arita.toml"), "").unwrap();
        let m = PackageManifest {
            name: "demo".into(),
            version: "0.1.0".into(),
            members: members.iter().map(|s| s.to_string()).collect(),
            path: root.join("arita.toml"),
        };
        (root, m)
    }

    #[test]
    fn member_input_direct_path() {
        let (r, m) = tree("direct", &["lib", "bin"]);
        assert_eq!(
            resolve_input_member(&r.join("bin/main.arita"), &m).unwrap(),
            "bin"
        );
        assert_eq!(
            resolve_input_member(&r.join("lib/lib.arita"), &m).unwrap(),
            "lib"
        );
        let _ = fs::remove_dir_all(&r);
    }

    #[test]
    fn member_input_dotdot_and_dot_forms() {
        let (r, m) = tree("dots", &["lib", "bin"]);
        let weird = r.join("./bin/../bin/main.arita");
        assert_eq!(resolve_input_member(&weird, &m).unwrap(), "bin");
        let abs = fs::canonicalize(r.join("bin/main.arita")).unwrap();
        assert_eq!(resolve_input_member(&abs, &m).unwrap(), "bin");
        let _ = fs::remove_dir_all(&r);
    }

    #[cfg(unix)]
    #[test]
    fn member_input_symlink() {
        let (r, m) = tree("sym", &["lib", "bin"]);
        let link = r.join("link.arita");
        std::os::unix::fs::symlink(r.join("bin/main.arita"), &link).unwrap();
        assert_eq!(resolve_input_member(&link, &m).unwrap(), "bin");
        let _ = fs::remove_dir_all(&r);
    }

    #[test]
    fn nonmember_input_is_e0332() {
        let (r, m) = tree("nonmem", &["lib", "bin"]);
        let e = resolve_input_member(&r.join("bin/other.arita"), &m).unwrap_err();
        assert!(e.starts_with("E0332:"), "{e}");
        assert!(
            e.contains("arita.toml") && e.contains("members: lib, bin"),
            "{e}"
        );
        let _ = fs::remove_dir_all(&r);
    }

    #[test]
    fn nonmember_check_has_no_side_effects() {
        let (r, m) = tree("noside", &["lib", "bin"]);
        let marker = r.join("target/arita-pkg/marker");
        fs::create_dir_all(marker.parent().unwrap()).unwrap();
        fs::write(&marker, "x").unwrap();
        assert!(resolve_input_member(&r.join("bin/other.arita"), &m).is_err());
        assert!(marker.is_file(), "marker must survive the failed check");
        let _ = fs::remove_dir_all(&r);
    }

    #[test]
    fn missing_input_is_io_error_not_e0332() {
        let (r, m) = tree("missing", &["lib", "bin"]);
        let e = resolve_input_member(&r.join("bin/nope.arita"), &m).unwrap_err();
        assert!(!e.contains("E0332"), "{e}");
        assert!(!r.join("target").exists(), "no build side effects");
        let _ = fs::remove_dir_all(&r);
    }

    #[test]
    fn member_without_arita_propagates_e0331() {
        let (r, m) = tree("e0331", &["lib", "ghost", "bin"]);
        let e = resolve_input_member(&r.join("bin/main.arita"), &m).unwrap_err();
        assert!(e.starts_with("E0331:"), "{e}");
        let _ = fs::remove_dir_all(&r);
    }

    #[test]
    fn lib_member_has_no_bin_artifact() {
        let (r, _m) = tree("libart", &["lib", "bin"]);
        let prof = r.join("target/arita-pkg/target/debug");
        fs::create_dir_all(&prof).unwrap();
        fs::write(prof.join("bin_demo"), "x").unwrap();
        let crates = vec![
            ("lib".to_string(), "lib_demo".to_string()),
            ("bin".to_string(), "bin_demo".to_string()),
        ];
        let p = select_member_artifact(&prof, &crates, "lib");
        assert_eq!(p, prof);
        assert!(!p.file_name().unwrap().to_string_lossy().starts_with("bin_"));
        assert_eq!(
            select_member_artifact(&prof, &crates, "bin"),
            prof.join("bin_demo")
        );
        let _ = fs::remove_dir_all(&r);
    }

    #[test]
    fn two_bin_members_pick_input_crate() {
        let (r, _m) = tree("twobin", &["bin", "bin2"]);
        let prof = r.join("target/arita-pkg/target/debug");
        fs::create_dir_all(&prof).unwrap();
        fs::write(prof.join("bin_demo"), "x").unwrap();
        fs::write(prof.join("bin_bin2"), "x").unwrap();
        for order in [
            vec![("bin", "bin_demo"), ("bin2", "bin_bin2")],
            vec![("bin2", "bin_bin2"), ("bin", "bin_demo")],
        ] {
            let crates: Vec<(String, String)> = order
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect();
            assert_eq!(
                select_member_artifact(&prof, &crates, "bin"),
                prof.join("bin_demo")
            );
            assert_eq!(
                select_member_artifact(&prof, &crates, "bin2"),
                prof.join("bin_bin2")
            );
        }
        let _ = fs::remove_dir_all(&r);
    }
}
