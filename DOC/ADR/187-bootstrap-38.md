# ADR-187 — Bootstrap-38

- **Estado:** **aceptada** + **verified** Lex **484/484**
- **CUT-ID:** `BOOTSTRAP-38-20260919`
- **Fecha:** 2026-09-19
- Dual-oracle `fib41 → 165580141` (measure `bootstrap-38` + Rust `GOLDEN_FIB41=165580141`).
- Baseline Lex **483/483** → **484/484**.
- OUT: self-host/timed/recursive; Mutex/`[]`/insert PARK. Parser/Codegen HOLD.
- Mirrors `_mirror_187.tgz`.
- Closed: 2026-09-19T08:41:42Z (2026-09-19 Europe/Madrid)
