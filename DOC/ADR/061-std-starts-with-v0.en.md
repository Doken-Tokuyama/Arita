Translation of `061-std-starts-with-v0.md`; the original is normative. / Traducción de `061-std-starts-with-v0.md`; el original es el normativo.

# ADR-061 — String `starts_with` v0

- **Estado:** **aceptada** + **verified** Lex **131/131**
- **CUT-ID:** `STD-STARTS-WITH-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-060 contains (espejo); ADR-026; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

contains Lex **128/128**. OUT deferred `starts_with` — same shape, Bool return.

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.starts_with(arg)` | `String` **o** lit str | `String` shared | **`Bool`** | `.starts_with(&…)` |

Pins: arity 1; no mut; whitelist; safe-only. Mirror ADR-060.

### 2. OUT v0

- `ends_with` (CUT next si aplica)
- regex / Vec
- Mutex

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-starts-with-true` | prefix present → canonical true → **accepted** |
| `std-starts-with-false` | absent → false → **accepted** |
| `neg-e0206-starts-with-vec` (opt.) | en Vec → **E0206** |

## Checklist

- [x] Pins + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **131/131** (STABLE_VERIFY)

## Queue

Mutex **PARK**. `ends_with` → **ADR-062**.
