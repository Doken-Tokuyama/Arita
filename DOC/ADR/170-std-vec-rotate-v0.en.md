Translation of `170-std-vec-rotate-v0.md`; the original is normative. / Traducción de `170-std-vec-rotate-v0.md`; el original es el normativo.

# ADR-170 — Vec `rotate_left` / `rotate_right` v0

- **Estado:** **aceptada** + **verified** Lex **445/445**
- **CUT-ID:** `STD-VEC-ROTATE-20260919`
- **Fecha:** 2026-09-19
- `mut v.rotate_left(n)` / `rotate_right(n)` → (); emit `% len` (len0/neg runtime → no-op).
- lit n < 0 → E0297; missing mut → E0202. Int bit-rotate (E0291) unchanged.
- Oracles: left/right + neg-e0297/e0202. Baseline 441→445.
- OUT: Mutex/`[]` PARK. Result.take VOID. Mirrors `_mirror_170.tgz`.
- Closed: 2026-09-19T07:14:21Z (2026-09-19 Europe/Madrid)
