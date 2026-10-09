Translation of `221-bootstrap-55.md`; the original is normative. / Traducción de `221-bootstrap-55.md`; el original es el normativo.

# ADR-221 — bootstrap-55 fib58

- **Estado:** **cerrada** Lex **572/572**
- **CUT-ID:** `BOOTSTRAP-55-20260919`
- Dual-oracle `fib58 → 591286729879` (measure `bootstrap-55` + Rust `GOLDEN_FIB58=591286729879`).
- Baseline Lex **571/571** → **~572**.
- OUT: self-host/timed/recursive; Mutex/`[]`/insert PARK. Parser/Codegen HOLD.
