[Español](PACK-F2.2-FEWSHOT.md) | English

# PACK F2.2 — Few-shots for AIs (`match`)

- **Estado:** **final** (GO Ingeniero Rust + E0223; 2026-09-13)
- **Fecha:** 2026-09-13
- **CUT surface:** `F2.2-MATCH-20260913` (`DOC/ADR/015-f22-match.md`)
- **Predecesores:** F2 (`PACK-F2-FEWSHOT`); F2.1 `if`/`while` (ADR-014)
- **Barra:** solo programas reales bajo `ejemplos/f2.2/`. skip ≠ PASS. Sin theater.
- **Separado** de `PACK-F2-FEWSHOT.md` / `PACK-F2.1` (si existe). Cargar este PACK **solo** si la tarea pide `match`.

## Usage

Paste the **system prompt** + few-shots when the task asks for `match` Bool|Int.
The `.arita` blocks are copies of real oracles (measure ids `f22-*`).

```bash
# positivos (build + run)
cargo run -p arita-cli -- build ejemplos/f2.2/01-match-bool.arita
./target/arita-out/match_bool
# stdout: yes

cargo run -p arita-cli -- build ejemplos/f2.2/02-match-int.arita
# stdout: a

cargo run -p arita-cli -- build ejemplos/f2.2/03-match-bool-wild.arita
# stdout: other

# negativos (must FAIL check with code)
# e0221-bool-nonex → E0221
# e0222-int-no-wild → E0222
```

## System prompt (canonical, English for the model)

```text
You write ARITA F2.2 (extends F2 + F2.1). Surface: stmt-form match on Bool|Int.

Allowed match MVP:
  match <scrutinee> {
    <pat> => { <stmts> }
    ...
  }

Scrutinee: LitBool | LitInt | Path typed Bool or Int.
Pattern: LitBool | LitInt | _ (wildcard).
Match is a STATEMENT (not an expression value).

Exhaustiveness:
- Bool: both true and false arms, OR a _ arm.
- Int: MUST include a _ arm (no full Int exhaustiveness).

OUT (do not emit): guards, or-patterns, bindings, enums/structs, match-as-expr, break/continue.

Diagnostics:
- E0221: non-exhaustive Bool match
- E0222: Int match requires `_` arm
- E0223: pattern/scrutinee type mismatch (or scrutinee not Bool|Int)

Canonical positives:
  ejemplos/f2.2/01-match-bool.arita     → stdout yes   (oracle f22-01-match-bool)
  ejemplos/f2.2/02-match-int.arita      → stdout a     (f22-02-match-int)
  ejemplos/f2.2/03-match-bool-wild.arita → stdout other (f22-03-match-bool-wild)

Canonical negatives:
  ejemplos/f2.2/neg/e0221-bool-nonex.arita → E0221
  ejemplos/f2.2/neg/e0222-int-no-wild.arita → E0222

If the task is F2 without match, use PACK-F2. If if/while only, ADR-014 / F2.1. Never invent PASS. skip ≠ PASS.
```

## E022x codes (summary)

| Code | English (canonical) |
|------|---------------------|
| **E0221** | `non-exhaustive Bool match` |
| **E0222** | `Int match requires `_` arm` |
| **E0223** | pattern type ≠ scrutinee / scrutinee not Bool\|Int |

(F2.1 **E0220** and E020x/E021x unchanged.)

## Few-shot 1 — Bool arms (`ejemplos/f2.2/01-match-bool.arita`)

**User:** ARITA F2.2: match a Bool flag; print "yes" if true else "no".

**Assistant:**

```arita
// F2.2 — match Bool true/false arms
module match_bool

fn main() -> Io<()> {
  let flag: Bool = true
  match flag {
    true => { print("yes") }
    false => { print("no") }
  }
}
```

Path: `ejemplos/f2.2/01-match-bool.arita`. Stdout oracle: `yes` (`f22-01-match-bool`).

## Few-shot 2 — Int + wildcard (`ejemplos/f2.2/02-match-int.arita`)

**User:** match Int 1 → print "a"; everything else "b".

**Assistant:**

```arita
// F2.2 — match Int lit + wildcard
module match_int

fn main() -> Io<()> {
  let n: Int = 1
  match n {
    1 => { print("a") }
    _ => { print("b") }
  }
}
```

Path: `ejemplos/f2.2/02-match-int.arita`. Stdout oracle: `a` (`f22-02-match-int`).

## Few-shot 3 — Bool with `_` (`ejemplos/f2.2/03-match-bool-wild.arita`)

**User:** match Bool false; true → "yes", wildcard → "other".

**Assistant:**

```arita
// F2.2 — match Bool with wildcard (exhaustive via _)
module match_bool_wild

fn main() -> Io<()> {
  let flag: Bool = false
  match flag {
    true => { print("yes") }
    _ => { print("other") }
  }
}
```

Path: `ejemplos/f2.2/03-match-bool-wild.arita`. Stdout oracle: `other` (`f22-03-match-bool-wild`).

## Counter-examples (do NOT emit)

| File | Code | Why |
|------|------|-----|
| `ejemplos/f2.2/neg/e0221-bool-nonex.arita` | **E0221** | Bool missing false and no `_` |
| `ejemplos/f2.2/neg/e0222-int-no-wild.arita` | **E0222** | Int arms without `_` |

Also refuse: match-as-expr, guards, or-patterns, bindings, enums, claiming PASS without oracles.

## Checklist GO (Rust Engineer) — closed

- [x] System prompt F2.2 OK (stmt match; exhaustiveness)
- [x] Few-shots = real `ejemplos/f2.2/01`–`03` only
- [x] E0221/E0222/E0223 + neg paths OK
- [x] Counter-examples / no fake PASS OK
- [x] Separation vs PACK-F2 / F2.1 OK
- [x] GO → status **final** + Docs index

## Links

- `DOC/ADR/015-f22-match.md` (CUT `F2.2-MATCH-20260913`)
- `DOC/ADR/014-f21-control-flow.md`, `DOC/ADR/006-f2-ownership-surface.md`
- `ejemplos/f2.2/`, `ejemplos/f2.2/neg/README.md`
- `DOC/PACK-F2-FEWSHOT.md`, `DOC/PACK-F3-FEWSHOT.md`
