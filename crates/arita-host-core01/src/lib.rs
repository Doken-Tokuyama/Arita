//! ADR-238 / CORE-0.1-FILES-JSON-CLI — curated host bindings (safe).
#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;

/// Read UTF-8 file → Ok(text) / Err(0).
pub fn read_text(path: String) -> Result<String, i64> {
    fs::read_to_string(Path::new(&path)).map_err(|_| 0i64)
}

/// Write UTF-8 file → Ok(0) / Err(0).
pub fn write_text(path: String, body: String) -> Result<i64, i64> {
    fs::write(Path::new(&path), body.as_bytes())
        .map(|_| 0i64)
        .map_err(|_| 0i64)
}

/// CLI arg at index (0 = program name) → Option.
pub fn cli_arg(i: i64) -> Option<String> {
    if i < 0 {
        return None;
    }
    std::env::args().nth(i as usize)
}

/// JSON object string field `key` as i64 (number or numeric string).
pub fn json_get_int(doc: String, key: String) -> Option<i64> {
    let v: serde_json::Value = serde_json::from_str(&doc).ok()?;
    let x = v.get(&key)?;
    match x {
        serde_json::Value::Number(n) => n.as_i64(),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_get_int_number() {
        assert_eq!(json_get_int(r#"{"n":42}"#.into(), "n".into()), Some(42));
    }

    #[test]
    fn read_write_roundtrip() {
        let p = std::env::temp_dir().join("arita-host-core01-rt.txt");
        let ps = p.to_string_lossy().into_owned();
        assert_eq!(write_text(ps.clone(), "hi".into()), Ok(0));
        assert_eq!(read_text(ps), Ok("hi".into()));
        let _ = fs::remove_file(p);
    }
}
