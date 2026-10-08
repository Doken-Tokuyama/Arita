Translation of `091-std-saturating-sub-v0.md`; the original is normative. / Traducción de `091-std-saturating-sub-v0.md`; el original es el normativo.

# ADR-091 — Int `saturating_sub` v0

- **Estado:** **aceptada** + **verified** Lex **221/221**
- **CUT-ID:** `STD-SATURATING-SUB-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-090 saturating_add; ADR-081 checked_sub; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

saturating_add Lex **218/218**. Mirror: **`saturating_sub` → Int** (MIN-1 → MIN).

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `a.saturating_sub(b)` | 1× Int | Int | **Int** | `.saturating_sub` |

### 2. OUT v0

- saturating_mul (later CUT); wrapping_*; Mutex PARK

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-saturating-sub-normal` | 5-3 → 2 |
| `std-saturating-sub-min` | MIN-1 → MIN |
| `neg-e0206-saturating-sub-string` | → **E0206** |

## Checklist

- [x] Pins + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **221/221** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
