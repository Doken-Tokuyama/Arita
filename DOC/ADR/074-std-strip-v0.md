# ADR-074 — String `strip_prefix` / `strip_suffix` → Option v0

- **Estado:** **aceptada** + **verified** Lex **169/169**
- **CUT-ID:** `STD-STRIP-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-050 Option; ADR-060–062; ADR-052 pop; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

trim_ends Lex **165/165**. Std que combina String + Option: **strip_prefix/suffix**.

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.strip_prefix(p)` | lit\|String | String shared | **`Option<String>`** | `.strip_prefix(&p).map(str::to_string)` |
| `s.strip_suffix(p)` | lit\|String | String shared | **`Option<String>`** | idem suffix |

Pins: arity 1; Some(resto owned) / None; whitelist; match/if-let Option aplican; safe-only.

### 2. OUT v0

- strip_prefix char; Pattern traits ricos; Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-strip-prefix-some` | match Some → print resto → **accepted** |
| `std-strip-prefix-none` | None path → **accepted** |
| `std-strip-suffix-some` (o combinar) | Some → **accepted** |
| `neg-e0206-strip-vec` (opc.) | Vec → **E0206** |

Mínimo: 1 Some prefix, 1 None prefix, 1 Some suffix (3 pos) + opc neg.

## Checklist

- [x] Pins Option<String> + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **169/169** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
