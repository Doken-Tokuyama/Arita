# ADR-058 — Std `clone` v0 (String + Vec)

- **Estado:** **aceptada** + **verified** Lex **122/122**
- **CUT-ID:** `STD-CLONE-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-026 std minima; ADR-049 clear; ADR-052 pop; ADR-006 ownership; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS; sin timed.

## Contexto

Bootstrap-03 Lex **119/119**. Mutex/Origin/insert PARK. Surface ownership (String/Vec) necesita **`clone`** medible para IAs (evitar move theater / fake re-use). No inventa Mutex.

## Decisión (pins GO)

### 1. IN v0

| Surface | Retorno | Borrow | Emit |
|---------|---------|--------|------|
| `s.clone()` | `String` | shared (`&self`) — **no** exige `mut` | `Clone::clone` / `.clone()` |
| `v.clone()` | `Vec<T>` | shared — **no** exige `mut` | idem |

Pins:

1. Arity 0; args → E0203.
2. Receiver **String | Vec<_>`** tipado; otro → **E0206**.
3. No move del receiver (post-clone, original usable).
4. Whitelist ADR-026 ampliada + `clone`.
5. Safe-only; T ∈ tipos Vec F2 ya permitidos.

### 2. OUT v0

- `clone` sobre `Int`/`Bool`/`Option`/`Result` (Int Copy implícito ya; no método)
- `deep` / custom Clone / Rc/Arc
- Mutex / Cell
- `to_owned` alias

### 3. Oráculos measure

| Id | Expect |
|----|--------|
| `std-clone-string` | clone String; mut/usar ambas rutas; stdout canónico → **accepted** |
| `std-clone-vec` | clone Vec; pop/clear en una; la otra intacta (vía len/print) → **accepted** |
| `neg-e0206-clone-bad` (opc.) | `1.clone()` o método inventado → **E0206** |

## Checklist

- [x] Pins clone String/Vec + OUT + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **122/122** (STABLE_VERIFY)

## Cola

Mutex **PARK**. insert/index siguen PARK.
