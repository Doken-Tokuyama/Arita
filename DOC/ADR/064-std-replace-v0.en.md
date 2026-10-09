Translation of `064-std-replace-v0.md`; the original is normative. / Traducción de `064-std-replace-v0.md`; el original es el normativo.

# ADR-064 — String `replace` v0

- **Estado:** **aceptada** + **verified** Lex **140/140**
- **CUT-ID:** `STD-REPLACE-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-063 trim; ADR-059–062; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

trim Lex **137/137**. Std AI-frequent: **`replace` → String** (new owned; todas las non-overlapping, Rust semantics).

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.replace(from, to)` | `from`: lit\|String; `to`: lit\|String | `String` shared | **`String`** | `.replace(&from, &to)` |

Pins:

1. Arity **2**; bad → E0203.
2. No requires `mut`; retorna String nueva.
3. Whitelist + `replace`. Safe-only.
4. Semantics = Rust `str::replace` (todas las non-overlapping).

### 2. OUT v0

- `replacen` / regex / `replace_range`
- in-place
- Mutex

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-replace-hit` | has match → canonical stdout/len → **accepted** |
| `std-replace-miss` | no match → same string content → **accepted** |
| `neg-e0206-replace-vec` (opt.) | Vec.replace → **E0206** |

## Checklist

- [x] Pins + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **140/140** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
