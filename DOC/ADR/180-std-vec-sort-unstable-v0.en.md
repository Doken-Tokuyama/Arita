Translation of `180-std-vec-sort-unstable-v0.md`; the original is normative. / Traducción de `180-std-vec-sort-unstable-v0.md`; el original es el normativo.

# ADR-180 — Vec `sort_unstable()` v0

- **Estado:** **aceptada** + **verified** Lex **468/468**
- **CUT-ID:** `STD-VEC-SORT-UNSTABLE-20260919`
- **Fecha:** 2026-09-19
- `mut v.sort_unstable() → ()`; emit `.sort_unstable()`; Ord elems (`Vec<Int>`).
- missing mut → E0202; wrong type → E0206.
- Oracles: int sort, reverse sort, neg-e0202/e0206.
- Note: pin “string sort” deferred — F1.1 `ty_vec_int` only (`Vec<String>` needs Parser GO).
- Baseline 464→468. OUT: sort stable / sort_by*; Mutex/`[]` PARK.
- Mirrors `_mirror_180.tgz`.
- Closed: 2026-09-19T08:07:49Z (2026-09-19 Europe/Madrid)
