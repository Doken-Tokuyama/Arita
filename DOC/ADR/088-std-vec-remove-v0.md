# ADR-088 — Vec `remove` → Option v0

- **Estado:** **aceptada** + **verified** Lex **214/214**
- **CUT-ID:** `STD-VEC-REMOVE-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-085 get; ADR-052 pop; ADR-049 mut exclusive; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

Vec.contains Lex **211/211**. Safe index remove: **`remove(i) → Option<T>`** (no panic).

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `v.remove(i)` | 1× Int | Vec **mut exclusive** | **`Option<T>`** | bounds-check then `remove` |

Pins: i<0/OOB → None; else Some + compact; **E0202** sin mut; no bare Rust `remove` (panic); safe-only; `#![forbid(unsafe_code)]`.

### 2. OUT v0

- insert / `[]` / swap_remove / drain / retain; Mutex PARK

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-vec-remove-some` | → **accepted** |
| `std-vec-remove-none` | → **accepted** |
| `neg-e0202-remove-not-mut` | → **E0202** |

## Checklist

- [x] Pins + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **214/214** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
