# ADR-016 — F2.3 `break` / `continue` (while only)

- **Estado:** **aceptada**
- **CUT-ID:** `F2.3-BREAK-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO)
- **Relacionados:** ADR-014 (F2.1 if/while), ADR-015 (F2.2 match), ADR-007 (F2 AST)
- **Alcance:** Surface + AST/HIR/codegen for unlabeled `break` / `continue` legal only inside `while` body (incl. nested `while` / `if` / `match` under an enclosing `while`).
- **Barra:** Real E2E parse → HIR check → emit Rust → rustc → execute; skip ≠ PASS; inconclusive ≠ accepted.

## Contexto

ADR-014 deferred `break` / `continue`. F2.1 delivered `while`; early exit and iteration skip need loop-control statements without labels or values.

## Decisión

### 1. MVP surface (IN)

```arita
module demo

fn main() -> Io<()> {
  let mut i: Int = 0
  while i < 10 {
    print(i)
    i = i + 1
    if i == 2 {
      break
    }
  }

  let mut j: Int = 0
  while j < 6 {
    let rem: Int = j % 2
    if rem == 1 {
      j = j + 1
      continue
    }
    print(j)
    j = j + 1
  }
}
```

| Constructo | Forma |
|------------|--------|
| `break` | bare `break` statement (no label, no value) |
| `continue` | bare `continue` statement (no label) |
| Legal locus | inside a `while` body, including nested `if` / `match` / inner `while` under an enclosing `while` |

### 2. OUT (deferred)

- labeled `break` / `continue`
- `break` with value
- `break` / `continue` in `for` (no `for` yet)
- `break` / `continue` in `match`-only (or `if`-only / fn body) **without** an enclosing `while`
- infinite `loop`

### 3. Diagnostics

| Código | Cuándo |
|--------|--------|
| **E0224** | `break` or `continue` outside any enclosing `while` |

Ownership E020x, anti-theater E021x, F2.1 **E0220**, and F2.2 **E0221–E0223** unchanged.

### 4. Pipeline

```text
.arita → pest parse (Stmt::Break / Stmt::Continue)
      → HIR lower + check (loop-depth; E0224 if depth == 0)
      → emit-Rust `break;` / `continue;`
      → rustc → execute (measure oracles)
```

### 5. Oracles / expected stdout

- `ejemplos/f2.3/01-while-break.arita` — print `0`, `1` then `break` (stops before further iterations)
- `ejemplos/f2.3/02-while-continue.arita` — skip odd `j` via `continue`; print `0`, `2`, `4`
- Neg: `ejemplos/f2.3/neg/e0224-break-outside.arita` → **E0224** (`break` inside `if` with no enclosing `while`)

## Consecuencias

- F1.1 / F2 / F2.1 / F2.2 programas siguen válidos.
- `break` / `continue` salen de keywords reservadas y pasan a IN como stmts.
- Clippy measure packages: `arita-syntax`, `arita-codegen`, `arita-hir`, `arita-logic`, `arita-cli` con `-D warnings`.
