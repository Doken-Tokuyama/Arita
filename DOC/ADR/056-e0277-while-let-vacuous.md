# ADR-056 — E0277 vacuous `while let` / None theater

- **Estado:** **aceptada** + **verified** Lex **118/118**
- **CUT-ID:** `TRAPS-WHILE-LET-VACUOUS-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-055 while-let (OUT vacuous → este CUT); ADR-040 E0226 while-false; ADR-041 E0227 if-false; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** HIR reject pre-emit; measure neg → **E0277**; skip ≠ PASS.

## Contexto

ADR-055 landed Lex **116/116**. `whilelet-02-none` acepta body-0 + print posterior (OK). Queda el teatro **vacuous** `while let Some(…) = None { … }` (espejo E0226 `while false`): loop muerto que finge cobertura.

## Decisión (pins GO)

### 1. Código

| Código | Mensaje EN | Cuándo |
|--------|------------|--------|
| **E0277** | `vacuous while-let none` | `while let Some(…) = <scrutinee>` donde scrutinee es **`None` literal** (siempre ausente) |

### 2. IN v0

```arita
while let Some(x) = None {
  print(x)   // dead
}
print("ok")  // theater verde sin E0277
```

Pin: scrutinee **sintácticamente** `None` (path/ctor lit Option vacío). No hace falta const-eval profunda v0.

Morph opc.: `while let Some(_) = None { }` → E0277.

### 3. OUT v0

- `while let Some(x) = v.pop()` (no lit None) — válido 055
- `while false` → sigue E0226
- while-let Result / Mutex
- Const-fold de exprs no-`None` (OUT v0)

### 4. Oráculos

| Id | Expect |
|----|--------|
| `neg-e0277` | fixture §2 → **E0277** |
| `neg-e0277-empty-body` (opc.) | `while let Some(_) = None { }` → **E0277** |
| pos: reusar `whilelet-01-pop-drain` / `whilelet-02` solo si no lit-None en Some-bind — `whilelet-02` hoy es `= None` con Some bind → **debe pasar a E0277** o reescribir pos |

**Pin oráculo:** si `whilelet-02-none` actual es exactamente el teatro §2, **retirarlo de accepted** o reescribir pos a scrutinee no-lit (p.ej. `let o: Option<Int> = None; while let Some(x) = o { … }`) — binding/`let` **no** es lit en scrutinee position → **no** E0277 v0. Ingeniero elige: (A) pos = `let o = None; while let Some(x) = o` accepted; (B) neg = lit `None` E0277.

Recomendación Arquitecto: **(A)+(B)** ambos.

## Checklist

- [x] Pins E0277 + IN/OUT + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **118/118** (STABLE_VERIFY); whilelet-02 → let-binding

## Cola

Mutex **PARK**.
