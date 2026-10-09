Translation of `041-e0227-if-false.md`; the original is normative. / Traducción de `041-e0227-if-false.md`; el original es el normativo.

# ADR-041 — E0227 vacuous `if false` (theater)

- **Estado:** **aceptada** + **verified** (Lex measure **82** accepted + **2** gated)
- **CUT-ID:** `TRAPS-IF-FALSE-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-014 (`if`), ADR-040 / E0226 (`while false`), sondeo HOLE 039b
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Separado de ADR-040.
- **Barra:** E2E measure neg; skip ≠ PASS. Sondeo: hoy PASS → hole.

## Context

Sondeo: `if false { print("x") }` + `print("ok")` = **PASS** hoy. Parelo a E0226 pero **CUT aparte** (no mezclar while/if).

## Decision (pins GO)

### 1. Code

| Code | Message EN canonical |
|--------|---------------------|
| **E0227** | `vacuous if false` |

No diluir E0220 / E0226.

### 2. Pattern v0

`if false { … }` (cond = LitBool `false` only). `else` optional: si `if false … else { real }` — OUT v0 (only then-without-else o then empty with false lit).

Pin v0 strict: **only** `if false { … }` without `else` (unreachable then body + continuation theater).

### 3. Oracles

**Neg canonical** (HOLE probe; evita E0215):

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

**Neg morph** (then muerto / cobertura theater; assert **no** tautological):

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

**Anti-collision:** do not use `assert 1 == 1` / `assert n == n` inside the then → that is **E0215**, no this CUT.

Expect neg: **E0227**.

### 4. OUT

- `if <Bool var>` / non-literal
- `if false … else { … }` (CUT future si hace falta)
- Mezclar with E0226
- Mutex/Result / CI Origin (siguen park/bloqueados)

## Checklist

- [x] E0227 + EN + CUT OK
- [x] Separado de while false OK
- [x] GO DOC+IMPL
- [x] Neg + measure (Ingeniero) — `neg-e0227-if-false` → NEG_ORACLES; Lex **82**+2 gated
