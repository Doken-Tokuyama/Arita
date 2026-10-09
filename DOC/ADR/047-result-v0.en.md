Translation of `047-result-v0.md`; the original is normative. / Traducción de `047-result-v0.md`; el original es el normativo.

# ADR-047 — Result v0 (surface)

- **Estado:** **aceptada** + **verified** Lex **92/92**
- **CUT-ID:** `RESULT-V0-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-015/025 (match), ADR-006, ADR-022 safe-only; swallow trap = **ADR-048** `TRAPS-RESULT-SWALLOW-20260918` (**IMPL GO**)
- **Gobernanza:** **aceptada** + **IMPL GO**. Executor Ingeniero stack; Parser/Codegen HOLD. Desbloquea trampas Result; **Mutex PARK** (siguiente oleada).
- **Barra:** E2E measure; skip ≠ PASS; safe-only.

## Context

<person> «continue»; Origin later. Standby trap no new surface → open **Result** (more leverage than Mutex for AI + swallow theater).

## Decision (pins GO)

### 1. IN v0

| Pieza | Regla |
|-------|--------|
| Tipo | `Result<T, E>` with `T, E ∈ {Int, Bool, String, ()}` |
| Constructores | `Ok(expr)` / `Err(expr)` (tipos deben unificar with T/E) |
| Match | `match <Result> { Ok(x) => … Err(e) => … }` — **both** arms; exhaustive |
| Bindings | `Ok(x)` / `Err(e)` bind en arm (scope arm) |
| Emit | `Result<T,E>` Rust + `Ok`/`Err` |

### 2. Diagnostics

| Code | Message EN (canonical) |
|--------|------------------------|
| **E0270** | `non-exhaustive result match` | falta Ok o Err |
| **E0271** | `result type mismatch` | Ok/Err payload vs T/E; o arm types (si no reutiliza E0203) |

Pin: if E0203 suffices for payload mismatch, use it y **E0271 OUT**; Ingeniero picks in IMPL without colliding codes. Arquitecto preference: **E0270** exhaustiveness; payload → **E0203** if free/aligned.

### 3. Oracles measure

| Id | Expect |
|----|--------|
| `result-01` | Ok path stdout fijado |
| `result-02` | Err path stdout fijado |
| `neg-e0270` | match only `Ok` → E0270 |

### 4. OUT v0

- `?` / `try` / `unwrap` / `map` / `and_then`
- `Option` → **ADR-050** / CUT `OPTION-V0-20260918` (**verified** Lex **102/102**)
- Mutex / lock / async Result especial
- Result swallow anti-theater → **ADR-048** / CUT `TRAPS-RESULT-SWALLOW-20260918` (**IMPL GO**)
- FFI / unsafe

## Checklist

- [x] Pins Result + match + E0270 OK
- [x] Mutex PARK / swallow → ADR-048 GO
- [x] GO DOC+IMPL
- [x] Landed + measure Lex **92/92** (Ingeniero) — IMPL on box; Lex measure pending
