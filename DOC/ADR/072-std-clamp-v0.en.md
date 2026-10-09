Translation of `072-std-clamp-v0.md`; the original is normative. / Traducción de `072-std-clamp-v0.md`; el original es el normativo.

# ADR-072 — Int `clamp` v0

- **Estado:** **aceptada** + **verified** Lex **162/162**
- **CUT-ID:** `STD-CLAMP-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-071 min/max; ADR-069 abs; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; HIR pre-emit; skip ≠ PASS.

## Context

min/max Lex **158/158**. Complete Ord: **`clamp(lo, hi)`** with reject if `lo > hi` (anti panic theater).

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `n.clamp(lo, hi)` | 2× **Int** | **Int** | **Int** | `Ord::clamp` after gate |

Pins:

1. Arity 2; bad → E0203.
2. Solo Int.
3. Si **lo** y **hi** son lits (o lit-bound) y **lo > hi** → **E0279** `invalid clamp range` (HIR).
4. Rust semantics: `min(max(n, lo), hi)` when lo ≤ hi.
5. Whitelist. Safe-only.

### 2. OUT v0

- non-lit lo>hi detection v0
- float clamp
- Mutex

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-clamp-in` | in-range value → same → **accepted** |
| `std-clamp-low` / `std-clamp-high` | recorte → **accepted** (1 o 2 oracles) |
| `neg-e0279-clamp-range` | `0.clamp(5, 1)` → **E0279** |

## Checklist

- [x] Pins + E0279 + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **162/162** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
