Translation of `193-bootstrap-41.md`; the original is normative. / Traducción de `193-bootstrap-41.md`; el original es el normativo.

# ADR-193 — bootstrap-41 fib44

- **Estado:** **cerrada** Lex **499/499**
- **CUT-ID:** `BOOTSTRAP-41-20260919`
- Dual-oracle `fib44 → 701408733` (measure `bootstrap-41` + Rust `GOLDEN_FIB44=701408733`).
- Baseline Lex **498/498** → expect **~499**.
- OUT: self-host/timed/recursive; Mutex/`[]`/insert PARK.
- Parser/Codegen HOLD.
