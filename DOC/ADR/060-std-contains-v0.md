# ADR-060 — String `contains` v0

- **Estado:** **aceptada** + **verified** Lex **128/128**
- **CUT-ID:** `STD-CONTAINS-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-026; ADR-059 push_str; ADR-023 Bool; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

push_str Lex **125/125**. Siguiente std medible para IAs: **`contains` → Bool** (sin Char/regex/Mutex).

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.contains(arg)` | `String` **o** lit str | `String` (shared) | **`Bool`** | `str::contains` / `.contains(&…)` |

Pins:

1. Arity **1**; malo → E0203.
2. Receiver **no** exige mut (shared borrow).
3. Arg: String (emit `contains(&arg)`) o lit.
4. Whitelist + `contains`. Safe-only.
5. Resultado usable en `if` / `assert` / `let b: Bool`.

### 2. OUT v0

- `starts_with` / `ends_with` / `find` / regex (CUTs posteriores)
- `contains` en Vec (posterior o E0206)
- Char / pattern traits ricos
- Mutex

### 3. Oráculos measure

| Id | Expect |
|----|--------|
| `std-contains-true` | hay substring → print/assert true canónico → **accepted** |
| `std-contains-false` | no hay → false canónico → **accepted** |
| `neg-e0206-contains-vec` (opc.) | `v.contains(…)` Vec → **E0206** |

## Checklist

- [x] Pins + OUT + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **128/128** (STABLE_VERIFY)

## Cola

Mutex **PARK**. starts_with → **ADR-061**.
