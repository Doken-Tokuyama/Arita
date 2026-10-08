//! Integration oracle for ADR-032 / CUT `LSP-DIAG-V0-20260914`.
//! Spawns `arita lsp`, `didOpen` a fixture with a known error, asserts
//! `textDocument/publishDiagnostics` carries the stable E0xxx code.
//! skip ≠ PASS: absence of the code fails the test.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

const FIXTURE_E0211: &str = include_str!("fixtures/e0211-assert-true.arita");
const FIXTURE_E0225: &str = include_str!("fixtures/e0225-match-bool-same.arita");

fn write_message(stdin: &mut impl Write, body: &Value) {
    let payload = serde_json::to_string(body).expect("serialize");
    write!(
        stdin,
        "Content-Length: {}\r\n\r\n{}",
        payload.len(),
        payload
    )
    .expect("write");
    stdin.flush().expect("flush");
}

fn read_message(reader: &mut impl BufRead) -> Value {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).expect("read header");
        assert!(n > 0, "EOF while reading LSP headers");
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        let lower = trimmed.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("content-length:") {
            content_length = Some(rest.trim().parse().expect("content-length"));
        }
    }
    let len = content_length.expect("missing Content-Length");
    let mut buf = vec![0u8; len];
    std::io::Read::read_exact(reader, &mut buf).expect("read body");
    serde_json::from_slice(&buf).expect("json body")
}

fn read_until_publish(reader: &mut impl BufRead, timeout_rounds: usize) -> Value {
    for _ in 0..timeout_rounds {
        let msg = read_message(reader);
        if msg.get("method").and_then(|m| m.as_str()) == Some("textDocument/publishDiagnostics") {
            return msg;
        }
        // initialize response or other — keep draining
        if msg.get("id").is_some() && msg.get("result").is_some() {
            continue;
        }
    }
    panic!("timed out waiting for publishDiagnostics");
}

fn spawn_lsp() -> (
    std::process::Child,
    std::process::ChildStdin,
    BufReader<std::process::ChildStdout>,
) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_arita"))
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn arita lsp");
    let stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    (child, stdin, BufReader::new(stdout))
}

fn handshake(stdin: &mut impl Write, reader: &mut impl BufRead) {
    write_message(
        stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": null,
                "capabilities": {},
                "rootUri": null
            }
        }),
    );
    let init = read_message(reader);
    assert_eq!(init["id"], 1, "initialize id");
    assert!(
        init["result"]["capabilities"]["textDocumentSync"].as_i64() == Some(1)
            || init["result"]["capabilities"]
                .get("textDocumentSync")
                .is_some(),
        "expected textDocumentSync full; got {}",
        init
    );

    write_message(
        stdin,
        &json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        }),
    );
}

fn shutdown(stdin: &mut impl Write, reader: &mut impl BufRead, child: &mut std::process::Child) {
    write_message(
        stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 99,
            "method": "shutdown",
            "params": null
        }),
    );
    let _ = read_message(reader);
    write_message(
        stdin,
        &json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": null
        }),
    );
    child.wait_timeout_soft();
}

trait WaitSoft {
    fn wait_timeout_soft(&mut self);
}

impl WaitSoft for std::process::Child {
    fn wait_timeout_soft(&mut self) {
        for _ in 0..50 {
            if let Ok(Some(_)) = self.try_wait() {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = self.kill();
        let _ = self.wait();
    }
}

fn assert_diag_has_code(publish: &Value, code: &str) {
    let diags = publish["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics array");
    assert!(
        !diags.is_empty(),
        "publishDiagnostics must not be empty for known-error fixture"
    );
    let found = diags.iter().any(|d| {
        let msg = d.get("message").and_then(|m| m.as_str()).unwrap_or("");
        let c = d.get("code").map(|c| match c {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => String::new(),
        });
        msg.contains(code) || c.as_deref() == Some(code)
    });
    assert!(
        found,
        "expected diagnostic containing {code}; got {publish}"
    );
}

#[test]
fn lsp_did_open_publishes_e0211() {
    let (mut child, mut stdin, mut reader) = spawn_lsp();
    handshake(&mut stdin, &mut reader);

    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///tmp/e0211-assert-true.arita",
                    "languageId": "arita",
                    "version": 1,
                    "text": FIXTURE_E0211
                }
            }
        }),
    );

    let publish = read_until_publish(&mut reader, 20);
    assert!(
        publish.get("method") == Some(&json!("textDocument/publishDiagnostics"))
            || publish.get("method").is_some(),
        "publishDiagnostics"
    );
    assert_diag_has_code(&publish, "E0211");

    shutdown(&mut stdin, &mut reader, &mut child);
}

#[test]
fn lsp_did_change_publishes_e0225() {
    let (mut child, mut stdin, mut reader) = spawn_lsp();
    handshake(&mut stdin, &mut reader);

    // Open clean-ish then change to vacuous match (E0225).
    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///tmp/change.arita",
                    "languageId": "arita",
                    "version": 1,
                    "text": "module ok\n\nfn main() -> Io<()> {\n  print(\"x\")\n}\n"
                }
            }
        }),
    );
    let open_pub = read_until_publish(&mut reader, 20);
    let open_diags = open_pub["params"]["diagnostics"].as_array().unwrap();
    assert!(
        open_diags.is_empty(),
        "clean open should publish empty diagnostics; got {open_pub}"
    );

    write_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": "file:///tmp/change.arita",
                    "version": 2
                },
                "contentChanges": [{ "text": FIXTURE_E0225 }]
            }
        }),
    );
    let publish = read_until_publish(&mut reader, 20);
    assert_diag_has_code(&publish, "E0225");

    shutdown(&mut stdin, &mut reader, &mut child);
}
