Translation of `217-bootstrap-53.md`; the original is normative. / Traducción de `217-bootstrap-53.md`; el original es el normativo.

# ADR-217 — bootstrap-53 fib56

- **Estado:** **cerrada** Lex **563/563**
- **CUT-ID:** `BOOTSTRAP-53-20260919`
- Dual-oracle `fib56 → 225851433717` (measure `bootstrap-53` + Rust `GOLDEN_FIB56=225851433717`).
- Baseline Lex **562/562** → **~563**.
- OUT: self-host/timed/recursive; Mutex/`[]`/insert PARK. Parser/Codegen HOLD.
