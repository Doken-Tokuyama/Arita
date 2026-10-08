# ADR-085 — Vec `get` → Option v0 (safe index)

- **Estado:** **aceptada** + **verified** Lex **205/205**
- **CUT-ID:** `STD-VEC-GET-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-036 (index PARK; este CUT es get Option — no `[]`); ADR-050 Option; ADR-052 pop; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

Checked arith cerrada Lex **201/201**. ADR-036 aparcó `[]`/get por panic theater. **`get(i) -> Option<T>`** es el unpark safe (None OOB).

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `v.get(i)` | **Int** i | `Vec<T>` shared | **`Option<T>`** | ver abajo |

Emit orientativo:
- si `i < 0` → `None`
- else `v.get(i as usize).copied()` (Int) / `.cloned()` (String)

Pins: arity 1; T ∈ Vec F2; **no** habilita `v[i]`; whitelist; safe-only.

### 2. OUT v0

- `v[i]` / `get_mut` / slice
- Mutex
- insert/remove (siguen PARK)

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-vec-get-some` | in-range → Some path → **accepted** |
| `std-vec-get-none-oob` | i ≥ len → None → **accepted** |
| `std-vec-get-none-neg` | i < 0 → None → **accepted** |
| `neg-e0206-get-string` (opc.) | String.get → **E0206** |

## Checklist

- [x] Pins Option get + OUT `[]` + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **205/205** (STABLE_VERIFY)

## Cola

Mutex **PARK**. `[]` sigue PARK.
