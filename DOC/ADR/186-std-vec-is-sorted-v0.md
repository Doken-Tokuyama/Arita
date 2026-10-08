# ADR-186 — Vec `is_sorted()` v0

- **Estado:** **aceptada** + **verified** Lex **483/483**
- **CUT-ID:** `STD-VEC-IS-SORTED-20260919`
- **Fecha:** 2026-09-19
- `v.is_sorted() → Bool`; emit `.is_sorted()`; shared arity 0; `Vec<Int>` F1.1.
- Wrong type → E0206.
- Oracles: true/false/empty + neg-e0206. Baseline 479→483.
- OUT: is_sorted_by*; Mutex/`[]` PARK. Parser/Codegen HOLD.
- Mirrors `_mirror_186.tgz`.
- Closed: 2026-09-19T08:37:22Z (2026-09-19 Europe/Madrid)
