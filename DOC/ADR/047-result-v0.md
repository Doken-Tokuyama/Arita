# ADR-047 — Result v0 (surface)

- **Estado:** **aceptada** + **verified** Lex **92/92**
- **CUT-ID:** `RESULT-V0-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-015/025 (match), ADR-006, ADR-022 safe-only; swallow trap = **ADR-048** `TRAPS-RESULT-SWALLOW-20260918` (**IMPL GO**)
- **Gobernanza:** **aceptada** + **IMPL GO**. Executor Ingeniero stack; Parser/Codegen HOLD. Desbloquea trampas Result; **Mutex PARK** (siguiente oleada).
- **Barra:** E2E measure; skip ≠ PASS; safe-only.

## Contexto

<person> «seguid»; Origin más tarde. Standby trap sin surface nueva → abrir **Result** (más apalancamiento que Mutex para IA + swallow theater).

## Decisión (pins GO)

### 1. IN v0

| Pieza | Regla |
|-------|--------|
| Tipo | `Result<T, E>` con `T, E ∈ {Int, Bool, String, ()}` |
| Constructores | `Ok(expr)` / `Err(expr)` (tipos deben unificar con T/E) |
| Match | `match <Result> { Ok(x) => … Err(e) => … }` — **ambos** brazos; exhaustivo |
| Bindings | `Ok(x)` / `Err(e)` bind en brazo (scope brazo) |
| Emit | `Result<T,E>` Rust + `Ok`/`Err` |

### 2. Diagnósticos

| Código | Mensaje EN (canónico) |
|--------|------------------------|
| **E0270** | `non-exhaustive result match` | falta Ok o Err |
| **E0271** | `result type mismatch` | Ok/Err payload vs T/E; o arm types (si no reutiliza E0203) |

Pin: si E0203 basta para payload mismatch, usarlo y **E0271 OUT**; Ingeniero elige en IMPL sin chocar códigos. Preferencia Arquitecto: **E0270** exhaustividad; payload → **E0203** si libre/alineado.

### 3. Oráculos measure

| Id | Expect |
|----|--------|
| `result-01` | Ok path stdout fijado |
| `result-02` | Err path stdout fijado |
| `neg-e0270` | match solo `Ok` → E0270 |

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
