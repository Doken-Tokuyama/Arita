# ADR-045 — E0217 integer overflow (`+` `-` `*`)

- **Estado:** **aceptada** + **verified** (Lex measure **87** accepted + **2** gated (add+sub+mul oracles); **STABLE_VERIFY**)
- **CUT-ID:** `TRAPS-INT-OVERFLOW-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL; sondeo HOLE)
- **Relacionados:** ADR-006 (Int binary), ADR-044 / E0216 (div0), ADR-022 safe-only
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Rechazo **HIR antes de emit**; no rustc E0100 / panic / wrap release.
- **Barra:** E2E measure neg; skip ≠ PASS.

## Contexto (sondeo)

| Axis | Resultado |
|------|-----------|
| Expressible | YES: `MAX+1` / `a+1` con `a=MAX` (LitInt i64) |
| HIR gate | **NO** (solo E0216 div0) |
| Emit | plain `+` (no checked) |
| Const | rustc E0100 |
| Runtime debug | panic |
| Runtime **release** | **silent wrap** → `i64::MIN` |
| Same class | Sub / Mul |

## Decisión (pins GO)

### 1. Código

| Código | Mensaje EN canónico |
|--------|---------------------|
| **E0217** | `integer overflow` |

Ops: **`+` `-` `*`** sobre `Int` (i64). **`/` `%`** = ADR-044 / E0216 (no este CUT).

### 2. Detección HIR v0

1. `Binary` `+`|`-`|`*` donde ambos lados son **LitInt** y el resultado **no cabe en i64** → E0217.
2. Morph runtime medible: `let a: Int = <i64::MAX>` (o MIN) + `a + 1` / `a - 1` / `a * 2` con operandos lit-foldables en el mismo scope v0 (Ingeniero elige profundidad mínima que cierre el hole release-wrap).
3. OUT v0: wrapping APIs intencionales; checked_add surface; overflow no-lit complejo.

### 3. Oráculos

Neg mínimo (lit):

```arita
module e0217_add_overflow
fn main() -> Io<()> {
  let x: Int = 9223372036854775807 + 1
  print("ok")
}
```

Expect: **E0217** (no wrap release, no rustc-only).

Opc. morphs `-` / `*` + runtime MAX en test (Ingeniero).

Clean: aritmética dentro de rango lit (p.ej. `1 + 1`).

### 4. OUT

- Depender de rustc E0100 / panic debug / wrap release como PASS
- E0216 diluido
- Float

## Checklist

- [x] E0217 + +/-/* + HIR-before-emit OK
- [x] GO DOC+IMPL
- [x] Neg + measure (Ingeniero) — Lex **85**+2 gated; `neg-e0217-add-overflow`; HIR before emit; morph `e0217-runtime-max-plus`

## Addendum — oráculos sub/mul measure (**verified**)

- **CUT:** `E0217-ORACLES-SUBMUL-20260915` (mismo E0217 / ADR-045; **no** ADR-047)
- **IN:** `neg-e0217-sub-overflow` + `neg-e0217-mul-overflow` wired en measure
- **Lex:** **87 accepted** + **2 gated inconclusive** (89 oracles); overall `accepted`
- **OUT:** nuevo E0xxx; ampliar gate más allá lit-foldable — still OUT
- **Post:** **PARK crates** + Investigador PARK research hasta surface nueva (Mutex/Result/…)
