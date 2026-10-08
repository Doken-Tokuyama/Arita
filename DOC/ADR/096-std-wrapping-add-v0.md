# ADR-096 — Int `wrapping_add` v0

**Status:** Accepted (2026-09-18) — Lex measure **238/238**.

**CUT:** `STD-WRAPPING-ADD-20260918`

## Decision

- `a.wrapping_add(b) → Int` (Int×Int, arity 1).
- Emit Rust `.wrapping_add`; MAX+1 → MIN.
- Starts wrapping trilogy (checked / saturating / wrapping).

## OUT / PARK

- wrapping_sub / wrapping_mul (later CUTs); Mutex PARK.

## Oracles

- `std-wrapping-add-normal` / `std-wrapping-add-overflow` / `neg-e0206-wrapping-add-string`
