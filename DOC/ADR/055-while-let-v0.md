# ADR-055 — `while let` Option v0

- **Estado:** **aceptada** + **verified** Lex **116/116**
- **CUT-ID:** `WHILE-LET-V0-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-053/054 if-let; ADR-050 Option; ADR-052 Vec.pop; ADR-040 E0226 while-false; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

if-let Result **114/114**. OUT de 053/054: `while let`. Encaja con **`pop`→Option** (drenar Vec). No inventa Mutex.

## Decisión (pins GO)

### 1. IN v0

```arita
while let Some(x) = <expr: Option<T>> {
  /* body; x: T; expr re-eval cada iteración */
}
```

Pins:

1. Solo patrón **`Some(binding)`** sobre scrutinee **`Option<T>`**.
2. Scrutinee se **re-evalúa** cada vuelta (emit Rust `while let Some(x) = …`).
3. Sin `else` en while-let v0 (OUT).
4. Stmt form; body = block.
5. Caso útil canónico: `while let Some(x) = v.pop() { … }` (v: mut Vec).

### 2. OUT v0

- `while let Ok/Err` (Result) — CUT futuro
- `while let None = …`
- `while let` + else
- Mutex / async while-let
- Vacuous `while let Some(x) = None { … }` theater → **ADR-056 / E0277**

### 3. Oráculos measure

| Id | Expect |
|----|--------|
| `whilelet-01-pop-drain` | push 1..n; `while let Some(x) = v.pop()` imprime en orden LIFO canónico → **accepted** |
| `whilelet-02-none` | `while let Some(x) = None { print(x) }` luego `print("ok")` → body 0 iters, stdout `ok` → **accepted** (no E0xxx v0) |
| (opc.) reusar pop mut | pop sin mut en scrutinee compuesto → E0202 si aplica |

## Checklist

- [x] Pins Some-only + pop drain + OUT
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **116/116** (STABLE_VERIFY)

## Cola

Mutex **PARK**. Trampa vacuous → **ADR-056**. while-let Result = posterior.
