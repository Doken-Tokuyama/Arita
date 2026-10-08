# ADR-082 — Int `checked_mul` → Option v0

- **Estado:** **aceptada** + **verified** Lex **193/193**
- **CUT-ID:** `STD-CHECKED-MUL-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-080/081 checked_add/sub; ADR-045 E0217; ADR-050 Option; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

checked_sub Lex **190/190**. Familia: **`checked_mul` → Option<Int>**.

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `a.checked_mul(b)` | 1× Int | Int | **`Option<Int>`** | `checked_mul` |

Pins: espejo 080/081.

### 2. OUT v0

- checked_div → **ADR-083**; rem/saturating posteriores; Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-checked-mul-some` | `6.checked_mul(7)` → Some(42) → **accepted** |
| `std-checked-mul-none` | `MAX.checked_mul(2)` → None → **accepted** |
| `neg-e0206-checked-mul-string` (opc.) | → **E0206** |

## Checklist

- [x] Pins + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **193/193** (STABLE_VERIFY)

## Cola

Mutex **PARK**. checked_div = posterior.
