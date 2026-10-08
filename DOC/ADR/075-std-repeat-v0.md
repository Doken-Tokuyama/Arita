# ADR-075 — String `repeat` v0

- **Estado:** **aceptada** + **verified** Lex **172/172**
- **CUT-ID:** `STD-REPEAT-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-026; ADR-045 overflow familia; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; HIR pre-emit; skip ≠ PASS.

## Contexto

strip Lex **169/169**. Std: **`repeat(n) -> String`**. Reject n<0 (Rust usize cast theater).

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.repeat(n)` | **Int** | String shared | **String** | `.repeat(n as usize)` tras gate |

Pins:

1. Arity 1; malo → E0203.
2. Si **n** lit (o lit-bound) **< 0** → **E0280** `negative repeat count`.
3. n ≥ 0: emit Rust `repeat` (usize).
4. Whitelist. Safe-only.
5. Huge n OOM: OUT v0 (no timed/capacity theater pin).

### 2. OUT v0

- non-lit negative detection v0
- Vec.repeat → **ADR-076**
- Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-repeat` | `"ab".repeat(3)` → `ababab` → **accepted** |
| `std-repeat-zero` | repeat(0) → empty → **accepted** |
| `neg-e0280-repeat-neg` | repeat(-1) → **E0280** |

## Checklist

- [x] Pins + E0280 + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **172/172** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
