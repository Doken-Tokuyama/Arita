Translation of `086-std-vec-first-last-v0.md`; the original is normative. / Traducción de `086-std-vec-first-last-v0.md`; el original es el normativo.

# ADR-086 — Vec `first` / `last` → Option v0

- **Estado:** **aceptada** + **verified** Lex **209/209**
- **CUT-ID:** `STD-VEC-FIRST-LAST-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-085 get; ADR-050 Option; ADR-052 pop; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

Vec.get Lex **205/205**. Safe sugar: **`first`/`last` → Option<T>** (no `[]`).

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `v.first()` | 0 | Vec shared | **`Option<T>`** | `.first().cloned()` |
| `v.last()` | 0 | Vec shared | **`Option<T>`** | `.last().cloned()` |

Pins: arity 0; empty → None; whitelist; `[]` remains PARK; safe-only.

### 2. OUT v0

- first_mut/last_mut; Mutex; insert

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-vec-first-some` / `std-vec-first-none` | → **accepted** |
| `std-vec-last-some` (or none) | → **accepted** |
| `neg-e0206-first-string` (opt.) | → **E0206** |

## Checklist

- [x] Pins + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **209/209** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
