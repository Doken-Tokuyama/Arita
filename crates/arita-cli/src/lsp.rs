//! ARITA LSP diagnostics v0 — CUT `LSP-DIAG-V0-20260914` (ADR-032).
//!
//! Stdio JSON-RPC. Push `textDocument/publishDiagnostics` on `didOpen` / `didChange`.
//! Codes + EN messages from the same parse/check pipeline as the CLI. No autocomplete,
//! rename, or pull diagnostics in v0.

use arita_hir::{check, lower_ast, CheckError};
use arita_syntax::{parse, ParseError, Span};
use lsp_server::{Connection, Message, Notification, Response};
use serde_json::{json, Value};
use std::error::Error;

/// One diagnostic ready for LSP publish (code stable E0xxx / E02xx…).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LspDiag {
    pub code: String,
    /// Canonical `E0xxx: English message` (no byte-span suffix).
    pub message: String,
    pub start: usize,
    pub end: usize,
}

/// Run parse → lower → check; map the first error to an LSP diagnostic list.
pub(crate) fn diagnose(src: &str) -> Vec<LspDiag> {
    match parse(src) {
        Err(e) => vec![diag_from_parse(e, src)],
        Ok(module) => {
            let hir = lower_ast(&module);
            match check(&hir) {
                Err(e) => vec![diag_from_check(e, src)],
                Ok(()) => Vec::new(),
            }
        }
    }
}

fn diag_from_parse(err: ParseError, src: &str) -> LspDiag {
    match err {
        ParseError::Coded {
            code,
            message,
            span,
        } => {
            let (start, end) = span_bytes(span, src);
            LspDiag {
                code: code.to_string(),
                message: format!("{code}: {message}"),
                start,
                end,
            }
        }
        ParseError::Message(m) => {
            let code = leading_code(&m).unwrap_or("E0006").to_string();
            let message = if m.starts_with(&code) {
                m.clone()
            } else {
                format!("{code}: {m}")
            };
            LspDiag {
                code,
                message,
                start: 0,
                end: fallback_end(src),
            }
        }
    }
}

fn diag_from_check(err: CheckError, src: &str) -> LspDiag {
    match err {
        CheckError::Coded { code, message } => LspDiag {
            code: code.to_string(),
            message: format!("{code}: {message}"),
            start: 0,
            end: fallback_end(src),
        },
    }
}

fn span_bytes(span: Option<Span>, src: &str) -> (usize, usize) {
    match span {
        Some(s) if s.end > s.start => (s.start.min(src.len()), s.end.min(src.len())),
        Some(s) => {
            let start = s.start.min(src.len());
            (start, (start + 1).min(src.len().max(1)))
        }
        None => (0, fallback_end(src)),
    }
}

fn fallback_end(src: &str) -> usize {
    if src.is_empty() {
        0
    } else {
        src.chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1)
            .min(src.len())
    }
}

fn leading_code(msg: &str) -> Option<&str> {
    let rest = msg.strip_prefix('E')?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 4 {
        Some(&msg[..1 + digits.len()])
    } else {
        None
    }
}

/// UTF-16 LSP position from a UTF-8 byte offset.
pub(crate) fn offset_to_position(src: &str, offset: usize) -> (u32, u32) {
    let offset = offset.min(src.len());
    let mut line: u32 = 0;
    let mut character: u32 = 0;
    for (i, ch) in src.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16() as u32;
        }
    }
    (line, character)
}

fn diag_to_json(src: &str, d: &LspDiag) -> Value {
    let (sl, sc) = offset_to_position(src, d.start);
    let (el, ec) = offset_to_position(src, d.end);
    json!({
        "range": {
            "start": { "line": sl, "character": sc },
            "end": { "line": el, "character": ec }
        },
        "severity": 1,
        "code": d.code,
        "source": "arita",
        "message": d.message,
    })
}

fn publish_diagnostics(
    conn: &Connection,
    uri: &str,
    src: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let diags = diagnose(src);
    let diagnostics: Vec<Value> = diags.iter().map(|d| diag_to_json(src, d)).collect();
    let params = json!({
        "uri": uri,
        "diagnostics": diagnostics,
    });
    conn.sender.send(Message::Notification(Notification::new(
        "textDocument/publishDiagnostics".into(),
        params,
    )))?;
    Ok(())
}

/// Run the stdio LSP server until shutdown/exit.
pub fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (connection, io_threads) = Connection::stdio();

    let (id, _params) = connection.initialize_start()?;
    let init_result = json!({
        "capabilities": {
            "textDocumentSync": 1
        },
        "serverInfo": {
            "name": "arita-lsp",
            "version": "0.1.0"
        }
    });
    connection.initialize_finish(id, init_result)?;

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                // v0: no autocomplete / rename / pull diagnostic — method not found.
                let resp = Response::new_err(
                    req.id,
                    lsp_server::ErrorCode::MethodNotFound as i32,
                    format!("unsupported in arita lsp v0: {}", req.method),
                );
                connection.sender.send(Message::Response(resp))?;
            }
            Message::Notification(note) => {
                handle_notification(&connection, &note)?;
            }
            Message::Response(_) => {}
        }
    }

    io_threads.join()?;
    Ok(())
}

fn handle_notification(
    conn: &Connection,
    note: &Notification,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match note.method.as_str() {
        "textDocument/didOpen" => {
            let uri = note.params["textDocument"]["uri"]
                .as_str()
                .ok_or("didOpen missing uri")?;
            let text = note.params["textDocument"]["text"]
                .as_str()
                .ok_or("didOpen missing text")?;
            publish_diagnostics(conn, uri, text)?;
        }
        "textDocument/didChange" => {
            let uri = note.params["textDocument"]["uri"]
                .as_str()
                .ok_or("didChange missing uri")?;
            // Full sync (textDocumentSync = 1): take the last change's full text.
            let changes = note.params["contentChanges"]
                .as_array()
                .ok_or("didChange missing contentChanges")?;
            let text = changes
                .last()
                .and_then(|c| c.get("text"))
                .and_then(|t| t.as_str())
                .ok_or("didChange missing text")?;
            publish_diagnostics(conn, uri, text)?;
        }
        "exit" => {
            // Shutdown path already broke the loop; ignore.
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnose_e0211_fixture() {
        let src = include_str!("../../../ejemplos/f2/neg/e0211-assert-true.arita");
        let diags = diagnose(src);
        assert_eq!(diags.len(), 1, "expected one diagnostic");
        assert_eq!(diags[0].code, "E0211");
        assert!(
            diags[0].message.contains("E0211"),
            "message={}",
            diags[0].message
        );
        assert!(
            diags[0].message.contains("assert requires evidence")
                || diags[0].message.contains("evidence"),
            "message={}",
            diags[0].message
        );
    }

    #[test]
    fn diagnose_clean_hello_empty() {
        let src = r#"module hello

fn main() -> Io<()> {
  print("hi")
}
"#;
        assert!(diagnose(src).is_empty());
    }

    #[test]
    fn offset_to_position_multiline() {
        let src = "a\nbc";
        assert_eq!(offset_to_position(src, 0), (0, 0));
        assert_eq!(offset_to_position(src, 2), (1, 0));
        assert_eq!(offset_to_position(src, 3), (1, 1));
    }
}
