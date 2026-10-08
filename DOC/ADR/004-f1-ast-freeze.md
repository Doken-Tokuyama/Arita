# ADR-004 — AST F1 congelado (pre-HIR)

- **Estado:** aceptada
- **CUT-ID:** `F1-AST-RICH-20260913` (**manda**; supersede borradores `main_prints`)
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (corte) + ARITA Arquitecto (DOC)
- **Relacionados:** ADR-001, ADR-003
- **Gobernanza:** cambio de AST solo con **GO** + nuevo CUT-ID. Parches DOC a `main_prints` posteriores a este corte = **VOID**.

## Decisión (CUT-ID F1-AST-RICH-20260913)

1. AST canónico F1 (post-parse, hasta HIR):

```text
Module { name, functions: [Function] }
Function { name, body: [Stmt] }
Stmt::Expr(Expr)
Expr::Call(Call) | Expr::LitStr(String)
Call { callee, args: [Expr] }
```

2. Spike hello: `print("…")` → `Call { callee: "print", args: [LitStr(…)] }` en `fn main`.
3. **`main_prints` = histórico solo** (no freeze). No reintroducir en DOC/código como contrato sin GO + CUT-ID nuevo.
4. **Emit:** camina el AST rico (`Call`/`LitStr` → `println!`). Congelado en comportamiento F1 hasta HIR + GO.

## Historia (no normativa)

Hubo cortes conflictivos (`main_prints` vs rich). **Este CUT-ID anula** el freeze `main_prints` en DOC.

## Consecuencias

- Boceto rico en `03` (spec/test/add) = dirección de lenguaje más allá del subset F1.
- Arquitecto: **no escribe código** de crates; solo DOC.
- Siguiente evolución = HIR / ADR nuevo con GO.

## Extensión F2

ADR-004 permanece el **subset F1** vigente (`F1-AST-RICH-20260913`). La extensión de nodos F2 está en **ADR-007** / CUT `F2-AST-20260913` (extiende; no reemplaza).

## Enlaces

- `crates/arita-syntax`, `crates/arita-codegen`
- `DOC/02-ARCHITECTURE.md`, `DOC/03-LANGUAGE-SKETCH.md`
