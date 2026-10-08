# ADR-079 — Int `pow` v0

- **Estado:** **aceptada** + **verified** Lex **184/184**
- **CUT-ID:** `STD-POW-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-069 abs; ADR-045 E0217; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; HIR pre-emit; skip ≠ PASS.

## Contexto

E0282 Lex **180/180**. Std Int: **`pow(exp)`** con reject exp negativo (y overflow lit si foldable).

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `base.pow(exp)` | **Int** exp | **Int** | **Int** | `checked_pow` / gate+`pow` |

Pins:

1. Arity 1; malo → E0203.
2. Solo Int×Int.
3. Si **exp** lit **< 0** → **E0281** `negative pow exponent`.
4. Overflow: si base/exp lits y `checked_pow` None → **E0217** (reuso overflow familia) **o** **E0283** `integer pow overflow` si E0217 queda semánticamente +/-/* only — **pin:** usar **E0283** para no diluir E0217.
5. Whitelist. Safe-only.

### 2. OUT v0

- float pow; modpow; Mutex
- non-lit neg exp detection v0

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-pow` | `2.pow(10)` → 1024 → **accepted** |
| `std-pow-zero-exp` | `5.pow(0)` → 1 → **accepted** |
| `neg-e0281-pow-neg-exp` | `2.pow(-1)` → **E0281** |
| `neg-e0283-pow-overflow` (opc.) | lit overflow → **E0283** |

## Checklist

- [x] Pins E0281/E0283 + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **184/184** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
