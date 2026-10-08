# ADR-015 — F2.2 `match` (stmt MVP)

- **Estado:** **aceptada**
- **CUT-ID:** `F2.2-MATCH-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO)
- **Relacionados:** ADR-006 (F2 surface; match deferred), ADR-014 (F2.1 if/while), ADR-007 (F2 AST)
- **Alcance:** Surface + AST/HIR/codegen for stmt-form `match` on Bool|Int scrutinee; LitBool|LitInt|`_` patterns; exhaustiveness.
- **Barra:** Real E2E parse → HIR check → emit Rust → rustc → execute; skip ≠ PASS; inconclusive ≠ accepted.

## Contexto

ADR-014 deferred `match`. F2.1 delivered `if` / `while` with Bool conditions. Branching on discrete values needs pattern match without full Rust expressiveness.

## Decisión

### 1. MVP surface (IN)

```arita
module demo

fn main() -> Io<()> {
  let flag: Bool = true
  match flag {
    true => { print("yes") }
    false => { print("no") }
  }

  let n: Int = 1
  match n {
    1 => { print("a") }
    _ => { print("b") }
  }
}
```

| Constructo | Forma |
|------------|--------|
| `match` | `match Scrutinee { arm+ }` as **statement** (not expr value) |
| Scrutinee | `LitBool` \| `LitInt` \| `Path` typed **Bool** or **Int** |
| Pattern | `LitBool` \| `LitInt` \| `_` (wildcard) |
| Arm | `pat => { stmts }` |
| Exhaustive Bool | both `true` and `false` arms, **or** a `_` arm |
| Exhaustive Int | **must** include a `_` arm (no full Int exhaustiveness) |

### 2. OUT (deferred)

- guards (`if` on arms)
- or-patterns (`a \| b`)
- binding patterns (`x`, `mut x`)
- enums / structs / destructuring
- ~~`match` as expression value~~ — **abierto en ADR-025** (`MATCH-AS-EXPR-20260913`); stmt MVP de este ADR **sigue**
- `break` / `continue`
- nested compound Bool in scrutinee beyond Path/lit

### 3. Diagnostics

| Código | Cuándo |
|--------|--------|
| **E0221** | non-exhaustive Bool `match` (missing true or false, and no `_`) |
| **E0222** | Int `match` without a `_` arm |
| **E0223** | pattern type ≠ scrutinee type, or scrutinee type ≠ Bool\|Int |
| **E0225** | `vacuous match arms` | identical constant arms — **ADR-031** |

Ownership E020x, anti-theater E021x, and F2.1 **E0220** unchanged.

### 4. Pipeline

```text
.arita → pest parse (Stmt::Match { scrutinee, arms: Vec<MatchArm> })
      → HIR lower + check (scrutinee Bool|Int; pat types; exhaustiveness)
      → emit-Rust `match … { pat => { … } … }`
      → rustc → execute (measure oracles)
```

### 5. Oracles

- `ejemplos/f2.2/01-match-bool.arita` — Bool true/false arms
- `ejemplos/f2.2/02-match-int.arita` — Int lit + `_`
- Optional: `ejemplos/f2.2/03-match-bool-wild.arita` — Bool with `_`
- Neg: `ejemplos/f2.2/neg/e0221-bool-nonex.arita` → E0221
- Neg: `ejemplos/f2.2/neg/e0222-int-no-wild.arita` → E0222

## Consecuencias

- F1.1 / F2 / F2.1 programas siguen válidos.
- `match` sale de `illegal_stmt` (pest) y pasa a IN.
- Clippy measure packages: `arita-syntax`, `arita-codegen`, `arita-hir`, `arita-logic`, `arita-cli` con `-D warnings`.

## Gaps vs ADR-014 (cerrados / abiertos)

| # | Tema | Estado |
|---|------|--------|
| G0 | Path ADR-014 | **Cerrado** — `DOC/ADR/014-f21-control-flow.md` |
| G1 | `match` vs `if` forma | **Cerrado** — F2.1 = stmt `if`/`while`; F2.2 = **stmt** `match` (alineado; no expr) |
| G2 | Cond Bool | **Cerrado** — F2.1 E0220; match usa E0221–E0223 (sin colisión) |
| G3 | Scopes brazo | **Cerrado v0** — brazo = `{ stmts }` como bloque F2.1 |
| G4 | ADR-006/007 pin textual `match`→F2.1 | **Cerrado DOC** — 006/007 anotados while=F2.1 / match=F2.2 done |
| G5 | Oráculos / measure | SoT: `ejemplos/f2.2/` + neg E0221/22/23 en measure; **PACK-F2.2 final** (GO Ingeniero). Measure Lex E2E = <person> |

Scopes/join no bloqueantes → ADR-016 notes (opcional).
**Arquitecto:** no redefinir E022x ni reescribir este ADR sin CUT nuevo.

