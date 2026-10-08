# ADR-084 — Int `checked_rem` → Option v0

- **Estado:** **aceptada** + **verified** Lex **201/201**
- **CUT-ID:** `STD-CHECKED-REM-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-083 checked_div; ADR-044 E0216 (%0); ADR-050 Option; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

checked_div Lex **197/197**. Cierra checked arith v0: **`checked_rem` → Option**.

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `a.checked_rem(b)` | 1× Int | Int | **`Option<Int>`** | `checked_rem` |

Pins: espejo div — None si `b==0` o overflow `MIN % -1`; Some(resto) else; trunc rem Rust.

### 2. OUT v0

- saturating/wrapping; Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-checked-rem-some` | `7.checked_rem(3)` → Some(1) → **accepted** |
| `std-checked-rem-none-zero` | `1.checked_rem(0)` → None → **accepted** |
| `std-checked-rem-none-overflow` (opc.) | `MIN.checked_rem(-1)` → None → **accepted** |
| `neg-e0206-checked-rem-string` (opc.) | → **E0206** |

## Checklist

- [x] Pins + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **201/201** (STABLE_VERIFY); familia checked v0 cerrada

## Cola

Mutex **PARK**. Familia checked +−*/% v0 completa tras este CUT.
