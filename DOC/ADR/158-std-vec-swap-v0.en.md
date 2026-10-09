Translation of `158-std-vec-swap-v0.md`; the original is normative. / Traducción de `158-std-vec-swap-v0.md`; el original es el normativo.

# ADR-158 — Vec `swap(i, j)` v0

- **Estado:** **aceptada** + **verified** Lex **413/413**
- **CUT-ID:** `STD-VEC-SWAP-20260919`
- **Fecha:** 2026-09-19
- Emit bounds-checked; OOB/non-lit neg → no-op; lit i|j < 0 → E0295.
- Mirrors `_mirror_158.tgz`.
