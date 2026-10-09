Translation of `071-std-min-max-v0.md`; the original is normative. / Traducción de `071-std-min-max-v0.md`; el original es el normativo.

# ADR-071 — Int `min` / `max` v0

- **Estado:** **aceptada** + **verified** Lex **158/158**
- **CUT-ID:** `STD-MIN-MAX-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-069 abs; ADR-023 cmp; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

Bool.to_string Lex **155/155**. Std Int: **`min`/`max`** (Ord), measurable without Mutex.

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `a.min(b)` | 1× **Int** | **Int** | **Int** | `Ord::min` / `a.min(b)` |
| `a.max(b)` | 1× **Int** | **Int** | **Int** | idem max |

Pins: arity 1; both Int; whitelist; safe-only; Rust semantics (no NaN).

### 2. OUT v0

- min/max en Bool/String
- clamp → **ADR-072**
- Mutex

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-min` | e.g. `3.min(5)` → 3 → **accepted** |
| `std-max` | `3.max(5)` → 5 → **accepted** |
| `neg-e0206-min-string` (opt.) | String.min → **E0206** |

## Checklist

- [x] Pins + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **158/158** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
