Translation of `055-while-let-v0.md`; the original is normative. / Traducción de `055-while-let-v0.md`; el original es el normativo.

# ADR-055 — `while let` Option v0

- **Estado:** **aceptada** + **verified** Lex **116/116**
- **CUT-ID:** `WHILE-LET-V0-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-053/054 if-let; ADR-050 Option; ADR-052 Vec.pop; ADR-040 E0226 while-false; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

if-let Result **114/114**. OUT de 053/054: `while let`. Encaja con **`pop`→Option** (drenar Vec). Does not invent Mutex.

## Decision (GO pins)

### 1. IN v0

```arita
while let Some(x) = <expr: Option<T>> {
  /* body; x: T; expr re-eval each iteration */
}
```

Pins:

1. Only pattern **`Some(binding)`** on scrutinee **`Option<T>`**.
2. Scrutinee se **re-evaluates** each vuelta (emit Rust `while let Some(x) = …`).
3. No `else` in while-let v0 (OUT).
4. Stmt form; body = block.
5. Caso useful canonical: `while let Some(x) = v.pop() { … }` (v: mut Vec).

### 2. OUT v0

- `while let Ok/Err` (Result) — CUT futuro
- `while let None = …`
- `while let` + else
- Mutex / async while-let
- Vacuous `while let Some(x) = None { … }` theater → **ADR-056 / E0277**

### 3. Oracles measure

| Id | Expect |
|----|--------|
| `whilelet-01-pop-drain` | push 1..n; `while let Some(x) = v.pop()` imprime en orden LIFO canonical → **accepted** |
| `whilelet-02-none` | `while let Some(x) = None { print(x) }` luego `print("ok")` → body 0 iters, stdout `ok` → **accepted** (no E0xxx v0) |
| (opt.) reusar pop mut | pop missing mut en scrutinee compuesto → E0202 si aplica |

## Checklist

- [x] Pins Some-only + pop drain + OUT
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **116/116** (STABLE_VERIFY)

## Queue

Mutex **PARK**. Trampa vacuous → **ADR-056**. while-let Result = posterior.
