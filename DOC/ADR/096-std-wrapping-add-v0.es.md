Translation of `096-std-wrapping-add-v0.md`; the original is normative. / Traducción de `096-std-wrapping-add-v0.md`; el original es el normativo.

# ADR-096 — Int `wrapping_add` v0

**Status:** Accepted (2026-09-18) — Lex measure **238/238**.

**CUT:** `STD-WRAPPING-ADD-20260918`

## Decisión

- `a.wrapping_add(b) → Int` (Int×Int, arity 1).
- Emit Rust `.wrapping_add`; MAX+1 → MIN.
- Abre la trilogía wrapping (checked / saturating / wrapping).

## OUT / PARK

- wrapping_sub / wrapping_mul (CUTs posteriores); Mutex PARK.

## Oráculos

- `std-wrapping-add-normal` / `std-wrapping-add-overflow` / `neg-e0206-wrapping-add-string`
