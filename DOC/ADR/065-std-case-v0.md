# ADR-065 — String `to_uppercase` / `to_lowercase` v0

- **Estado:** **aceptada** + **verified** Lex **143/143**
- **CUT-ID:** `STD-CASE-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-063 trim; ADR-064 replace; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

replace Lex **140/140**. Par std IA: case fold owned.

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.to_uppercase()` | 0 | `String` shared | **`String`** | `.to_uppercase()` |
| `s.to_lowercase()` | 0 | `String` shared | **`String`** | `.to_lowercase()` |

Pins: arity 0; no mut; owned nuevo; whitelist ambos; Unicode = Rust std; safe-only.

### 2. OUT v0

- locale-specific case
- in-place ASCII-only API distinta
- Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-to-uppercase` | entrada conocida → stdout upper canónico → **accepted** |
| `std-to-lowercase` | idem lower → **accepted** |
| `neg-e0206-case-vec` (opc.) | Vec.to_uppercase → **E0206** |

## Checklist

- [x] Pins + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **143/143** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
