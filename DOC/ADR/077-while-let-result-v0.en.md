Translation of `077-while-let-result-v0.md`; the original is normative. / Traducción de `077-while-let-result-v0.md`; el original es el normativo.

# ADR-077 — `while let` Result v0

- **Estado:** **aceptada** + **verified** Lex **178/178**
- **CUT-ID:** `WHILE-LET-RESULT-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-055 while-let Option; ADR-054 if-let Result; ADR-047 Result; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

Vec.repeat Lex **175/175**. OUT de 055: **while-let Result**. Completes the matrix if-let/while-let × Option/Result. No Mutex.

## Decision (GO pins)

### 1. IN v0

```arita
while let Ok(x) = <expr: Result<T,E>> { /* body; re-eval */ }
while let Err(e) = <expr: Result<T,E>> { /* body; re-eval */ }
```

Pins:

1. Patterns **`Ok(binding)`** | **`Err(binding)`** solo.
2. Scrutinee re-eval each iter (emit Rust `while let`).
3. No else.
4. Stmt form.
5. Canonical: `mut r: Result<…> = Ok(…)`; body assigns `r = Err(…)` / `Ok(…)` to stop/continue (if Result assign already legal; else report HOLE).

### 2. OUT v0

- vacuous lit theater (Ok forever) — later trap CUT
- Mutex
- while-let Option (already 055)

### 3. Oracles

| Id | Expect |
|----|--------|
| `whilelet-result-01-ok-once` | enters Ok, then Err; canonical stdout → **accepted** |
| `whilelet-result-02-err-skip` | entry scrutinee Err; body 0; print after → **accepted** |
| `whilelet-result-03-err-pat` (opt.) | `while let Err(e) = …` path → **accepted** |

## Checklist

- [x] Pins Ok/Err while-let + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **178/178** (STABLE_VERIFY); mut Result<Int,Int> OK

## Queue

Mutex **PARK**. Vacuous → **ADR-078**.
