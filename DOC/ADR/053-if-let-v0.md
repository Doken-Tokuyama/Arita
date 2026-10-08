# ADR-053 — `if let` Option v0

- **Estado:** **aceptada** + **verified** Lex **111/111**
- **CUT-ID:** `IF-LET-V0-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-050 Option; ADR-051 E0274 swallow; ADR-014/015 control+match; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

Post `pop`→Option (**108/108**). Surface Option + match exhaustivo ya existe; falta azúcar **`if let`** (patrón IA habitual). No inventa Mutex. `else` **obligatorio** en v0 → evita tragar `None` sin brazo (teatro hermano de E0274).

## Decisión (pins GO)

### 1. IN v0

```arita
if let Some(x) = <expr: Option<T>> {
  /* then; x: T in scope */
} else {
  /* else — obligatorio */
}
```

Pins:

1. Solo patrón **`Some(binding)`** sobre scrutinee **`Option<T>`**.
2. **`else` requerido** — sin else → **E0275** `if-let without else`.
3. Binding scope = bloque then; no filtra al else.
4. Tipos: then/else como `if` Bool existente (ambos Io/stmts v0; si if-let-as-expr OUT).
5. Emit: `if let Some(x) = … { … } else { … }` Rust seguro.

### 2. Diagnóstico

| Código | Mensaje EN | Cuándo |
|--------|------------|--------|
| **E0275** | `if-let without else` | falta `else` |
| E0203 / E0273 familia | scrutinee no Option / patrón ilegal | reutilizar si cabe; si patrón `None`/`Ok` → E0206-or-parse — pin: **solo Some** en gramática v0 |

### 3. OUT v0

- `if let None = …`
- `if let Ok/Err` (Result) → **ADR-054**
- `while let`
- `if let` sin else “parcial”
- if-let como expresión con valor
- Mutex

### 4. Oráculos measure

| Id | Expect |
|----|--------|
| `iflet-01-some` | Some path + else muerto print canónico → **accepted** |
| `iflet-02-none` | None → else path stdout fijado → **accepted** |
| `neg-e0275` | `if let Some(x) = o { … }` sin else → **E0275** |

## Checklist

- [x] Pins + E0275 + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **111/111** (STABLE_VERIFY)

## Cola

Mutex **PARK**. Result if-let → **ADR-054**. while-let = CUT posterior.
