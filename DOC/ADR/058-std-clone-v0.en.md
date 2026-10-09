Translation of `058-std-clone-v0.md`; the original is normative. / Traducción de `058-std-clone-v0.md`; el original es el normativo.

# ADR-058 — Std `clone` v0 (String + Vec)

- **Estado:** **aceptada** + **verified** Lex **122/122**
- **CUT-ID:** `STD-CLONE-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-026 std minima; ADR-049 clear; ADR-052 pop; ADR-006 ownership; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS; sin timed.

## Context

Bootstrap-03 Lex **119/119**. Mutex/Origin/insert PARK. Ownership surface (String/Vec) needs **`clone`** measurable for AIs (evitar move theater / fake re-use). Does not invent Mutex.

## Decision (GO pins)

### 1. IN v0

| Surface | Return | Borrow | Emit |
|---------|---------|--------|------|
| `s.clone()` | `String` | shared (`&self`) — **no** requires `mut` | `Clone::clone` / `.clone()` |
| `v.clone()` | `Vec<T>` | shared — **no** requires `mut` | idem |

Pins:

1. Arity 0; args → E0203.
2. Receiver **String | Vec<_>`** tipado; otro → **E0206**.
3. No move del receiver (post-clone, original usable).
4. Whitelist ADR-026 expanded + `clone`.
5. Safe-only; T ∈ Vec F2 types ya permitidos.

### 2. OUT v0

- `clone` sobre `Int`/`Bool`/`Option`/`Result` (Int Copy already implicit; no method)
- `deep` / custom Clone / Rc/Arc
- Mutex / Cell
- `to_owned` alias

### 3. Oracles measure

| Id | Expect |
|----|--------|
| `std-clone-string` | clone String; mut/use both paths; canonical stdout → **accepted** |
| `std-clone-vec` | clone Vec; pop/clear en una; la otra intact (via len/print) → **accepted** |
| `neg-e0206-clone-bad` (opt.) | `1.clone()` or invented method → **E0206** |

## Checklist

- [x] Pins clone String/Vec + OUT + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **122/122** (STABLE_VERIFY)

## Queue

Mutex **PARK**. insert/index remain PARK.
