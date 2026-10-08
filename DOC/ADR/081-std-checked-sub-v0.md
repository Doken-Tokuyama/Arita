# ADR-081 — Int `checked_sub` → Option v0

- **Estado:** **aceptada** + **verified** Lex **190/190**
- **CUT-ID:** `STD-CHECKED-SUB-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-080 checked_add (espejo); ADR-045 E0217; ADR-050 Option; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

checked_add Lex **187/187**. Siguiente de familia: **`checked_sub` → Option<Int>**.

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `a.checked_sub(b)` | 1× Int | Int | **`Option<Int>`** | `checked_sub` |

Pins: espejo ADR-080 (arity 1; Int; whitelist; safe-only).

### 2. OUT v0

- checked_mul → **ADR-082**; div/saturating posteriores; Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-checked-sub-some` | p.ej. `5.checked_sub(3)` → Some(2) → **accepted** |
| `std-checked-sub-none` | `MIN.checked_sub(1)` → None → **accepted** |
| `neg-e0206-checked-sub-string` (opc.) | → **E0206** |

## Checklist

- [x] Pins + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **190/190** (STABLE_VERIFY)

## Cola

Mutex **PARK**. checked_mul = posterior.
