# ADR-076 — Vec `repeat` v0

- **Estado:** **aceptada** + **verified** Lex **175/175**
- **CUT-ID:** `STD-VEC-REPEAT-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-075 String repeat (E0280); ADR-052 pop; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; HIR pre-emit; skip ≠ PASS.

## Contexto

String.repeat Lex **172/172**. Espejo: **`Vec.repeat(n) -> Vec<T>`** + mismo **E0280**.

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `v.repeat(n)` | **Int** | `Vec<T>` shared | **`Vec<T>`** | `.repeat(n as usize)` tras gate |

Pins: espejo ADR-075; lit n<0 → **E0280**; T ∈ tipos Vec F2; whitelist; safe-only.

### 2. OUT v0

- non-lit neg; Mutex; huge OOM pin

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-vec-repeat` | `[1].repeat(3)` equiv → len/print canónico → **accepted** |
| `std-vec-repeat-zero` | repeat(0) empty → **accepted** |
| `neg-e0280-vec-repeat-neg` | repeat(-1) → **E0280** |

## Checklist

- [x] Pins + E0280 reuse + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **175/175** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
