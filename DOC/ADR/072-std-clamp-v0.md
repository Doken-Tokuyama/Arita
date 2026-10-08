# ADR-072 — Int `clamp` v0

- **Estado:** **aceptada** + **verified** Lex **162/162**
- **CUT-ID:** `STD-CLAMP-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-071 min/max; ADR-069 abs; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; HIR pre-emit; skip ≠ PASS.

## Contexto

min/max Lex **158/158**. Completar Ord: **`clamp(lo, hi)`** con reject si `lo > hi` (anti panic theater).

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `n.clamp(lo, hi)` | 2× **Int** | **Int** | **Int** | `Ord::clamp` tras gate |

Pins:

1. Arity 2; malo → E0203.
2. Solo Int.
3. Si **lo** y **hi** son lits (o lit-bound) y **lo > hi** → **E0279** `invalid clamp range` (HIR).
4. Semántica Rust: `min(max(n, lo), hi)` cuando lo ≤ hi.
5. Whitelist. Safe-only.

### 2. OUT v0

- non-lit lo>hi detection v0
- float clamp
- Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-clamp-in` | valor dentro → mismo → **accepted** |
| `std-clamp-low` / `std-clamp-high` | recorte → **accepted** (1 o 2 oráculos) |
| `neg-e0279-clamp-range` | `0.clamp(5, 1)` → **E0279** |

## Checklist

- [x] Pins + E0279 + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **162/162** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
