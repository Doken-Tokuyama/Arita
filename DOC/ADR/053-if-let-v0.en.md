Translation of `053-if-let-v0.md`; the original is normative. / Traducción de `053-if-let-v0.md`; el original es el normativo.

# ADR-053 — `if let` Option v0

- **Estado:** **aceptada** + **verified** Lex **111/111**
- **CUT-ID:** `IF-LET-V0-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-050 Option; ADR-051 E0274 swallow; ADR-014/015 control+match; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

Post `pop`→Option (**108/108**). Option surface + exhaustive match already exist; missing sugar **`if let`** (common AI pattern). Does not invent Mutex. `else` **mandatory** in v0 → avoids swallowing `None` without an arm (theater sibling of E0274).

## Decision (GO pins)

### 1. IN v0

```arita
if let Some(x) = <expr: Option<T>> {
  /* then; x: T in scope */
} else {
  /* else — obligatorio */
}
```

Pins:

1. Only pattern **`Some(binding)`** on scrutinee **`Option<T>`**.
2. **`else` required** — without else → **E0275** `if-let without else`.
3. Binding scope = bloque then; no filtra al else.
4. Tipos: then/else as `if` Bool existente (both Io/stmts v0; si if-let-as-expr OUT).
5. Emit: `if let Some(x) = … { … } else { … }` Rust seguro.

### 2. Diagnosis

| Code | EN message | When |
|--------|------------|--------|
| **E0275** | `if-let without else` | falta `else` |
| E0203 / E0273 family | scrutinee not Option / illegal pattern | reuse if it fits; if pattern `None`/`Ok` → E0206-or-parse — pin: **only Some** in grammar v0 |

### 3. OUT v0

- `if let None = …`
- `if let Ok/Err` (Result) → **ADR-054**
- `while let`
- `if let` without else “partial”
- if-let as expression with value
- Mutex

### 4. Oracles measure

| Id | Expect |
|----|--------|
| `iflet-01-some` | Some path + else muerto print canonical → **accepted** |
| `iflet-02-none` | None → else path stdout fijado → **accepted** |
| `neg-e0275` | `if let Some(x) = o { … }` without else → **E0275** |

## Checklist

- [x] Pins + E0275 + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **111/111** (STABLE_VERIFY)

## Queue

Mutex **PARK**. Result if-let → **ADR-054**. while-let = CUT posterior.
