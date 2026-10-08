# ADR-054 — `if let` Result v0

- **Estado:** **aceptada** + **verified** Lex **114/114**
- **CUT-ID:** `IF-LET-RESULT-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-053 if-let Option (espejo); ADR-047 Result; ADR-048 E0272; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

ADR-053 landed Lex **111/111** (`if let Some` + else obligatorio). OUT dejaba Result if-let. Surface Result ya existe → desbloquea azúcar simétrica **sin Mutex**.

## Decisión (pins GO)

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

1. Patrones **`Ok(binding)`** | **`Err(binding)`** solo sobre scrutinee **`Result<T,E>`**.
2. **`else` obligatorio** — sin else → **E0275** (reutilizar código ADR-053; mismo mensaje `if-let without else`).
3. Binding scope = bloque then.
4. Stmt form (como 053); if-let-as-expr OUT.
5. Emit Rust `if let Ok/Err(...) = ... { ... } else { ... }`; safe-only.

### 2. OUT v0

- `while let`
- `if let` Option (ya 053)
- if-let sin else
- Mutex / `?` / unwrap
- if-let anidado fancy / or-patterns

### 3. Oráculos measure

| Id | Expect |
|----|--------|
| `iflet-result-01-ok` | Ok path + else → **accepted** |
| `iflet-result-02-err` | Err path + else → **accepted** |
| `neg-e0275-result` | `if let Ok(x) = r { … }` sin else → **E0275** |

## Checklist

- [x] Pins Ok/Err + E0275 reuse + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **114/114** (STABLE_VERIFY)

## Cola

Mutex **PARK**. while-let → **ADR-055**.
