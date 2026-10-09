Translation of `045-e0217-int-overflow.md`; the original is normative. / Traducción de `045-e0217-int-overflow.md`; el original es el normativo.

# ADR-045 — E0217 integer overflow (`+` `-` `*`)

- **Estado:** **aceptada** + **verified** (Lex measure **87** accepted + **2** gated (add+sub+mul oracles); **STABLE_VERIFY**)
- **CUT-ID:** `TRAPS-INT-OVERFLOW-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL; sondeo HOLE)
- **Relacionados:** ADR-006 (Int binary), ADR-044 / E0216 (div0), ADR-022 safe-only
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Rechazo **HIR antes de emit**; no rustc E0100 / panic / wrap release.
- **Barra:** E2E measure neg; skip ≠ PASS.

## Context (sondeo)

| Axis | Resultado |
|------|-----------|
| Expressible | YES: `MAX+1` / `a+1` with `a=MAX` (LitInt i64) |
| HIR gate | **NO** (only E0216 div0) |
| Emit | plain `+` (no checked) |
| Const | rustc E0100 |
| Runtime debug | panic |
| Runtime **release** | **silent wrap** → `i64::MIN` |
| Same class | Sub / Mul |

## Decision (pins GO)

### 1. Code

| Code | Message EN canonical |
|--------|---------------------|
| **E0217** | `integer overflow` |

Ops: **`+` `-` `*`** sobre `Int` (i64). **`/` `%`** = ADR-044 / E0216 (no this CUT).

### 2. HIR detection v0

1. `Binary` `+`|`-`|`*` where both lados son **LitInt** y el resultado **no cabe en i64** → E0217.
2. Morph runtime measurable: `let a: Int = <i64::MAX>` (o MIN) + `a + 1` / `a - 1` / `a * 2` with operandos lit-foldables in the same scope v0 (Ingeniero elige minimum depth que closes the hole release-wrap).
3. OUT v0: wrapping APIs intencionales; checked_add surface; overflow no-lit complejo.

### 3. Oracles

Neg minimum (lit):

```arita
module e0217_add_overflow
fn main() -> Io<()> {
  let x: Int = 9223372036854775807 + 1
  print("ok")
}
```

Expect: **E0217** (no wrap release, no rustc-only).

Opc. morphs `-` / `*` + runtime MAX in test (Ingeniero).

Clean: arithmetic inside lit range (p.ej. `1 + 1`).

### 4. OUT

- Depender de rustc E0100 / panic debug / wrap release as PASS
- E0216 diluido
- Float

## Checklist

- [x] E0217 + +/-/* + HIR-before-emit OK
- [x] GO DOC+IMPL
- [x] Neg + measure (Ingeniero) — Lex **85**+2 gated; `neg-e0217-add-overflow`; HIR before emit; morph `e0217-runtime-max-plus`

## Addendum — oracles sub/mul measure (**verified**)

- **CUT:** `E0217-ORACLES-SUBMUL-20260915` (mismo E0217 / ADR-045; **no** ADR-047)
- **IN:** `neg-e0217-sub-overflow` + `neg-e0217-mul-overflow` wired en measure
- **Lex:** **87 accepted** + **2 gated inconclusive** (89 oracles); overall `accepted`
- **OUT:** new E0xxx; widen gate beyond lit-foldable — still OUT
- **Post:** **PARK crates** + Investigator PARK research until surface nueva (Mutex/Result/…)
