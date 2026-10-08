# ADR-050 — Option v0 (surface)

- **Estado:** **aceptada** + **verified** Lex **102/102**
- **CUT-ID:** `OPTION-V0-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-047 Result v0 (espejo; Option era OUT → este CUT); ADR-015/025 match; ADR-022 safe-only; Option-swallow = CUT **posterior** (como ADR-048 tras Result)
- **Gobernanza:** <person> avanzar sin parar; Mutex **PARK** (no inventar). Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS; safe-only.

## Contexto

Cola post STD-CLEAR **99/99**: Mutex PARK; Investigador sin HOLE no-control portable. Siguiente surface **ya prevista** en ADR-047 OUT: **`Option`**. No inventa Mutex; desbloquea oleada Option-swallow después.

## Decisión (pins GO)

### 1. IN v0

| Pieza | Regla |
|-------|--------|
| Tipo | `Option<T>` con `T ∈ {Int, Bool, String, ()}` |
| Constructores | `Some(expr)` / `None` |
| Match | `match <Option> { Some(x) => … None => … }` — **ambos** brazos; exhaustivo |
| Bindings | `Some(x)` bind en brazo (scope brazo) |
| Emit | `Option<T>` Rust + `Some`/`None` |

### 2. Diagnósticos

| Código | Mensaje EN (canónico) | Cuándo |
|--------|------------------------|--------|
| **E0273** | `non-exhaustive option match` | falta `Some` o `None` |
| Payload / arm types | reutilizar **E0203** si libre/alineado (como Result) | mismatch |

No chocar E0270/E0271/E0272 (Result familia).

### 3. Oráculos measure

| Id | Expect |
|----|--------|
| `option-01` | Some path stdout fijado |
| `option-02` | None path stdout fijado |
| `neg-e0273` | match solo `Some` (o solo `None`) → **E0273** |

Paths: Ingeniero fija bajo `ejemplos/`; forma canónica vía `let o: Option<Int> = …` (no exigir `fn → Option` si `fn_ret` sigue Io/Int — espejo ADR-047/048).

### 4. OUT v0

- `?` / `unwrap` / `expect` / `map` / `and_then` / `ok_or`
- `if let` / `while let`
- Option-swallow anti-theater → **ADR-051** (**verified** Lex **105/105**)
- Mutex / Result cambios
- `fn … -> Option<…>` si surface ret aun no lo permite — usar `let` (como Result v0)

## Checklist

- [x] Pins Option + match + E0273
- [x] Mutex PARK / swallow CUT posterior
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **102/102** (STABLE_VERIFY)

## Cola

1. Este CUT IMPL + measure  
2. ~~Option-swallow~~ → **ADR-051** **verified** Lex **105/105**  
3. Mutex **PARK** hasta surface dedicada
