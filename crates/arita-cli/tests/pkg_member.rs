//! ADR-287 PACKAGE-MEMBER (`CORE-PKG-MEMBER-20261001`, B-286-10): PM-1..PM-7.
//! `arita build <input>` under a `[workspace]` only compiles an `input` that is a member
//! (canonical path). A non-member is `E0332`, exit 1, empty stdout, and NOTHING is mutated
//! (tree compared byte-for-byte before/after). Real binary, tmpdir copy of the fixture
//! `ejemplos/core10/pkg-member/`; skip ≠ PASS.
//! Layer producing E0332: arita-cli (`package::resolve_input_member`, called from
//! `build_with_entry_profile_target` in main.rs) — not HIR, not codegen.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const MEMBER_OUT: &str = "member-main:42";

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ejemplos/core10/pkg-member")
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("mkdir");
    for e in fs::read_dir(from).expect("read_dir") {
        let e = e.expect("entry");
        let p = e.path();
        let dst = to.join(e.file_name());
        if p.is_dir() {
            if e.file_name() == "target" {
                continue;
            }
            copy_tree(&p, &dst);
        } else {
            fs::copy(&p, &dst).expect("copy");
        }
    }
}

/// Fresh copy of the fixture (never built in place). Tag has no `bin_` component.
fn fresh_fixture(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arita_pm_{tag}_{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clean tmpdir");
    }
    copy_tree(&fixture_dir(), &dir);
    dir
}

/// rel path → bytes, for every file and (empty) dir marker under `root`.
fn snapshot(root: &Path) -> BTreeMap<String, Option<Vec<u8>>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Option<Vec<u8>>>) {
        for e in fs::read_dir(dir).expect("read_dir") {
            let p = e.expect("entry").path();
            let rel = p.strip_prefix(root).unwrap().display().to_string();
            if p.is_dir() {
                out.insert(format!("{rel}/"), None);
                walk(root, &p, out);
            } else {
                out.insert(rel, Some(fs::read(&p).expect("read")));
            }
        }
    }
    let mut m = BTreeMap::new();
    walk(root, root, &mut m);
    m
}

fn run_arita(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arita"))
        .current_dir(dir)
        // Hermetic: the workspace build must land in `<tmpdir>/target/arita-pkg/target/...`
        // even when the harness redirects cargo with CARGO_TARGET_DIR.
        .env_remove("CARGO_TARGET_DIR")
        .args(args)
        .output()
        .expect("spawn arita")
}

fn plant_marker(dir: &Path) -> PathBuf {
    let marker = dir.join("target").join("arita-pkg").join("marker");
    fs::create_dir_all(marker.parent().unwrap()).expect("mkdir marker dir");
    fs::write(&marker, b"keep-me").expect("write marker");
    marker
}

/// Path after `ok:`. `arita` prints it relative to ITS cwd (the tmpdir) when the input is
/// relative, so resolve against `dir` before touching the filesystem. The raw (printed) form is
/// what the D4 assertions on components look at.
fn ok_path_raw(out: &Output) -> PathBuf {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.trim_end();
    assert_eq!(line.lines().count(), 1, "one `ok:` line expected: {stdout}");
    PathBuf::from(
        line.strip_prefix("ok: ")
            .unwrap_or_else(|| panic!("no `ok:`: {stdout}")),
    )
}

fn ok_path(dir: &Path, out: &Output) -> PathBuf {
    dir.join(ok_path_raw(out)) // absolute printed paths replace `dir`
}

fn assert_build_ok(out: &Output) {
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "stderr={stderr}");
}

/// Run the artifact `ok:` named and return its stdout lines joined with ':' (print is println).
fn run_artifact(bin: &Path) -> String {
    let o = Command::new(bin).output().expect("run artifact");
    assert_eq!(o.status.code(), Some(0), "artifact exit");
    String::from_utf8_lossy(&o.stdout)
        .lines()
        .collect::<Vec<_>>()
        .join("")
}

/// Shared body of PM-2/4/5: non-member ⇒ E0332, exit 1, empty stdout, no mutation.
fn assert_nonmember_e0332(tag: &str, input: &str) -> String {
    let dir = fresh_fixture(tag);
    let marker = plant_marker(&dir);
    let before = snapshot(&dir);
    let out = run_arita(&dir, &["build", input]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "exit 1; stderr={stderr}");
    assert!(stdout.is_empty(), "stdout must be empty, got: {stdout}");
    assert!(
        stderr.trim_start().starts_with("E0332:"),
        "expected E0332, got stderr={stderr}"
    );
    assert!(
        stderr.contains("is not a member of workspace") && stderr.contains("arita.toml"),
        "message must name the manifest: {stderr}"
    );
    assert!(
        stderr.contains("members: lib, bin"),
        "members listed: {stderr}"
    );
    assert!(!stderr.contains("ok:"), "no ok: on stderr: {stderr}");
    assert_eq!(fs::read(&marker).expect("marker survives"), b"keep-me");
    assert_eq!(before, snapshot(&dir), "tree must be byte-identical");
    fs::remove_dir_all(&dir).expect("cleanup tmpdir");
    stderr.into_owned()
}

#[test]
fn pm1_member_bin_builds_and_ok_names_its_own_bin() {
    let dir = fresh_fixture("pm1");
    let out = run_arita(&dir, &["build", "bin/main.arita"]);
    assert_build_ok(&out);
    let bin = ok_path(&dir, &out);
    let name = bin.file_name().unwrap().to_string_lossy().into_owned();
    assert!(
        name.starts_with("bin_"),
        "bin_* expected: {}",
        bin.display()
    );
    assert!(bin.is_file(), "artifact exists: {}", bin.display());
    assert_eq!(run_artifact(&bin), MEMBER_OUT);
    assert!(out.stderr.is_empty(), "stderr empty");
    fs::remove_dir_all(&dir).expect("cleanup tmpdir");
}

#[test]
fn pm2_nonmember_other_is_e0332_no_mutation() {
    let stderr = assert_nonmember_e0332("pm2", "bin/other.arita");
    assert!(
        stderr.contains("bin/other.arita"),
        "E0332 must name the offending input: {stderr}"
    );
}

#[test]
fn pm3_member_dotdot_path_form_same_output() {
    let dir = fresh_fixture("pm3");
    let out = run_arita(&dir, &["build", "./bin/../bin/main.arita"]);
    assert_build_ok(&out);
    let bin = ok_path(&dir, &out);
    assert!(
        bin.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("bin_"),
        "{}",
        bin.display()
    );
    assert_eq!(run_artifact(&bin), MEMBER_OUT);
    // absolute form resolves to the same member too
    let abs = dir.join("bin/main.arita");
    let out2 = run_arita(&dir, &["build", abs.to_str().unwrap()]);
    assert_build_ok(&out2);
    assert_eq!(run_artifact(&ok_path(&dir, &out2)), MEMBER_OUT);
    fs::remove_dir_all(&dir).expect("cleanup tmpdir");
}

#[test]
fn pm4_nonmember_other_record_is_e0332_no_mutation() {
    let stderr = assert_nonmember_e0332("pm4", "bin/other-record.arita");
    assert!(
        stderr.contains("bin/other-record.arita"),
        "E0332 must name the offending input: {stderr}"
    );
}

#[test]
fn pm5_nonmember_other_edge_is_e0332_no_mutation() {
    let stderr = assert_nonmember_e0332("pm5", "bin/other-edge.arita");
    assert!(
        stderr.contains("bin/other-edge.arita"),
        "E0332 must name the offending input: {stderr}"
    );
}

#[test]
fn pm6_missing_input_is_io_error_not_e0332_no_mutation() {
    let dir = fresh_fixture("pm6");
    let marker = plant_marker(&dir);
    let before = snapshot(&dir);
    let out = run_arita(&dir, &["build", "bin/nope.arita"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "exit 1; stderr={stderr}");
    assert!(stdout.is_empty(), "stdout must be empty, got: {stdout}");
    assert!(
        !stderr.trim().is_empty(),
        "an I/O error is expected on stderr"
    );
    assert!(
        !stderr.contains("E0332"),
        "no E0332 for a missing file: {stderr}"
    );
    assert_eq!(fs::read(&marker).expect("marker survives"), b"keep-me");
    assert_eq!(before, snapshot(&dir), "tree must be byte-identical");
    fs::remove_dir_all(&dir).expect("cleanup tmpdir");
}

#[test]
fn pm7_member_lib_ok_names_profile_dir_no_bin_component() {
    let dir = fresh_fixture("pm7");
    let out = run_arita(&dir, &["build", "lib/lib.arita"]);
    assert_build_ok(&out);
    assert!(out.stderr.is_empty(), "stderr empty");
    let printed = ok_path_raw(&out);
    let p = ok_path(&dir, &out);
    assert!(
        p.ends_with("target/arita-pkg/target/debug"),
        "profile dir expected, got {}",
        p.display()
    );
    assert!(p.is_dir(), "profile dir exists");
    for c in printed.components() {
        let c = c.as_os_str().to_string_lossy();
        assert!(
            !c.starts_with("bin_"),
            "no bin_* component allowed: {}",
            printed.display()
        );
    }
    fs::remove_dir_all(&dir).expect("cleanup tmpdir");
}
