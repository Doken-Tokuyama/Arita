Translation of `065-std-case-v0.md`; the original is normative. / Traducción de `065-std-case-v0.md`; el original es el normativo.

# ADR-065 — String `to_uppercase` / `to_lowercase` v0

- **Estado:** **aceptada** + **verified** Lex **143/143**
- **CUT-ID:** `STD-CASE-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-063 trim; ADR-064 replace; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

replace Lex **140/140**. Par std IA: case fold owned.

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.to_uppercase()` | 0 | `String` shared | **`String`** | `.to_uppercase()` |
| `s.to_lowercase()` | 0 | `String` shared | **`String`** | `.to_lowercase()` |

Pins: arity 0; no mut; new owned; whitelist both; Unicode = Rust std; safe-only.

### 2. OUT v0

- locale-specific case
- in-place ASCII-only API distinta
- Mutex

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-to-uppercase` | known input → canonical upper stdout → **accepted** |
| `std-to-lowercase` | idem lower → **accepted** |
| `neg-e0206-case-vec` (opt.) | Vec.to_uppercase → **E0206** |

## Checklist

- [x] Pins + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **143/143** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
