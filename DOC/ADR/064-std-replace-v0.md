# ADR-064 — String `replace` v0

- **Estado:** **aceptada** + **verified** Lex **140/140**
- **CUT-ID:** `STD-REPLACE-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-063 trim; ADR-059–062; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

trim Lex **137/137**. Std IA-frecuente: **`replace` → String** (owned nuevo; todas las no-solapadas, semántica Rust).

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.replace(from, to)` | `from`: lit\|String; `to`: lit\|String | `String` shared | **`String`** | `.replace(&from, &to)` |

Pins:

1. Arity **2**; malo → E0203.
2. No exige `mut`; retorna String nueva.
3. Whitelist + `replace`. Safe-only.
4. Semántica = Rust `str::replace` (todas las no-solapadas).

### 2. OUT v0

- `replacen` / regex / `replace_range`
- in-place
- Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-replace-hit` | hay match → stdout/len canónico → **accepted** |
| `std-replace-miss` | sin match → string igual en contenido → **accepted** |
| `neg-e0206-replace-vec` (opc.) | Vec.replace → **E0206** |

## Checklist

- [x] Pins + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **140/140** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
