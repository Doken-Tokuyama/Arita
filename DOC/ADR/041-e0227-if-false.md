# ADR-041 — E0227 vacuous `if false` (theater)

- **Estado:** **aceptada** + **verified** (Lex measure **82** accepted + **2** gated)
- **CUT-ID:** `TRAPS-IF-FALSE-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-014 (`if`), ADR-040 / E0226 (`while false`), sondeo HOLE 039b
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Separado de ADR-040.
- **Barra:** E2E measure neg; skip ≠ PASS. Sondeo: hoy PASS → hole.

## Contexto

Sondeo: `if false { print("x") }` + `print("ok")` = **PASS** hoy. Parelo a E0226 pero **CUT aparte** (no mezclar while/if).

## Decisión (pins GO)

### 1. Código

| Código | Mensaje EN canónico |
|--------|---------------------|
| **E0227** | `vacuous if false` |

No diluir E0220 / E0226.

### 2. Patrón v0

`if false { … }` (cond = LitBool `false` only). `else` opcional: si `if false … else { real }` — OUT v0 (solo then-sin-else o then vacío con false lit).

Pin v0 estricto: **solo** `if false { … }` sin `else` (cuerpo then inalcanzable + continuación theater).

### 3. Oráculos

**Neg canónico** (sondeo HOLE; evita E0215):

`ejemplos/f2.1/neg/e0227-if-false.arita`:

```arita
module e0227_if_false
fn main() -> Io<()> {
  if false {
    print("x")
  }
  print("ok")
}
```

**Neg morph** (then muerto / cobertura theater; assert **no** tautológico):

```arita
module e0227_if_false_assert
fn main() -> Io<()> {
  let x: Int = 3
  if false {
    assert x == 3
  }
  assert x == 3
  print("ok")
}
```

**Clean (no E0227):** `if true { assert x == 3 }` o `if <Bool var>`.

**Anti-colisión:** no usar `assert 1 == 1` / `assert n == n` dentro del then → eso es **E0215**, no este CUT.

Expect neg: **E0227**.

### 4. OUT

- `if <Bool var>` / non-literal
- `if false … else { … }` (CUT futuro si hace falta)
- Mezclar con E0226
- Mutex/Result / CI Origin (siguen park/bloqueados)

## Checklist

- [x] E0227 + EN + CUT OK
- [x] Separado de while false OK
- [x] GO DOC+IMPL
- [x] Neg + measure (Ingeniero) — `neg-e0227-if-false` → NEG_ORACLES; Lex **82**+2 gated
