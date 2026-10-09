Translation of `063-std-trim-v0.md`; the original is normative. / Traducción de `063-std-trim-v0.md`; el original es el normativo.

# ADR-063 — String `trim` v0

- **Estado:** **aceptada** + **verified** Lex **137/137**
- **CUT-ID:** `STD-TRIM-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-026; ADR-060–062 query trio; ADR-058 clone; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

contains/starts_with/ends_with trio Lex **134/134**. Next std AI-useful: **`trim() -> String`** (nuevo string; shared borrow).

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.trim()` | 0 | `String` shared | **`String`** | `.trim().to_string()` (o equiv. owned) |

Pins:

1. Arity 0; args → E0203.
2. No requires `mut`.
3. Retorna **String nueva** (original intact).
4. Whitelist + `trim`. Safe-only.
5. Unicode-aware = Rust `str::trim` (ws Unicode).

### 2. OUT v0

- `trim_start` / `trim_end` → **ADR-073**; `trim_matches` posterior
- in-place trim
- Mutex

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-trim-spaces` | `"  hi  ".trim()` → print `hi` / canonical len → **accepted** |
| `std-trim-noop` | no edge ws → same content → **accepted** |
| `neg-e0206-trim-vec` (opt.) | Vec.trim → **E0206** |

## Checklist

- [x] Pins + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **137/137** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
