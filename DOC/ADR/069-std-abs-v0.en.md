Translation of `069-std-abs-v0.md`; the original is normative. / Traducción de `069-std-abs-v0.md`; el original es el normativo.

# ADR-069 — Int `abs` v0

- **Estado:** **aceptada** + **verified** Lex **153/153**
- **CUT-ID:** `STD-ABS-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-045 E0217 overflow; ADR-067 to_string; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; HIR reject pre-emit; skip ≠ PASS.

## Context

Bootstrap-04 Lex **150/150**. Std Int useful: **`abs`**, with anti-theater overflow pin on `i64::MIN`.

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `n.abs()` | 0 | **`Int`** | **`Int`** | checked/HIR-safe abs |

Pins:

1. Arity 0; bad → E0203.
2. Solo **Int**; otro → E0206.
3. **Overflow:** si scrutinee es lit **`i64::MIN`** (−9223372036854775808) → **E0278** `integer abs overflow` (HIR, before emit). No rustc panic theater.
4. Other lits / runtime: emit `n.abs()` o `checked_abs`+unreachable forbid — prefer **`.abs()`** after gate lit MIN.
5. Whitelist. Safe-only.

### 2. OUT v0

- saturating_abs / wrapping_abs surface
- abs non-lit MIN detection (OUT v0; solo lit)
- Mutex

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-abs-pos` | `5.abs()` → 5 → **accepted** |
| `std-abs-neg` | `(-3).abs()` → 3 → **accepted** |
| `neg-e0278-abs-min` | `(-9223372036854775808).abs()` → **E0278** |

## Checklist

- [x] Pins + E0278 + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **153/153** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
