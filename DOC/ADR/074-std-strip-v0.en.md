Translation of `074-std-strip-v0.md`; the original is normative. / Traducción de `074-std-strip-v0.md`; el original es el normativo.

# ADR-074 — String `strip_prefix` / `strip_suffix` → Option v0

- **Estado:** **aceptada** + **verified** Lex **169/169**
- **CUT-ID:** `STD-STRIP-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-050 Option; ADR-060–062; ADR-052 pop; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

trim_ends Lex **165/165**. Std que combina String + Option: **strip_prefix/suffix**.

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.strip_prefix(p)` | lit\|String | String shared | **`Option<String>`** | `.strip_prefix(&p).map(str::to_string)` |
| `s.strip_suffix(p)` | lit\|String | String shared | **`Option<String>`** | idem suffix |

Pins: arity 1; Some(owned remainder) / None; whitelist; Option match/if-let apply; safe-only.

### 2. OUT v0

- strip_prefix char; rich Pattern traits; Mutex

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-strip-prefix-some` | match Some → print remainder → **accepted** |
| `std-strip-prefix-none` | None path → **accepted** |
| `std-strip-suffix-some` (or combine) | Some → **accepted** |
| `neg-e0206-strip-vec` (opt.) | Vec → **E0206** |

Minimum: 1 Some prefix, 1 None prefix, 1 Some suffix (3 pos) + opt. neg.

## Checklist

- [x] Pins Option<String> + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **169/169** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
