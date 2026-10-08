# ADR-092 — Int `saturating_mul` v0

**Status:** Accepted (2026-09-18) — Lex measure **224/224**.

**CUT:** `STD-SATURATING-MUL-20260918`

## Decision

- `a.saturating_mul(b) → Int` (Int×Int, arity 1).
- Emit Rust `.saturating_mul`; overflow → MAX/MIN.
- Closes saturating +−× v0 (with ADR-090/091).

## OUT / PARK

- saturating_div (no std i64); wrapping_*; Mutex PARK.

## Oracles

- `std-saturating-mul-normal` / `std-saturating-mul-max` / `neg-e0206-saturating-mul-string`
