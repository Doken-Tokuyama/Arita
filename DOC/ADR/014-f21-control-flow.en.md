Translation of `014-f21-control-flow.md`; the original is normative. / Traducción de `014-f21-control-flow.md`; el original es el normativo.

# ADR-014 — F2.1 control flow (`if` / `while`)

- **Estado:** **aceptada**
- **CUT-ID:** `F2.1-CTRL-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO)
- **Relacionados:** ADR-006 (F2 surface; control flow deferred to F2.1), ADR-007 (F2 AST), ADR-009 (borrow notes)
- **Alcance:** Surface + AST/HIR/codegen for `if` / `while` with Bool conditions; Int comparison ops in expr position; mut assign for while counters.
- **Barra:** Real E2E parse → HIR → emit Rust → rustc → execute; skip ≠ PASS; inconclusive ≠ accepted.

## Context

ADR-006 pin: `match` / `while` = OUT → F2.1. F2 delivered `let` / Vec / Bool / assert without branching. Conditions need Bool in expr position; today `==` exists only as `Stmt::Assert` separator (not `BinOp`).

## Decision

### 1. MVP surface (IN)

```arita
module demo

fn main() -> Io<()> {
  let flag: Bool = true
  if flag {
    print("yes")
  } else {
    print("no")
  }

  let mut i: Int = 0
  while i < 3 {
    print(i)
    i = i + 1
  }
}
```

| Constructo | Forma |
|------------|--------|
| `if` | `if BoolExpr { stmts }` with optional `else { stmts }` |
| `while` | `while BoolExpr { stmts }` |
| BoolExpr | `LitBool` \| `Path` (Bool binding) \| Int comparison |
| Int compare | `(ident\|int_lit) (==\|!=\|<\|<=\|>\|>=) (ident\|int_lit)` → Bool |
| assign | `ident = expr` to an existing `let mut` place (needed for while counters) |

### 2. OUT (deferred)

- `match`
- `break` / `continue`
- `loop` (infinite)
- nested compound Bool (`&&` / `\|\|` / `!`) — later CUT
- `for`

### 3. Diagnostics

| Code | When |
|--------|--------|
| **E0220** | `if`/`while` condition type ≠ Bool |

Ownership E020x and anti-theater E021x unchanged.

### 4. Pipeline

```text
.arita → pest parse (Stmt::If / Stmt::While / BinOp cmp / Assign)
      → HIR lower + check (cond Bool; bodies; mut assign)
      → emit-Rust `if` / `else` / `while` / `x = …`
      → rustc → execute (measure oracles)
```

### 5. Oracles

- `ejemplos/f2.1/01-if-true.arita` — LitBool cond
- `ejemplos/f2.1/02-if-else.arita` — else branch
- `ejemplos/f2.1/03-while-count.arita` — cmp + mut assign
- Optional neg: `ejemplos/f2.1/neg/e0220-bad-cond.arita` → E0220

## Consequences

- F1.1 / F2 programs remain valid.
- `BinOp` gains comparison ops in expr position; `assert … == …` remains `Stmt::Assert` (not Binary Eq).
- Clippy measure packages: `arita-syntax`, `arita-codegen`, `arita-hir`, `arita-logic`, `arita-cli` with `-D warnings`.
