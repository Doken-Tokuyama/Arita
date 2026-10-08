# ADR-080 — Int `checked_add` → Option v0

- **Estado:** **aceptada** + **verified** Lex **187/187**
- **CUT-ID:** `STD-CHECKED-ADD-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-045 E0217 (reject lit +/-/*); ADR-050 Option; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

pow Lex **184/184**. E0217 rechaza overflow lit en `+`. Falta API **runtime-safe**: `checked_add` → Option (IA puede manejar None sin panic theater).

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `a.checked_add(b)` | 1× **Int** | **Int** | **`Option<Int>`** | `a.checked_add(b)` → Some/None |

Pins: arity 1; Int only; whitelist; match/if-let Option; safe-only. Complementa E0217 (no lo sustituye para `+` lit).

### 2. OUT v0

- checked_sub → **ADR-081**; mul/div posteriores
- saturating_*; wrapping_*
- Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-checked-add-some` | `1.checked_add(2)` → Some(3) path → **accepted** |
| `std-checked-add-none` | `MAX.checked_add(1)` → None path → **accepted** |
| `neg-e0206-checked-add-string` (opc.) | String → **E0206** |

## Checklist

- [x] Pins Option + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **187/187** (STABLE_VERIFY)

## Cola

Mutex **PARK**. checked_sub/mul = posteriores.
