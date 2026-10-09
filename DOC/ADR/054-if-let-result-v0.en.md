Translation of `054-if-let-result-v0.md`; the original is normative. / Traducción de `054-if-let-result-v0.md`; el original es el normativo.

# ADR-054 — `if let` Result v0

- **Estado:** **aceptada** + **verified** Lex **114/114**
- **CUT-ID:** `IF-LET-RESULT-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-053 if-let Option (espejo); ADR-047 Result; ADR-048 E0272; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

ADR-053 landed Lex **111/111** (`if let Some` + mandatory else). OUT left Result if-let. Result surface already exists → unlocks symmetric sugar **no Mutex**.

## Decision (GO pins)

### 1. IN v0

```arita
if let Ok(x) = <expr: Result<T,E>> {
  /* then; x: T */
} else {
  /* else — obligatorio */
}

if let Err(e) = <expr: Result<T,E>> {
  /* then; e: E */
} else {
  /* else — obligatorio */
}
```

Pins:

1. Patterns **`Ok(binding)`** | **`Err(binding)`** solo sobre scrutinee **`Result<T,E>`**.
2. **`else` mandatory** — without else → **E0275** (reuse code ADR-053; same message `if-let without else`).
3. Binding scope = bloque then.
4. Stmt form (as 053); if-let-as-expr OUT.
5. Emit Rust `if let Ok/Err(...) = ... { ... } else { ... }`; safe-only.

### 2. OUT v0

- `while let`
- `if let` Option (ya 053)
- if-let without else
- Mutex / `?` / unwrap
- if-let anidado fancy / or-patterns

### 3. Oracles measure

| Id | Expect |
|----|--------|
| `iflet-result-01-ok` | Ok path + else → **accepted** |
| `iflet-result-02-err` | Err path + else → **accepted** |
| `neg-e0275-result` | `if let Ok(x) = r { … }` without else → **E0275** |

## Checklist

- [x] Pins Ok/Err + E0275 reuse + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **114/114** (STABLE_VERIFY)

## Queue

Mutex **PARK**. while-let → **ADR-055**.
