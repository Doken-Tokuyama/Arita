# ADR-090 — Int `saturating_add` v0

- **Estado:** **aceptada** + **verified** Lex **218/218**
- **CUT-ID:** `STD-SATURATING-ADD-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-080 checked_add; ADR-045 E0217; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

Bootstrap-05 Lex **215/215**. Familia saturating: **`saturating_add` → Int** (no Option; clamp a MAX/MIN).

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `a.saturating_add(b)` | 1× Int | Int | **Int** | `.saturating_add` |

### 2. OUT v0

- saturating_sub/mul (CUTs después); wrapping_*; Mutex PARK

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-saturating-add-normal` | 1+2 → 3 |
| `std-saturating-add-max` | MAX+1 → MAX |
| `neg-e0206-saturating-add-string` | → **E0206** |

## Checklist

- [x] Pins + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **218/218** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
