Translation of `085-std-vec-get-v0.md`; the original is normative. / Traducción de `085-std-vec-get-v0.md`; el original es el normativo.

# ADR-085 — Vec `get` → Option v0 (safe index)

- **Estado:** **aceptada** + **verified** Lex **205/205**
- **CUT-ID:** `STD-VEC-GET-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-036 (index PARK; este CUT es get Option — no `[]`); ADR-050 Option; ADR-052 pop; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

Checked arith closed Lex **201/201**. ADR-036 parked `[]`/get for panic theater. **`get(i) -> Option<T>`** is the safe unpark (None OOB).

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `v.get(i)` | **Int** i | `Vec<T>` shared | **`Option<T>`** | see below |

Emit orientativo:
- si `i < 0` → `None`
- else `v.get(i as usize).copied()` (Int) / `.cloned()` (String)

Pins: arity 1; T ∈ Vec F2; **no** enable `v[i]`; whitelist; safe-only.

### 2. OUT v0

- `v[i]` / `get_mut` / slice
- Mutex
- insert/remove (remain PARK)

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-vec-get-some` | in-range → Some path → **accepted** |
| `std-vec-get-none-oob` | i ≥ len → None → **accepted** |
| `std-vec-get-none-neg` | i < 0 → None → **accepted** |
| `neg-e0206-get-string` (opt.) | String.get → **E0206** |

## Checklist

- [x] Pins Option get + OUT `[]` + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **205/205** (STABLE_VERIFY)

## Queue

Mutex **PARK**. `[]` remains PARK.
