Translation of `025-match-as-expr.md`; the original is normative. / Traducción de `025-match-as-expr.md`; el original es el normativo.

# ADR-025 — `match` as expression (extends ADR-015)

- **Estado:** **aceptada** (DOC + IMPL CUT `MATCH-AS-EXPR-20260913`)
- **CUT-ID:** `MATCH-AS-EXPR-20260913`
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (draft) · ARITA Arquitecto (**GO** / pins)
- **Relacionados:** ADR-015 (match stmt), ADR-014, ADR-007, ADR-006, ADR-022, ADR-023
- **Gobernanza:** DOC aceptada. **IMPL** solo con CUT+GO explícito (pedido <person>: tras oráculos F3, salvo GO <person> que adelanto). Sin ejemplos decorativos. Safe-only (ADR-022).

## Context

ADR-015 freezes `match` as **statement** (`pat => { stmts }`). Need `match` que **produzca valor** (p.ej. init de `let`, return de `fn`).

## Pins GO (`MATCH-AS-EXPR-20260913`)

1. **`match` es Expr**; type = unification of arm types.
2. **Stmt form ADR-015 remains valid** (no se elimina).
3. **Scrutinee / pats / exhaustiveness = ADR-015** (Bool|Int; LitBool|LitInt|`_`; E0221/E0222/E0223).
4. **Arm body v0:**
   - `pat => expr` (Lit / Path / Call / Binary / `match` anidado), **o**
   - `pat => { stmts }` tipando **`()`** (compat 015), **o**
   - `pat => { stmts; tail_expr }` (block-as-expr **only inside a match arm**; does not enable generic block-expr en F2).
5. **Tipos de valor v0 (arm):** **`Bool` | `Int`** only. **OUT** valor `String` / `Vec` / `Io<()>` en expr match (evita join de moves entre arms en v0).
6. **Mismatch across arms** → **E0203** (`type mismatch`). No new E022x code.
7. **OUT:** guards, or-patterns, bindings, enums/structs, quitar stmt match.
8. **Emit:** Rust `match` expression; `#![forbid(unsafe_code)]` on user code (ADR-022).

## Canonical form

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

## Oracles (only after IMPL GO)

| Id | Path | stdout (pin GO IMPL) |
|----|------|----------------------|
| `f22-04-match-expr-bool` | `ejemplos/f2.2/04-match-expr-bool.arita` | `1` |
| `f22-05-match-expr-int` | `ejemplos/f2.2/05-match-expr-int.arita` | `10` |

Neg: arms `Int` vs `Bool` → **E0203**. Stmt 01–03 siguen verdes.

## AST (referencia)

```text
Expr::Match { scrutinee, arms }
MatchArm { pat, body: Expr }   # Block | other
# Stmt::Match 015 = use in stmt position / or Stmt::Expr(Match) with type ()
```

## Consequences

- Parser/Codegen: **DOC GO** here; **do not** start IMPL until CUT IMPL (after F3 oracles unless <person> override).
- ADR-015: OUT line “match as expression” → **superseded by this ADR** (stmt intact).
- ADR-023 OUT “match sobre cmp as expr”: queda **habilitado** when scrutinee es Bool (p.ej. resultado de cmp).
- PACK-F2.2: ampliar few-shots expr only after oracles E2E reales.
- IMPL GO (<person>): HOLD lifted post F3 05–10; MVP arm body `{ value }`.

## Checklist cerrado (Arquitecto)

- [x] Expr + stmt coexisten
- [x] Pats/exhaustiveness = 015 (E0221–E0223)
- [x] Valor arm v0 = Bool|Int; E0203 mismatch
- [x] Block-as-expr **only** en arm match
- [x] No new E0xxx; safe-only
- [x] IMPL HOLD post F3 (o GO <person>)

## Anti-theater (addendum)

Brazos identical / constante morph → **E0225** (`DOC/ADR/031-e0225-vacuous-match.md`, CUT `TRAPS-MATCH-VACUOUS-20260914`).
