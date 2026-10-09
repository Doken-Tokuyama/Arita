Translation of `166-std-vec-fill-v0.md`; the original is normative. / Traducción de `166-std-vec-fill-v0.md`; el original es el normativo.

# ADR-166 — Vec `fill(x)` v0

- **Estado:** **aceptada** + **verified** Lex **434/434**
- **CUT-ID:** `STD-VEC-FILL-20260919`
- **Fecha:** 2026-09-19
- `mut v.fill(x) → ()`; emit `.fill(x)`; exclusive mut.
- missing mut → E0202; non-Vec (String/Int) → E0206.
- Oracles: fill overwrite, empty no-op, neg-e0202/e0206. Baseline 430→434.
- OUT: fill_with; Mutex/`[]` PARK. Mirrors `_mirror_166.tgz`.
- Closed: 2026-09-19T06:46:21Z (2026-09-19 Europe/Madrid)
