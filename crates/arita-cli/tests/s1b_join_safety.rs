//! ADR-286 S1b JOIN-SAFETY (`CORE-0.10-JOIN-SAFETY-20260927`, §0.1d (b) P3).
//! Codegen never drops a statement silently: `n + 1` as a non-final stmt in
//! `fn f(n: Int) -> Int` is accepted by HIR but must fail the build with E0006,
//! exit ≠ 0 and no binary. skip ≠ PASS: every assertion is on real CLI output.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SRC_P3: &str =
    "module sa\n\nfn f(n: Int) -> Int {\n  n + 1\n  n\n}\n\nfn main() -> Io<()> {\n  print(f(2))\n}\n";

const E0006_P3: &str = "E0006: cannot emit expression statement whose value is discarded in fn `f`";

fn fresh_tmpdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arita_s1b_{tag}_{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clean tmpdir");
    }
    fs::create_dir_all(&dir).expect("create tmpdir");
    dir
}

fn run_arita(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arita"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("spawn arita")
}

/// No artifact named after the module stem under the CLI output dir (`target/arita-out`).
fn assert_no_artifact(dir: &Path, stem: &str) {
    let out_dir = dir.join("target").join("arita-out");
    if let Ok(rd) = fs::read_dir(&out_dir) {
        let hits: Vec<String> = rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with(stem))
            .collect();
        assert!(hits.is_empty(), "no artifact expected, found: {hits:?}");
    }
}

#[test]
fn s1b_p3_discarded_value_stmt_build_is_e0006_exit_nonzero_no_binary() {
    let dir = fresh_tmpdir("p3_build");
    fs::write(dir.join("sa.arita"), SRC_P3).expect("write fixture");
    let out = run_arita(&dir, &["build", "sa.arita"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "build must fail; stdout={stdout} stderr={stderr}"
    );
    assert_ne!(out.status.code(), Some(0), "stderr={stderr}");
    assert!(
        stderr.trim_start().starts_with(E0006_P3),
        "expected `{E0006_P3}`, got stderr={stderr}"
    );
    assert!(
        !stdout.contains("ok:"),
        "no binary may be reported: {stdout}"
    );
    assert_no_artifact(&dir, "sa");
    fs::remove_dir_all(&dir).expect("cleanup tmpdir");
}

#[test]
fn s1b_p3_discarded_value_stmt_test_cmd_is_e0006_exit_nonzero_no_binary() {
    let dir = fresh_tmpdir("p3_test");
    let src = format!("{SRC_P3}\ntest t {{\n  assert f(2) == 2\n}}\n");
    fs::write(dir.join("st.arita"), src).expect("write fixture");
    let out = run_arita(&dir, &["test", "st.arita"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_ne!(out.status.code(), Some(0), "stderr={stderr}");
    assert!(
        stderr.trim_start().starts_with(E0006_P3),
        "expected `{E0006_P3}`, got stderr={stderr}"
    );
    assert_no_artifact(&dir, "st");
    fs::remove_dir_all(&dir).expect("cleanup tmpdir");
}

// ---- ADR-286 S1b decision (1): package path (lib fns stripped from the bin before emit) ----

const PKG_TOML: &str = "[package]\nname = \"pk\"\nversion = \"0.1.0\"\n\n[workspace]\nmembers = [\"lib\", \"bin\"]\n\n[deps]\ntokio = { bridge = \"async-runtime\" }\n";

fn write_pkg(dir: &Path, lib_src: &str, bin_src: &str) {
    fs::create_dir_all(dir.join("lib")).expect("mkdir lib");
    fs::create_dir_all(dir.join("bin")).expect("mkdir bin");
    fs::write(dir.join("arita.toml"), PKG_TOML).expect("write arita.toml");
    fs::write(dir.join("lib/lib.arita"), lib_src).expect("write lib");
    fs::write(dir.join("bin/main.arita"), bin_src).expect("write bin");
}

/// Layer: HIR (`type_of_spawn`, E0203). A lib `pub fn` returning `Result` launched from the bin
/// with `join(spawn(..))` never reaches codegen/`package.rs` emit; exit ≠ 0, no bin artifact.
#[test]
fn s1b_pkg_lib_result_fn_spawned_from_bin_is_e0203_exit_nonzero() {
    let dir = fresh_tmpdir("pkg_result");
    write_pkg(
        &dir,
        "module lib\n\npub fn lib_res() -> Result<Int, Int> {\n  Err(7)\n}\n",
        "module pk\n\nuse lib::lib_res\n\nasync fn main() -> Io<()> {\n  await join(spawn(lib_res()))\n  print(\"after\")\n}\n",
    );
    let out = run_arita(&dir, &["build", "bin/main.arita"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_ne!(out.status.code(), Some(0), "stderr={stderr}");
    assert!(
        stderr.trim_start().starts_with(
            "E0203: type mismatch: `spawn` expects async fn returning Io<()>, `lib_res` returns Result<Int, Int>"
        ),
        "got stderr={stderr}"
    );
    assert!(
        !stdout.contains("ok:"),
        "no binary may be reported: {stdout}"
    );
    fs::remove_dir_all(&dir).expect("cleanup tmpdir");
}

/// A lib `async fn` cannot cross the crate boundary today: the grammar has no `pub async fn`
/// (arita.pest `async_function` has no `pub_vis`) and a private fn is E0331. Positive
/// `join(spawn(lib_fn()))` from a bin therefore needs a syntax change (not in S1b scope);
/// the package.rs → `emit_rust_with_entry_and_sigs` path is covered at codegen level by
/// `s1b_extern_sigs_allow_io_unit_and_reject_result`. This pins the current layer (HIR/package).
///
/// KNOWN LIMITATION, NOT equivalent coverage of the positive case (1)(a): this test only pins the
/// CURRENT state (E0331 from `package.rs` `check_pub_export`). It does not exercise
/// `join(spawn(lib_fn()))` from a package bin end to end; the codegen-level tests only cover the
/// `emit_rust_with_entry_and_sigs` path, not the package build. Tracked as B-286-13 (P2):
/// `pub async fn` in arita-syntax + async/tokio support in the bin of a package. When B-286-13
/// lands, REPLACE this test with the positive one (a package whose lib exposes an `Io<()>` async
/// fn, a bin doing `await join(spawn(lib_fn()))`, and a check of the observable stdout).
#[test]
fn s1b_pkg_lib_async_fn_cannot_cross_crate_today_e0331() {
    let dir = fresh_tmpdir("pkg_async_priv");
    write_pkg(
        &dir,
        "module lib\n\nasync fn lib_fn() -> Io<()> {\n  print(\"lib-ran\")\n}\n",
        "module pk\n\nuse lib::lib_fn\n\nasync fn main() -> Io<()> {\n  await join(spawn(lib_fn()))\n  print(\"after\")\n}\n",
    );
    let out = run_arita(&dir, &["build", "bin/main.arita"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_ne!(out.status.code(), Some(0), "stderr={stderr}");
    assert!(
        stderr
            .trim_start()
            .starts_with("E0331: item `lib_fn` is private; only `pub` crosses crate boundary"),
        "got stderr={stderr}"
    );
    fs::remove_dir_all(&dir).expect("cleanup tmpdir");
}
