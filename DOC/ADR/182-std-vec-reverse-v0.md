# ADR-182 — Vec `reverse()` v0

- **Estado:** **aceptada** + **verified** Lex **473/473**
- **CUT-ID:** `STD-VEC-REVERSE-20260919`
- **Fecha:** 2026-09-19
- `mut v.reverse() → ()`; emit `.reverse()`.
- sin mut → E0202; String → E0206 (String.reverse OUT).
- Oracles: reverse, empty, neg-e0202/e0206. Baseline 469→473.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD.
- Mirrors `_mirror_182.tgz`.
- Closed: 2026-09-19T08:17:25Z (2026-09-19 Europe/Madrid)
