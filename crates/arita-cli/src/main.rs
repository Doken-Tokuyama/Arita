mod attest;
mod bootstrap;
mod contract;
mod deps;
mod lsp;
mod measure;
mod package;

use arita_codegen::{emit_rust, BuildProfile};
use arita_syntax::parse;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "help".into());
    match cmd.as_str() {
        "build" => {
            let rest: Vec<String> = args.collect();
            match parse_build_args(&rest) {
                Ok((profile, target, input)) => {
                    match build_with_entry_profile_target(
                        &input,
                        "main",
                        profile,
                        target.as_deref(),
                    ) {
                        Ok(out) => {
                            println!("ok: {}", out.bin.display());
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("{e}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{e}");
                    // E0250/E0260/E0261 = failure (1); usage = 2
                    if e.contains("E0250") || e.contains("E0260") || e.contains("E0261") {
                        ExitCode::FAILURE
                    } else {
                        ExitCode::from(2)
                    }
                }
            }
        }
        "parse" => {
            let Some(input) = args.next() else {
                eprintln!("usage: arita parse <file.arita>");
                return ExitCode::from(2);
            };
            match parse_only(&input) {
                Ok(name) => {
                    println!("ok: parsed module {name}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::FAILURE
                }
            }
        }
        "measure" => measure::cmd_measure(),
        "test" => {
            let Some(input) = args.next() else {
                eprintln!("usage: arita test <file.arita>");
                return ExitCode::from(2);
            };
            match run_tests(&input) {
                Ok(()) => {
                    println!("PASS");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::FAILURE
                }
            }
        }
        "logic" => {
            let Some(input) = args.next() else {
                eprintln!("usage: arita logic <file.arita>");
                return ExitCode::from(2);
            };
            match run_logic(&input) {
                Ok(()) => {
                    println!("true");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::FAILURE
                }
            }
        }
        "contract" => {
            let Some(first) = args.next() else {
                eprintln!("usage: arita contract <path.json|path.arita>");
                eprintln!("       arita contract --verify-attest <path-to-json>");
                return ExitCode::from(2);
            };
            if first == "--verify-attest" {
                let Some(input) = args.next() else {
                    eprintln!("usage: arita contract --verify-attest <path-to-json>");
                    return ExitCode::from(2);
                };
                contract::cmd_contract_verify_attest(&input)
            } else {
                contract::cmd_contract(&first)
            }
        }
        "attest" => {
            let Some(sub) = args.next() else {
                eprintln!("usage: arita attest verify <path-to-json>");
                return ExitCode::from(2);
            };
            if sub != "verify" {
                eprintln!("usage: arita attest verify <path-to-json>");
                return ExitCode::from(2);
            }
            let Some(input) = args.next() else {
                eprintln!("usage: arita attest verify <path-to-json>");
                return ExitCode::from(2);
            };
            attest::cmd_attest_verify(&input)
        }
        "lsp" => match lsp::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("E0400: lsp server error: {e}");
                ExitCode::FAILURE
            }
        },
        "version" => {
            println!("arita 0.1.0-f1");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!(
                "arita build [--profile debug|release] [--target <triple>] <file.arita>\narita parse <file.arita>\narita test <file.arita>\narita logic <file.arita>\narita contract <path.json|path.arita>\narita contract --verify-attest <path.json>\narita attest verify <path.json>\narita measure\narita lsp\narita version"
            );
            ExitCode::from(2)
        }
    }
}

pub(crate) fn parse_lower_check(src: &str) -> Result<arita_syntax::Module, String> {
    let module = parse(src).map_err(|e| e.to_string())?;
    let hir = arita_hir::lower_ast(&module);
    arita_hir::check(&hir).map_err(|e| e.to_string())?;
    Ok(module)
}

fn parse_only(input: &str) -> Result<String, String> {
    let src = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let module = parse_lower_check(&src)?;
    Ok(module.name)
}

/// Artifacts from a real emit → rustc/cargo build (ADR-028: cargo_toml for release evidence).
#[derive(Debug, Clone)]
pub(crate) struct BuildOutput {
    pub bin: PathBuf,
    /// Path to generated `Cargo.toml` when the Cargo emit path was used (async and/or release).
    pub cargo_toml: Option<PathBuf>,
}

/// Parse `arita build` args: `[--profile debug|release] [--target <triple>] <file.arita>`.
/// Unknown profile → **E0250**. Missing file → usage error.
/// ADR-034: `--target` composes with `--profile`; default = host (no flag).
pub(crate) fn parse_build_args(
    args: &[String],
) -> Result<(BuildProfile, Option<String>, String), String> {
    let usage = "usage: arita build [--profile debug|release] [--target <triple>] <file.arita>";
    let mut profile = BuildProfile::Debug;
    let mut target: Option<String> = None;
    let mut input: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "--profile" {
            i += 1;
            let Some(name) = args.get(i) else {
                return Err(usage.into());
            };
            profile = BuildProfile::parse(name)?;
        } else if let Some(name) = a.strip_prefix("--profile=") {
            profile = BuildProfile::parse(name)?;
        } else if a == "--target" {
            i += 1;
            let Some(t) = args.get(i) else {
                return Err(usage.into());
            };
            if t.is_empty() || t.starts_with('-') {
                return Err(format!("{usage} (empty/invalid --target)"));
            }
            target = Some(t.clone());
        } else if let Some(t) = a.strip_prefix("--target=") {
            if t.is_empty() {
                return Err(format!("{usage} (empty/invalid --target)"));
            }
            target = Some(t.to_string());
        } else if a.starts_with('-') {
            return Err(format!("{usage} (unknown flag `{a}`)"));
        } else if input.is_none() {
            input = Some(a.clone());
        } else {
            return Err(usage.into());
        }
        i += 1;
    }
    let Some(input) = input else {
        return Err(usage.into());
    };
    Ok((profile, target, input))
}

/// Shared build pipeline (default **debug**). Used by `arita measure` E2E oracles.
pub(crate) fn build(input: &str) -> Result<PathBuf, String> {
    Ok(build_with_profile(input, BuildProfile::Debug)?.bin)
}

/// ADR-028: build with explicit profile (`debug` | `release`).
pub(crate) fn build_with_profile(
    input: &str,
    profile: BuildProfile,
) -> Result<BuildOutput, String> {
    build_with_entry_profile(input, "main", profile)
}

/// ADR-020: build with contract entry rewrite (`emit_rust_with_entry`), default debug.
pub(crate) fn build_with_entry(input: &str, entry: &str) -> Result<PathBuf, String> {
    Ok(build_with_entry_profile(input, entry, BuildProfile::Debug)?.bin)
}

/// ADR-020 + ADR-028: entry rewrite + build profile (host target).
pub(crate) fn build_with_entry_profile(
    input: &str,
    entry: &str,
    profile: BuildProfile,
) -> Result<BuildOutput, String> {
    build_with_entry_profile_target(input, entry, profile, None)
}

/// ADR-034: entry rewrite + profile + optional `--target` triple.
pub(crate) fn build_with_entry_profile_target(
    input: &str,
    entry: &str,
    profile: BuildProfile,
    target: Option<&str>,
) -> Result<BuildOutput, String> {
    let arita_path = Path::new(input);
    // ADR-255: package workspace (arita.toml [workspace]) → emit lib+bin crates.
    if target.is_none() {
        if let Some(manifest) = package::load_package_manifest(arita_path)? {
            // ADR-287 D1-D3: `input` must be a workspace member (canonical path). Checked
            // BEFORE any mutation (no remove_dir_all / target/arita-pkg / cargo).
            let member = package::resolve_input_member(arita_path, &manifest)?;
            let (out_root, crates) = package::build_package_workspace_crates(arita_path, profile)?;
            // ADR-263/ADR-287 D4: runnable artifact of the INPUT's own member (not the first
            // `bin_*` of `read_dir`); a lib member has none → profile dir.
            let profile_dir = out_root.join("target").join(match profile {
                BuildProfile::Release => "release",
                BuildProfile::Debug => "debug",
            });
            return Ok(BuildOutput {
                bin: package::select_member_artifact(&profile_dir, &crates, &member),
                cargo_toml: Some(out_root.join("Cargo.toml")),
            });
        }
    }
    let src = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let module = parse_lower_check(&src)?;
    let deps = load_local_use_deps(arita_path, &module)?;
    let mut rust_src = if deps.is_empty() {
        arita_codegen::emit_rust_with_entry(&module, entry)?
    } else {
        arita_codegen::emit_rust_program(&module, &deps, entry)?
    };
    let needs_async = arita_codegen::needs_async_runtime(&module)
        || deps.iter().any(arita_codegen::needs_async_runtime);
    // ADR-029: load/validate arita.toml [deps] on Cargo-emit path needs.
    let declared = deps::load_deps_for_arita(arita_path)?;
    let with_tokio = arita_codegen::resolve_with_tokio(&declared, needs_async)?;
    // ADR-035: [host-bridges] → rewrite host:: + Cargo path dep.
    let hosts = deps::load_host_bridges_for_arita(arita_path)?;
    rust_src = arita_codegen::rewrite_host_calls(&rust_src, &hosts)?;
    let force_cargo = !declared.is_empty() || !hosts.is_empty() || target.is_some();
    compile_emitted(
        &module.name,
        &rust_src,
        profile,
        with_tokio,
        force_cargo,
        &hosts,
        target,
    )
}

/// ADR-254 / CORE-0.4-MULTI-MODULE: resolve `use mod::item` to sibling `{mod}.arita`.
/// Rejects missing file (E0404), module name mismatch (E0405), missing item (E0406).
fn load_local_use_deps(
    entry_path: &Path,
    entry: &arita_syntax::Module,
) -> Result<Vec<arita_syntax::Module>, String> {
    if entry.uses.is_empty() {
        return Ok(Vec::new());
    }
    let dir = entry_path
        .parent()
        .ok_or_else(|| "E0404: entry path has no parent directory".to_string())?;
    let mut deps = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for u in &entry.uses {
        if !seen.insert(u.module.clone()) {
            // same module used twice for different items — load once
            let dep = deps
                .iter()
                .find(|m: &&arita_syntax::Module| m.name == u.module)
                .ok_or_else(|| {
                    format!("E0404: internal: module `{}` missing after load", u.module)
                })?;
            if !dep.functions.iter().any(|f| f.name == u.item) {
                return Err(format!(
                    "E0406: item `{}` not found in module `{}`",
                    u.item, u.module
                ));
            }
            continue;
        }
        let dep_path = dir.join(format!("{}.arita", u.module));
        if !dep_path.is_file() {
            return Err(format!(
                "E0404: module file not found for `use {}::{}` (expected {})",
                u.module,
                u.item,
                dep_path.display()
            ));
        }
        let dep_src = fs::read_to_string(&dep_path).map_err(|e| e.to_string())?;
        let dep = parse_lower_check(&dep_src)?;
        if dep.name != u.module {
            return Err(format!(
                "E0405: file `{}` declares module `{}`, expected `{}`",
                dep_path.display(),
                dep.name,
                u.module
            ));
        }
        if dep.functions.iter().any(|f| f.name == "main") {
            return Err(format!(
                "E0405: dependency module `{}` must not define `main` (library module only)",
                u.module
            ));
        }
        if !dep.functions.iter().any(|f| f.name == u.item) {
            return Err(format!(
                "E0406: item `{}` not found in module `{}`",
                u.item, u.module
            ));
        }
        deps.push(dep);
    }
    Ok(deps)
}

/// True iff `rustup target list --installed` contains `triple`.
/// Spawn/rustup failure → false (measure maps to inconclusive; never fake PASS).
pub(crate) fn rustup_target_installed(triple: &str) -> bool {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .any(|l| l.trim() == triple),
        _ => false,
    }
}

/// Linker / rustup-target absence signals (ADR-034 gated → inconclusive, never fake reject).
pub(crate) fn cross_toolchain_unavailable(stderr: &str) -> bool {
    let s = stderr.to_lowercase();
    s.contains("may not be installed")
        || s.contains("isn't installed")
        || s.contains("is not installed")
        || s.contains("can't find crate for `std`")
        || s.contains("can't find crate for \"std\"")
        || s.contains("unable to find utility")
        || s.contains("linker `") && (s.contains("not found") || s.contains("no such file"))
        || s.contains("error: linker")
        || s.contains("could not find file")
            && (s.contains(".o") || s.contains("crt") || s.contains("libgcc"))
}

/// Compile emitted Rust. Debug+sync → rustc (unoptimized). Release / async / deps / --target → Cargo.
/// On reject → E0100 + stderr (no fake PASS).
pub(crate) fn compile_emitted(
    module_name: &str,
    rust_src: &str,
    profile: BuildProfile,
    with_tokio: bool,
    force_cargo: bool,
    hosts: &[arita_codegen::DeclaredHostBridge],
    target: Option<&str>,
) -> Result<BuildOutput, String> {
    // Release always uses Cargo so generated Cargo.toml carries real opt-level evidence.
    // Async / explicit `[deps]` / `[host-bridges]` / `--target` always uses Cargo (ADR-027/029/034/035).
    // Debug+sync host stays rustc-only.
    if profile == BuildProfile::Release || with_tokio || force_cargo || target.is_some() {
        compile_emitted_cargo(module_name, rust_src, profile, with_tokio, hosts, target)
    } else {
        compile_emitted_rust(module_name, rust_src)
    }
}

/// Compile emitted Rust with real rustc (debug/sync). No `-O` (unoptimized baseline).
/// Output names include a content stamp so parallel tests / ADR-020 entry rewrites
/// do not clobber the same binary (ETXTBSY / link races).
pub(crate) fn compile_emitted_rust(
    module_name: &str,
    rust_src: &str,
) -> Result<BuildOutput, String> {
    let out_dir = PathBuf::from("target/arita-out");
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let stamp = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        rust_src.hash(&mut h);
        std::thread::current().id().hash(&mut h);
        format!("{:x}", h.finish())
    };
    let stem = format!("{module_name}_{stamp}");
    let rs_path = out_dir.join(format!("{stem}.rs"));
    let bin_path = out_dir.join(&stem);
    fs::write(&rs_path, rust_src).map_err(|e| e.to_string())?;
    let output = Command::new("rustc")
        .arg(&rs_path)
        .arg("-o")
        .arg(&bin_path)
        .output()
        .map_err(|e| format!("failed to spawn rustc: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("E0100: emitted Rust failed rustc\n{stderr}"));
    }
    Ok(BuildOutput {
        bin: bin_path,
        cargo_toml: None,
    })
}

/// ADR-027/028/034: compile via generated Cargo crate (async and/or release and/or --target).
pub(crate) fn compile_emitted_cargo(
    module_name: &str,
    rust_src: &str,
    profile: BuildProfile,
    with_tokio: bool,
    hosts: &[arita_codegen::DeclaredHostBridge],
    target: Option<&str>,
) -> Result<BuildOutput, String> {
    let out_dir = PathBuf::from("target/arita-out");
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let stamp = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        rust_src.hash(&mut h);
        profile.as_str().hash(&mut h);
        with_tokio.hash(&mut h);
        target.hash(&mut h);
        hosts.len().hash(&mut h);
        for b in hosts {
            b.crate_name.hash(&mut h);
            b.path.hash(&mut h);
        }
        std::thread::current().id().hash(&mut h);
        format!("{:x}", h.finish())
    };
    let prefix = if target.is_some() {
        "arita_tgt"
    } else if with_tokio {
        "arita_async"
    } else if !hosts.is_empty() {
        "arita_host"
    } else {
        "arita_rel"
    };
    // Cargo package names: alphanumeric + underscore
    let pkg = format!("{prefix}_{module_name}_{stamp}")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    let crate_dir = out_dir.join(&pkg);
    let src_dir = crate_dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    let cargo_toml_path = crate_dir.join("Cargo.toml");
    // Resolve host path deps against workspace root (absolute → stable for out-of-tree crates).
    let mut host_path_deps: Vec<(String, String)> = Vec::new();
    if !hosts.is_empty() || rust_src.contains("arita_host_http::") {
        let root = measure::find_workspace_root()
            .ok_or_else(|| "cannot locate workspace root for host-bridges path deps".to_string())?;
        for b in hosts {
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
        // ADR-245: auto path-dep when emit references curated HTTP host.
        if rust_src.contains("arita_host_http::")
            && !host_path_deps
                .iter()
                .any(|(n, _)| n == "arita-host-http" || n == "arita_host_http")
        {
            let abs = root.join("crates/arita-host-http");
            if !abs.join("Cargo.toml").is_file() {
                return Err(format!(
                    "ADR-245 host http path missing Cargo.toml: {}",
                    abs.display()
                ));
            }
            host_path_deps.push((
                "arita-host-http".into(),
                abs.canonicalize().unwrap_or(abs).display().to_string(),
            ));
        }
    }
    let cargo_toml =
        arita_codegen::emit_cargo_toml_with_hosts(&pkg, profile, with_tokio, &host_path_deps);
    fs::write(&cargo_toml_path, &cargo_toml).map_err(|e| e.to_string())?;
    fs::write(src_dir.join("main.rs"), rust_src).map_err(|e| e.to_string())?;
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    if profile == BuildProfile::Release {
        cmd.arg("--release");
    }
    if let Some(t) = target {
        cmd.arg("--target").arg(t);
    }
    let output = cmd
        .arg("--manifest-path")
        .arg(&cargo_toml_path)
        .arg("--target-dir")
        .arg(crate_dir.join("target"))
        .output()
        .map_err(|e| format!("failed to spawn cargo: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let kind = if target.is_some() {
            "cross-target"
        } else if with_tokio {
            "async/tokio"
        } else if !hosts.is_empty() {
            "host-bridge"
        } else {
            "release"
        };
        return Err(format!(
            "E0100: emitted Rust failed cargo build ({kind})\n{stderr}"
        ));
    }
    let profile_dir = match profile {
        BuildProfile::Release => "release",
        BuildProfile::Debug => "debug",
    };
    // Host: target/<profile>/<pkg>; cross: target/<triple>/<profile>/<pkg>[.exe]
    let mut bin_path = crate_dir.join("target");
    if let Some(t) = target {
        bin_path = bin_path.join(t);
    }
    bin_path = bin_path.join(profile_dir);
    let bin_name = if target.is_some_and(|t| t.contains("windows")) {
        format!("{pkg}.exe")
    } else {
        pkg.clone()
    };
    bin_path = bin_path.join(&bin_name);
    if !bin_path.is_file() {
        return Err(format!(
            "E0100: cargo build ok but binary missing: {}",
            bin_path.display()
        ));
    }
    Ok(BuildOutput {
        bin: bin_path,
        cargo_toml: Some(cargo_toml_path),
    })
}

/// parse → emit_rust → rustc --test → run binary. Empty tests / skip ≠ PASS.
pub(crate) fn run_tests(input: &str) -> Result<(), String> {
    run_tests_impl(input, false)
}

/// Same as [`run_tests`], but the test binary's stdout report goes to **stderr**.
/// `arita measure` uses it so its own stdout stays a single JSON document.
pub(crate) fn run_tests_report_to_stderr(input: &str) -> Result<(), String> {
    run_tests_impl(input, true)
}

fn run_tests_impl(input: &str, report_to_stderr: bool) -> Result<(), String> {
    let src = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let module = parse_lower_check(&src)?;
    if module.tests.is_empty() {
        return Err("no tests in module (skip ≠ PASS)".into());
    }
    let rust_src = emit_rust(&module)?;
    let bin = compile_emitted_rust_test(&module.name, &rust_src)?;
    let output = Command::new(&bin)
        .output()
        .map_err(|e| format!("failed to spawn test binary: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        if report_to_stderr {
            eprint!("{stdout}");
        } else {
            print!("{stdout}");
        }
    }
    if !stderr.is_empty() {
        eprint!("{stderr}");
    }
    if !output.status.success() {
        return Err(format!(
            "tests failed (exit {})",
            output.status.code().unwrap_or(-1)
        ));
    }
    // rustc --test "ok" with 0 passed is skip, not PASS
    if !stdout.contains("test result: ok") || stdout.contains("0 passed;") {
        return Err("no tests ran (skip ≠ PASS)".into());
    }
    Ok(())
}

/// ADR-292: flags of `rustc` for `arita test`. `-O` turns overflow checks off by default, so
/// `-C overflow-checks=on` restores the same uniform panic as `arita build` (debug).
pub(crate) const RUSTC_TEST_FLAGS: [&str; 4] = ["--test", "-O", "-C", "overflow-checks=on"];

/// rustc --test -O -C overflow-checks=on -o <name>_test file.rs (same emit as build; cfg(test) module included).
fn compile_emitted_rust_test(module_name: &str, rust_src: &str) -> Result<PathBuf, String> {
    let out_dir = PathBuf::from("target/arita-out");
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let stamp = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        rust_src.hash(&mut h);
        std::thread::current().id().hash(&mut h);
        format!("{:x}", h.finish())
    };
    let stem = format!("{module_name}_{stamp}");
    let rs_path = out_dir.join(format!("{stem}.rs"));
    let bin_path = out_dir.join(format!("{stem}_test"));
    fs::write(&rs_path, rust_src).map_err(|e| e.to_string())?;
    let output = Command::new("rustc")
        .args(RUSTC_TEST_FLAGS)
        .arg("-o")
        .arg(&bin_path)
        .arg(&rs_path)
        .output()
        .map_err(|e| format!("failed to spawn rustc: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("E0100: emitted Rust failed rustc --test\n{stderr}"));
    }
    Ok(bin_path)
}

/// Parse + evaluate a logic-only `.arita` with the own positive-Datalog engine.
/// All queries sat → Ok; any fail / empty → Err(E0301…); never skip→ok.
pub(crate) fn run_logic(input: &str) -> Result<(), String> {
    match arita_logic::eval_file(input) {
        Ok(r) if r.all_sat => Ok(()),
        Ok(_) => Err("E0301: query failed".into()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{compile_emitted_rust, parse_build_args, run_tests, RUSTC_TEST_FLAGS};
    use arita_codegen::BuildProfile;

    #[test]
    fn adr292_test_cmd_rustc_flags_keep_o_and_add_overflow_checks() {
        assert_eq!(
            RUSTC_TEST_FLAGS,
            ["--test", "-O", "-C", "overflow-checks=on"]
        );
    }

    /// ADR-292: `a + b` overflow inside an `arita test` must panic (test fails), not wrap.
    /// The assertion would PASS under wrap (`MAX + 1 == MIN`), so a green run means no panic.
    #[test]
    fn adr292_arita_test_overflow_is_a_failed_test_not_a_wrap() {
        let dir = std::env::temp_dir().join(format!("arita_adr292_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tmp dir");
        let src = "module adr292_test_overflow\n\nfn add(a: Int, b: Int) -> Int {\n  a + b\n}\n\nfn main() -> Io<()> {\n  print(\"ok\")\n}\n\ntest wraps_would_pass {\n  assert add(9223372036854775807, 1) == -9223372036854775808\n}\n";
        let path = dir.join("t.arita");
        std::fs::write(&path, src).expect("write");
        let err = run_tests(path.to_str().expect("utf8")).expect_err("overflow must fail the test");
        assert_eq!(err.as_str(), "tests failed (exit 101)");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Control: a test without overflow still passes with the new flag.
    #[test]
    fn adr292_arita_test_without_overflow_still_passes() {
        let dir = std::env::temp_dir().join(format!("arita_adr292_testok_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tmp dir");
        let src = "module adr292_test_ok\n\nfn add(a: Int, b: Int) -> Int {\n  a + b\n}\n\nfn main() -> Io<()> {\n  print(\"ok\")\n}\n\ntest adds {\n  assert add(1, 2) == 3\n}\n";
        let path = dir.join("t.arita");
        std::fs::write(&path, src).expect("write");
        let res = run_tests(path.to_str().expect("utf8"));
        assert!(res.is_ok(), "plain test must pass, got: {res:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn e0100_when_rustc_rejects() {
        // Intentionally invalid Rust — real rustc must reject.
        let err = compile_emitted_rust("e0100_bad", "fn main() { not_a_thing!!! }\n")
            .expect_err("rustc should reject");
        assert!(
            err.starts_with("E0100:"),
            "expected E0100 prefix, got: {err}"
        );
        assert!(
            err.contains("error") || err.len() > 20,
            "expected rustc stderr in message, got: {err}"
        );
    }

    #[test]
    fn parse_build_args_profile_and_e0250() {
        let (p, t, f) = parse_build_args(&[
            "--profile".into(),
            "release".into(),
            "ejemplos/x.arita".into(),
        ])
        .expect("ok");
        assert_eq!(p, BuildProfile::Release);
        assert!(t.is_none());
        assert_eq!(f, "ejemplos/x.arita");
        let err = parse_build_args(&[
            "--profile".into(),
            "fantasma".into(),
            "ejemplos/x.arita".into(),
        ])
        .expect_err("E0250");
        assert!(err.contains("E0250"), "err={err}");
        assert!(err.contains("unknown build profile"), "err={err}");
    }

    #[test]
    fn parse_build_args_target_composes_with_profile() {
        let (p, t, f) = parse_build_args(&[
            "--profile".into(),
            "release".into(),
            "--target".into(),
            "x86_64-pc-windows-gnu".into(),
            "ejemplos/01-hello.arita".into(),
        ])
        .expect("ok");
        assert_eq!(p, BuildProfile::Release);
        assert_eq!(t.as_deref(), Some("x86_64-pc-windows-gnu"));
        assert_eq!(f, "ejemplos/01-hello.arita");
        let (p2, t2, f2) = parse_build_args(&[
            "--target=aarch64-unknown-linux-gnu".into(),
            "ejemplos/01-hello.arita".into(),
        ])
        .expect("ok");
        assert_eq!(p2, BuildProfile::Debug);
        assert_eq!(t2.as_deref(), Some("aarch64-unknown-linux-gnu"));
        assert_eq!(f2, "ejemplos/01-hello.arita");
    }
}
