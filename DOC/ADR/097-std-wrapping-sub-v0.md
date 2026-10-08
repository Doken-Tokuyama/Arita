# ADR-097 — Int `wrapping_sub` v0

**Status:** Accepted (2026-09-18) — Lex measure **241/241**.

**CUT:** `STD-WRAPPING-SUB-20260918`

## Decision

- `a.wrapping_sub(b) → Int` (Int×Int, arity 1).
- Emit Rust `.wrapping_sub`; MIN-1 → MAX.

## OUT / PARK

- wrapping_mul (later CUT); Mutex PARK.

## Oracles

- `std-wrapping-sub-normal` / `std-wrapping-sub-underflow` / `neg-e0206-wrapping-sub-string`
