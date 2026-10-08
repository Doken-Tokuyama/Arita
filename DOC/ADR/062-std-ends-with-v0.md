# ADR-062 — String `ends_with` v0

- **Estado:** **aceptada** + **verified** Lex **134/134**
- **CUT-ID:** `STD-ENDS-WITH-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-060 contains; ADR-061 starts_with (espejo); ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

starts_with Lex **131/131**. Completar trío prefix/suffix/contains con **`ends_with`→Bool**.

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.ends_with(arg)` | `String` **o** lit str | `String` shared | **`Bool`** | `.ends_with(&…)` |

Pins: espejo ADR-061 (arity 1; no mut; whitelist; safe-only).

### 2. OUT v0

- regex / find / strip
- Vec
- Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-ends-with-true` | sufijo presente → true → **accepted** |
| `std-ends-with-false` | ausente → false → **accepted** |
| `neg-e0206-ends-with-vec` (opc.) | Vec → **E0206** |

## Checklist

- [x] Pins + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **134/134** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
