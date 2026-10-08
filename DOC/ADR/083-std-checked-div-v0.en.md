Translation of `083-std-checked-div-v0.md`; the original is normative. / Traducción de `083-std-checked-div-v0.md`; el original es el normativo.

# ADR-083 — Int `checked_div` → Option v0

- **Estado:** **aceptada** + **verified** Lex **197/197**
- **CUT-ID:** `STD-CHECKED-DIV-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-080–082 checked_*; ADR-044 E0216 div0; ADR-050 Option; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

checked_mul Lex **193/193**. Family: **`checked_div` → Option**. Complements E0216 (`/` lit0 reject): Option API for runtime.

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `a.checked_div(b)` | 1× Int | Int | **`Option<Int>`** | `checked_div` |

Pins:

1. Arity 1; Int only.
2. **None** si `b == 0` **o** overflow (`MIN / -1`).
3. Some(quot) otherwise (Rust trunc toward zero).
4. Whitelist. Safe-only. E0216 sigue para operador `/` lit0.

### 2. OUT v0

- checked_rem → **ADR-084**; later saturating; Mutex

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-checked-div-some` | `7.checked_div(2)` → Some(3) → **accepted** |
| `std-checked-div-none-zero` | `1.checked_div(0)` → None → **accepted** |
| `std-checked-div-none-overflow` (opt.) | `MIN.checked_div(-1)` → None → **accepted** |
| `neg-e0206-checked-div-string` (opt.) | → **E0206** |

## Checklist

- [x] Pins + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **197/197** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
