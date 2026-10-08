# ADR-070 — Bool `to_string` v0

- **Estado:** **aceptada** + **verified** Lex **155/155**
- **CUT-ID:** `STD-BOOL-TO-STRING-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-067 Int to_string (Bool era OUT/E0206 → este CUT); ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

abs Lex **153/153**. ADR-067 negó Bool.to_string (E0206). Abrir **Bool → String** canónico.

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `b.to_string()` | 0 | **`Bool`** | **`String`** | `b.to_string()` → `"true"` / `"false"` |

Pins: arity 0; Display bool Rust; whitelist Bool; Int sigue 067; String/Vec.to_string OUT (E0206).

### 2. OUT v0

- custom true/false spellings
- Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-bool-to-string-true` | `true.to_string()` → print `true` → **accepted** |
| `std-bool-to-string-false` | `false.to_string()` → `false` → **accepted** |
| ajustar `neg-e0206-to-string-bool` | **retirar o invertir**: ya no E0206; neg pasa a Vec/String `.to_string()` si aplica |

## Checklist

- [x] Pins + oráculos + nota neg 067
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **155/155** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
