# ADR-098 — Int `wrapping_mul` v0

**Status:** Accepted (2026-09-18) — Lex measure **244/244**.

**CUT:** `STD-WRAPPING-MUL-20260918`

## Decision

- `a.wrapping_mul(b) → Int` (Int×Int, arity 1).
- Emit Rust `.wrapping_mul`; overflow wraps (MAX×2 → -2).
- Closes wrapping +−× v0 (with ADR-096/097).

## OUT / PARK

- wrapping_div / wrapping_rem; Mutex PARK.

## Oracles

- `std-wrapping-mul-normal` / `std-wrapping-mul-overflow` / `neg-e0206-wrapping-mul-string`
