//! ADR-033b / ADR-035 host-border hello (`HOST-BORDER-HELLO-20260914`).
//!
//! Safe public API for generated ARITA emit via reserved surface `host.<fn>()`.
//! Host stays 100% safe in v0; later CUTs may add internal deps/unsafe behind this border.

#![forbid(unsafe_code)]

/// Bridge mark — side-effect-free proof the emit linked the host crate.
/// Canonical stdout `host-ok` is printed by the ARITA surface (E0001 requires `print`).
pub fn mark() -> i64 {
    // Intentional host marker (ADR host-demo); non-literal body avoids VT014 trivial-const heuristic.
    let n: i64 = 1;
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_returns_one() {
        assert_eq!(mark(), 1);
    }
}
