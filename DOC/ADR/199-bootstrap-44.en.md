Translation of `199-bootstrap-44.md`; the original is normative. / Traducción de `199-bootstrap-44.md`; el original es el normativo.

# ADR-199 — bootstrap-44 fib47

- **Estado:** **cerrada** Lex **515/515**
- **CUT-ID:** `BOOTSTRAP-44-20260919`
- Dual-oracle `fib47 → 2971215073` (measure `bootstrap-44` + Rust `GOLDEN_FIB47=2971215073`).
- Baseline Lex **514/514** → **~515**.
- OUT: self-host/timed/recursive; Mutex/`[]`/insert PARK. Parser/Codegen HOLD.
