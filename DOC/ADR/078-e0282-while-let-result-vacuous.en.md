Translation of `078-e0282-while-let-result-vacuous.md`; the original is normative. / Traducción de `078-e0282-while-let-result-vacuous.md`; el original es el normativo.

# ADR-078 — E0282 vacuous `while let` Result

- **Estado:** **aceptada** + **verified** Lex **180/180**
- **CUT-ID:** `TRAPS-WHILE-LET-RESULT-VACUOUS-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-077 while-let Result; ADR-056 E0277 Option; ADR-040 while-false; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** HIR reject pre-emit; measure neg → **E0282**; skip ≠ PASS.

## Context

ADR-077 Lex **178/178**. Mirror E0277: scrutinee **lit** Result that **never** matches the pattern → dead theater body.

**No** pinnear `while let Ok(x) = Ok(…)` (loop infinito). Lit mismatch only.

## Decision (GO pins)

### 1. Code

| Code | EN message | When |
|--------|------------|--------|
| **E0282** | `vacuous while-let result` | `while let Ok(…) = Err(<lit>)` **o** `while let Err(…) = Ok(<lit>)` |

### 2. IN v0

Scrutinee syntactically `Ok(lit)` / `Err(lit)` (lit ctors) incompatible with the pattern.

### 3. OUT v0

- Ok-forever / Err-forever lit same-variant (infinite) — no E0282; no hanging oracle
- non-lit scrutinee
- Mutex

### 4. Oracles

| Id | Expect |
|----|--------|
| `neg-e0282-ok-on-err` | `while let Ok(x) = Err(0) { … }` → **E0282** |
| `neg-e0282-err-on-ok` | `while let Err(e) = Ok(0) { … }` → **E0282** |
| pos: reusar whilelet-result-02 (non-lit / binding) | **accepted** |

## Checklist

- [x] Pins E0282 + IN/OUT + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **180/180** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
