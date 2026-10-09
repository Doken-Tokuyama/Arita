Translation of `195-bootstrap-42.md`; the original is normative. / Traducción de `195-bootstrap-42.md`; el original es el normativo.

# ADR-195 — bootstrap-42 fib45

- **Estado:** **cerrada** Lex **505/505**
- **CUT-ID:** `BOOTSTRAP-42-20260919`
- Dual-oracle `fib45 → 1134903170` (measure `bootstrap-42` + Rust `GOLDEN_FIB45=1134903170`).
- Baseline Lex **504/504** → **~505**.
- OUT: self-host/timed/recursive; Mutex/`[]`/insert PARK. Parser/Codegen HOLD.
