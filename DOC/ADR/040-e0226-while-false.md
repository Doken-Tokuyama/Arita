# ADR-040 — E0226 vacuous `while false` (theater)

- **Estado:** **aceptada** + **verified** (Lex measure **81** accepted + **2** gated)
- **CUT-ID:** `TRAPS-WHILE-FALSE-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-014 (`while`), ADR-031/E0225 (vacuous match), ADR-039 (deferred 039b → este ADR)
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Número **040** (no 039b).
- **Barra:** E2E measure neg; skip ≠ PASS. Si hoy PASS → hueco; reject ARITA antes de “ok” theater.

## Contexto

Loop con condición **literal `false`**: cuerpo inalcanzable. Theater típico IA (“cubrí el branch”) + `print` posterior que “pasa”. Familia control-flow (junto E0225), no E021x assert.

## Decisión (pins GO)

### 1. Código

| Código | Mensaje EN canónico |
|--------|---------------------|
| **E0226** | `vacuous while false` |

No diluir E0220 (non-Bool cond) ni E0225 (match arms).

### 2. Patrón v0

`while false { … }` (condición = LitBool `false` únicamente).

OUT v0: `while <expr>` que “resulta” false en const-eval complejo; `while true` infinito; `if false` → **ADR-041**/E0227.

### 3. Oráculo

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

Expect: **E0226** (no stdout `ok` como PASS).

### 4. OUT

- `if false` → **ADR-041** / E0227
- Const-eval general
- Mezclar con E0242 / E0215

## Checklist

- [x] E0226 + EN + CUT OK
- [x] GO DOC+IMPL (ADR-040, no 039b)
- [x] Neg + measure (Ingeniero) — `neg-e0226-while-false` → NEG_ORACLES; Lex **81**+2 gated
