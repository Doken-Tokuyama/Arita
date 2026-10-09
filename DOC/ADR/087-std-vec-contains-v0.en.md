Translation of `087-std-vec-contains-v0.md`; the original is normative. / Traducción de `087-std-vec-contains-v0.md`; el original es el normativo.

# ADR-087 — Vec `contains` → Bool v0

- **Estado:** **aceptada** + **verified** Lex **211/211**
- **CUT-ID:** `STD-VEC-CONTAINS-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-060 String contains; ADR-085/086 Vec get/first; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

first/last Lex **209/209**. Extends String `contains` to **`Vec<T>.contains(x) -> Bool`**.

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `v.contains(x)` | 1× T | Vec shared | **Bool** | `.contains(&x)` |

Pins: arity 1; T = elem; String.contains intact; whitelist; safe-only.

### 2. OUT v0

- contains_key / HashSet; Mutex; `[]` PARK

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-vec-contains-true` / `std-vec-contains-false` | → **accepted** |
| `neg-e0206-contains-int` | Int.contains → **E0206** |

Note: removed `neg-e0206-contains-vec` (ADR-060) — now valid.

## Checklist

- [x] Pins + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **211/211** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
