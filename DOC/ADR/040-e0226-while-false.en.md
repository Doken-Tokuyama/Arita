Translation of `040-e0226-while-false.md`; the original is normative. / Traducción de `040-e0226-while-false.md`; el original es el normativo.

# ADR-040 — E0226 vacuous `while false` (theater)

- **Estado:** **aceptada** + **verified** (Lex measure **81** accepted + **2** gated)
- **CUT-ID:** `TRAPS-WHILE-FALSE-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-014 (`while`), ADR-031/E0225 (vacuous match), ADR-039 (deferred 039b → este ADR)
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Número **040** (no 039b).
- **Barra:** E2E measure neg; skip ≠ PASS. Si hoy PASS → hueco; reject ARITA antes de “ok” theater.

## Context

Loop with condition **literal `false`**: unreachable body. Typical AI theater (“I covered the branch”) + later `print` that “passes”. Control-flow family (alongside E0225), not E021x assert.

## Decision (pins GO)

### 1. Code

| Code | Message EN canonical |
|--------|---------------------|
| **E0226** | `vacuous while false` |

No diluir E0220 (non-Bool cond) ni E0225 (match arms).

### 2. Pattern v0

`while false { … }` (condition = LitBool `false` only).

OUT v0: `while <expr>` que “resulta” false en const-eval complejo; `while true` infinito; `if false` → **ADR-041**/E0227.

### 3. Oracle

`ejemplos/f2.1/neg/e0226-while-false.arita` (o `f2/neg/` si Ingeniero unifica):

```arita
module e0226_while_false
fn main() -> Io<()> {
  while false {
    print("x")
  }
  print("ok")
}
```

Expect: **E0226** (no stdout `ok` as PASS).

### 4. OUT

- `if false` → **ADR-041** / E0227
- Const-eval general
- Mezclar with E0242 / E0215

## Checklist

- [x] E0226 + EN + CUT OK
- [x] GO DOC+IMPL (ADR-040, no 039b)
- [x] Neg + measure (Ingeniero) — `neg-e0226-while-false` → NEG_ORACLES; Lex **81**+2 gated
