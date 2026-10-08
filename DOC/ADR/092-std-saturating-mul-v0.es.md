Translation of `092-std-saturating-mul-v0.md`; the original is normative. / Traducción de `092-std-saturating-mul-v0.md`; el original es el normativo.

# ADR-092 — Int `saturating_mul` v0

**Status:** Accepted (2026-09-18) — Lex measure **224/224**.

**CUT:** `STD-SATURATING-MUL-20260918`

## Decisión

- `a.saturating_mul(b) → Int` (Int×Int, arity 1).
- Emit Rust `.saturating_mul`; overflow → MAX/MIN.
- Cierra saturating +−× v0 (con ADR-090/091).

## OUT / PARK

- saturating_div (no std i64); wrapping_*; Mutex PARK.

## Oráculos

- `std-saturating-mul-normal` / `std-saturating-mul-max` / `neg-e0206-saturating-mul-string`
