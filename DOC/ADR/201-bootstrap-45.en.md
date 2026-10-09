Translation of `201-bootstrap-45.md`; the original is normative. / Traducción de `201-bootstrap-45.md`; el original es el normativo.

# ADR-201 — bootstrap-45 fib48

- **Estado:** **cerrada** Lex **522/522**
- **CUT-ID:** `BOOTSTRAP-45-20260919`
- Dual-oracle `fib48 → 4807526976` (measure `bootstrap-45` + Rust `GOLDEN_FIB48=4807526976`).
- Baseline Lex **521/521** → **~522**.
- OUT: self-host/timed/recursive; Mutex/`[]`/insert PARK. Parser/Codegen HOLD.
