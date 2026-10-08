# ADR-025 — `match` as expression (extends ADR-015)

- **Estado:** **aceptada** (DOC + IMPL CUT `MATCH-AS-EXPR-20260913`)
- **CUT-ID:** `MATCH-AS-EXPR-20260913`
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (draft) · ARITA Arquitecto (**GO** / pins)
- **Relacionados:** ADR-015 (match stmt), ADR-014, ADR-007, ADR-006, ADR-022, ADR-023
- **Gobernanza:** DOC aceptada. **IMPL** solo con CUT+GO explícito (pedido <person>: tras oráculos F3, salvo GO <person> que adelanto). Sin ejemplos decorativos. Safe-only (ADR-022).

## Contexto

ADR-015 congela `match` como **statement** (`pat => { stmts }`). Hace falta `match` que **produzca valor** (p.ej. init de `let`, return de `fn`).

## Pins GO (`MATCH-AS-EXPR-20260913`)

1. **`match` es Expr**; tipo = unificación de tipos de brazos.
2. **Stmt form ADR-015 sigue válida** (no se elimina).
3. **Scrutinee / pats / exhaustividad = ADR-015** (Bool|Int; LitBool|LitInt|`_`; E0221/E0222/E0223).
4. **Arm body v0:**
   - `pat => expr` (Lit / Path / Call / Binary / `match` anidado), **o**
   - `pat => { stmts }` tipando **`()`** (compat 015), **o**
   - `pat => { stmts; tail_expr }` (block-as-expr **solo dentro de brazo match**; no habilita block-expr genérico en F2).
5. **Tipos de valor v0 (brazo):** **`Bool` | `Int`** únicamente. **OUT** valor `String` / `Vec` / `Io<()>` en expr match (evita join de moves entre brazos en v0).
6. **Mismatch entre brazos** → **E0203** (`type mismatch`). Sin código E022x nuevo.
7. **OUT:** guards, or-patterns, bindings, enums/structs, quitar stmt match.
8. **Emit:** Rust `match` expresión; `#![forbid(unsafe_code)]` en usuario (ADR-022).

## Forma canónica

```arita
module demo

fn classify(n: Int) -> Int {
  match n {
    0 => 0,
    1 => 1,
    _ => 2,
  }
}

fn main() -> Io<()> {
  let x: Int = classify(1)
  let y: Int = match true {
    true => { 1 }
    false => { 0 }
  }
  match y {
    1 => { print("one") }
    _ => { print("other") }
  }
}
```

## Oráculos (solo tras IMPL GO)

| Id | Path | stdout (pin GO IMPL) |
|----|------|----------------------|
| `f22-04-match-expr-bool` | `ejemplos/f2.2/04-match-expr-bool.arita` | `1` |
| `f22-05-match-expr-int` | `ejemplos/f2.2/05-match-expr-int.arita` | `10` |

Neg: brazos `Int` vs `Bool` → **E0203**. Stmt 01–03 siguen verdes.

## AST (referencia)

```text
Expr::Match { scrutinee, arms }
MatchArm { pat, body: Expr }   # Block | other
# Stmt::Match 015 = uso en posición stmt / o Stmt::Expr(Match) con tipo ()
```

## Consecuencias

- Parser/Codegen: **DOC GO** aquí; **no** empezar IMPL hasta CUT IMPL (post F3 oráculos salvo override <person>).
- ADR-015: línea OUT “match as expression” → **superseded por este ADR** (stmt intacto).
- ADR-023 OUT “match sobre cmp como expr”: queda **habilitado** cuando scrutinee es Bool (p.ej. resultado de cmp).
- PACK-F2.2: ampliar few-shots expr solo tras oráculos E2E reales.
- IMPL GO (<person>): HOLD lifted post F3 05–10; MVP arm body `{ value }`.

## Checklist cerrado (Arquitecto)

- [x] Expr + stmt coexisten
- [x] Pats/exhaustividad = 015 (E0221–E0223)
- [x] Valor brazo v0 = Bool|Int; E0203 mismatch
- [x] Block-as-expr **solo** en brazo match
- [x] Sin E0xxx nuevo; safe-only
- [x] IMPL HOLD post F3 (o GO <person>)

## Anti-theater (addendum)

Brazos idénticos / constante morph → **E0225** (`DOC/ADR/031-e0225-vacuous-match.md`, CUT `TRAPS-MATCH-VACUOUS-20260914`).
