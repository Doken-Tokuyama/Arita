Translation of `004-f1-ast-freeze.md`; the original is normative. / Traducción de `004-f1-ast-freeze.md`; el original es el normativo.

# ADR-004 — AST F1 congelado (pre-HIR)

- **Estado:** aceptada
- **CUT-ID:** `F1-AST-RICH-20260913` (**manda**; supersede borradores `main_prints`)
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (corte) + ARITA Arquitecto (DOC)
- **Relacionados:** ADR-001, ADR-003
- **Gobernanza:** cambio de AST solo con **GO** + nuevo CUT-ID. Parches DOC a `main_prints` posteriores a este corte = **VOID**.

## Decision (CUT-ID F1-AST-RICH-20260913)

1. AST canonical F1 (post-parse, until HIR):

```text
Module { name, functions: [Function] }
Function { name, body: [Stmt] }
Stmt::Expr(Expr)
Expr::Call(Call) | Expr::LitStr(String)
Call { callee, args: [Expr] }
```

2. Spike hello: `print("…")` → `Call { callee: "print", args: [LitStr(…)] }` en `fn main`.
3. **`main_prints` = historical only** (no freeze). No reintroduce en DOC/code as contrato without GO + CUT-ID nuevo.
4. **Emit:** camina el AST rico (`Call`/`LitStr` → `println!`). Congelado en comportamiento F1 until HIR + GO.

## Historia (no normativa)

There were conflicting cuts (`main_prints` vs rich). **This CUT-ID voids** the `main_prints` freeze in DOC.

## Consequences

- Rich sketch in `03` (spec/test/add) = language direction beyond the F1 subset.
- Arquitecto: **no escribe code** de crates; only DOC.
- Next evolution = HIR / new ADR with GO.

## Extension F2

ADR-004 remains el **subset F1** in force (`F1-AST-RICH-20260913`). The F2 node extension F2 is in **ADR-007** / CUT `F2-AST-20260913` (extends; no replaces).

## Enlaces

- `crates/arita-syntax`, `crates/arita-codegen`
- `DOC/02-ARCHITECTURE.md`, `DOC/03-LANGUAGE-SKETCH.md`
