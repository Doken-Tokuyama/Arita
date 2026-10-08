# ADR-077 — `while let` Result v0

- **Estado:** **aceptada** + **verified** Lex **178/178**
- **CUT-ID:** `WHILE-LET-RESULT-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-055 while-let Option; ADR-054 if-let Result; ADR-047 Result; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

Vec.repeat Lex **175/175**. OUT de 055: **while-let Result**. Completa matriz if-let/while-let × Option/Result. Sin Mutex.

## Decisión (pins GO)

### 1. IN v0

```arita
while let Ok(x) = <expr: Result<T,E>> { /* body; re-eval */ }
while let Err(e) = <expr: Result<T,E>> { /* body; re-eval */ }
```

Pins:

1. Patrones **`Ok(binding)`** | **`Err(binding)`** solo.
2. Scrutinee re-eval cada iter (emit Rust `while let`).
3. Sin else.
4. Stmt form.
5. Canónico: `mut r: Result<…> = Ok(…)`; body asigna `r = Err(…)` / `Ok(…)` para terminar/continuar (si assign Result ya legal; si no, reportar HOLE).

### 2. OUT v0

- vacuous lit theater (Ok forever) — CUT trampa posterior
- Mutex
- while-let Option (ya 055)

### 3. Oráculos

| Id | Expect |
|----|--------|
| `whilelet-result-01-ok-once` | entra Ok, luego Err; stdout canónico → **accepted** |
| `whilelet-result-02-err-skip` | scrutinee Err de entrada; body 0; print after → **accepted** |
| `whilelet-result-03-err-pat` (opc.) | `while let Err(e) = …` path → **accepted** |

## Checklist

- [x] Pins Ok/Err while-let + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **178/178** (STABLE_VERIFY); mut Result<Int,Int> OK

## Cola

Mutex **PARK**. Vacuous → **ADR-078**.
