Translation of `067-std-int-to-string-v0.md`; the original is normative. / Traducción de `067-std-int-to-string-v0.md`; el original es el normativo.

# ADR-067 — Int `to_string` v0

- **Estado:** **aceptada** + **verified** Lex **149/149**
- **CUT-ID:** `STD-INT-TO-STRING-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-066 parse_int (espejo); ADR-026; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

parse_int Lex **146/146**. Mirror: **Int → String** decimal.

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `n.to_string()` | 0 | **`Int`** | **`String`** | `n.to_string()` / format |

Pins:

1. Arity 0; bad → E0203.
2. Solo receiver **Int** v0 (Bool/Option OUT → E0206).
3. Decimal signed ASCII (Rust i64 Display).
4. Method whitelist on Int (extends std; not a String method).
5. Measurable round-trip: `n.to_string().parse_int()` → Ok(n) in pos oracle.

### 2. OUT v0

- `to_string` en Bool → **ADR-070**; String/Vec siguen OUT
- hex/bin format
- Mutex

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-int-to-string` | print/assert `"42"` etc. → **accepted** |
| `std-int-to-string-roundtrip` | to_string+parse_int Ok same → **accepted** |
| `neg-e0206-to-string-bool` (opt.) | `true.to_string()` → **E0206** |

## Checklist

- [x] Pins + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **149/149** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
