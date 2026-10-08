# ADR-176 — Vec `dedup()` v0

- **Estado:** **aceptada** + **verified** Lex **459/459**
- **CUT-ID:** `STD-VEC-DEDUP-20260919`
- **Fecha:** 2026-09-19
- `mut v.dedup() → ()`; emit `.dedup()`; consecutive only.
- sin mut → E0202; String → E0206. Oracles: runs, nonconsec, negs. Baseline 455→459.
- OUT: dedup_by*; Mutex/`[]` PARK. Mirrors `_mirror_176.tgz`.
- Closed: 2026-09-19T07:46:49Z (2026-09-19 Europe/Madrid)
