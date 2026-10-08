//! B-measure-stdout: `arita measure` must print exactly one JSON document on stdout.
//!
//! Regression: oracles with `require_test` call the in-process `arita test` path, which echoed the
//! test binary's report (`running 1 test` …) to our stdout and polluted the JSON.
//! This runs the real `arita measure` binary in a throw-away workspace that only contains
//! `ejemplos/f2/07-assert.arita` (a `require_test` oracle); every other oracle finds its inputs
//! missing and fails fast. The test asserts only on stdout shape and on that oracle's detail,
//! never on the overall verdict. It is NOT the full 844-id run (that is the dry/exclusive run).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fresh_tmpdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arita_mstdout_{tag}_{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clean tmpdir");
    }
    fs::create_dir_all(&dir).expect("create tmpdir");
    dir
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn measure_stdout_is_one_json_document() {
    let ws = fresh_tmpdir("ws");
    fs::write(ws.join("Cargo.toml"), "[workspace]\nmembers = []\n").expect("write Cargo.toml");
    let dst = ws.join("ejemplos").join("f2");
    fs::create_dir_all(&dst).expect("mkdir ejemplos/f2");
    fs::copy(
        repo_root().join("ejemplos/f2/07-assert.arita"),
        dst.join("07-assert.arita"),
    )
    .expect("copy 07-assert.arita");

    let out = Command::new(env!("CARGO_BIN_EXE_arita"))
        .current_dir(&ws)
        .arg("measure")
        .output()
        .expect("spawn arita measure");
    let stdout = String::from_utf8(out.stdout).expect("stdout is utf-8");

    // Whole stdout must parse as exactly one JSON value (serde_json rejects trailing text).
    let doc: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        let head: String = stdout.chars().take(300).collect();
        panic!("stdout is not a single JSON document: {e}; head={head:?}")
    });
    assert!(
        stdout.trim_start().starts_with('{'),
        "stdout must start with '{{'"
    );
    assert!(doc.get("verdict").and_then(|v| v.as_str()).is_some());
    let oracles = doc
        .get("oracles")
        .and_then(|v| v.as_array())
        .expect("oracles array");
    assert!(!oracles.is_empty());

    // The require_test path was really exercised (otherwise this test would prove nothing).
    let o = oracles
        .iter()
        .find(|o| o.get("id").and_then(|v| v.as_str()) == Some("f2-07-assert"))
        .expect("f2-07-assert oracle present");
    let detail = o.get("detail").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(
        o.get("verdict").and_then(|v| v.as_str()),
        Some("accepted"),
        "detail={detail}"
    );
    assert!(detail.contains("arita test PASS"), "detail={detail}");

    // The test report is still visible, on stderr.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("test result: ok"),
        "report must go to stderr"
    );
    let _ = fs::remove_dir_all(&ws);
}
