Translation of `050-option-v0.md`; the original is normative. / Traducción de `050-option-v0.md`; el original es el normativo.

# ADR-050 — Option v0 (surface)

- **Estado:** **aceptada** + **verified** Lex **102/102**
- **CUT-ID:** `OPTION-V0-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-047 Result v0 (espejo; Option era OUT → este CUT); ADR-015/025 match; ADR-022 safe-only; Option-swallow = CUT **posterior** (como ADR-048 tras Result)
- **Gobernanza:** <person> avanzar sin parar; Mutex **PARK** (no inventar). Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS; safe-only.

## Context

Queue after STD-CLEAR **99/99**: Mutex PARK; Investigator without portable non-control HOLE. Next surface **already planned** in ADR-047 OUT: **`Option`**. Does not invent Mutex; unlocks the Option-swallow wave afterwards.

## Decision (GO pins)

### 1. IN v0

| Pieza | Regla |
|-------|--------|
| Tipo | `Option<T>` con `T ∈ {Int, Bool, String, ()}` |
| Constructores | `Some(expr)` / `None` |
| Match | `match <Option> { Some(x) => … None => … }` — **both** arms; exhaustivo |
| Bindings | `Some(x)` bind en brazo (scope brazo) |
| Emit | `Option<T>` Rust + `Some`/`None` |

### 2. Diagnostics

| Code | EN message (canonical) | When |
|--------|------------------------|--------|
| **E0273** | `non-exhaustive option match` | falta `Some` o `None` |
| Payload / arm types | reuse **E0203** if free/aligned (like Result) | mismatch |

Do not collide with E0270/E0271/E0272 (Result family).

### 3. Oracles measure

| Id | Expect |
|----|--------|
| `option-01` | Some path stdout fijado |
| `option-02` | None path stdout fijado |
| `neg-e0273` | match solo `Some` (o solo `None`) → **E0273** |

Paths: Ingeniero fixes under `ejemplos/`; canonical form via `let o: Option<Int> = …` (do not require `fn → Option` si `fn_ret` still Io/Int — mirror ADR-047/048).

### 4. OUT v0

- `?` / `unwrap` / `expect` / `map` / `and_then` / `ok_or`
- `if let` / `while let`
- Option-swallow anti-theater → **ADR-051** (**verified** Lex **105/105**)
- Mutex / Result cambios
- `fn … -> Option<…>` if ret surface still disallows it — use `let` (like Result v0)

## Checklist

- [x] Pins Option + match + E0273
- [x] Mutex PARK / swallow CUT posterior
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **102/102** (STABLE_VERIFY)

## Queue

1. This CUT IMPL + measure  
2. ~~Option-swallow~~ → **ADR-051** **verified** Lex **105/105**  
3. Mutex **PARK** until dedicated surface
