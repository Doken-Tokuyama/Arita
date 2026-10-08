Translation of `098-std-wrapping-mul-v0.md`; the original is normative. / Traducción de `098-std-wrapping-mul-v0.md`; el original es el normativo.

# ADR-098 — Int `wrapping_mul` v0

**Status:** Accepted (2026-09-18) — Lex measure **244/244**.

**CUT:** `STD-WRAPPING-MUL-20260918`

## Decisión

- `a.wrapping_mul(b) → Int` (Int×Int, arity 1).
- Emit Rust `.wrapping_mul`; overflow hace wrap (MAX×2 → -2).
- Cierra wrapping +−× v0 (con ADR-096/097).

## OUT / PARK

- wrapping_div / wrapping_rem; Mutex PARK.

## Oráculos

- `std-wrapping-mul-normal` / `std-wrapping-mul-overflow` / `neg-e0206-wrapping-mul-string`
