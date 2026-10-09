Translation of `197-bootstrap-43.md`; the original is normative. / Traducción de `197-bootstrap-43.md`; el original es el normativo.

# ADR-197 — bootstrap-43 fib46

- **Estado:** **cerrada** Lex **510/510**
- **CUT-ID:** `BOOTSTRAP-43-20260919`
- Dual-oracle `fib46 → 1836311903` (measure `bootstrap-43` + Rust `GOLDEN_FIB46=1836311903`).
- Baseline Lex **509/509** → **~510**.
- OUT: self-host/timed/recursive; Mutex/`[]`/insert PARK. Parser/Codegen HOLD.
