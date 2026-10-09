Translation of `073-std-trim-ends-v0.md`; the original is normative. / Traducción de `073-std-trim-ends-v0.md`; el original es el normativo.

# ADR-073 — String `trim_start` / `trim_end` v0

- **Estado:** **aceptada** + **verified** Lex **165/165**
- **CUT-ID:** `STD-TRIM-ENDS-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-063 trim (OUT trim_start/end → este); ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

clamp Lex **162/162**. Complete the trim family: **start/end** (owned String; shared).

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.trim_start()` | 0 | String shared | **String** | `.trim_start().to_string()` |
| `s.trim_end()` | 0 | String shared | **String** | `.trim_end().to_string()` |

Pins: mirror ADR-063; Rust Unicode ws; whitelist both.

### 2. OUT v0

- trim_matches; in-place; Mutex

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-trim-start` | canonical leading ws stripped → **accepted** |
| `std-trim-end` | trailing ws stripped → **accepted** |
| `neg-e0206-trim-start-vec` (opt.) | Vec → **E0206** |

## Checklist

- [x] Pins + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **165/165** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
