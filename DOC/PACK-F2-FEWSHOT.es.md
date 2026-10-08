[Original](PACK-F2-FEWSHOT.md) | Español | [English](PACK-F2-FEWSHOT.en.md)

# PACK F2 — Few-shots para IAs

- **Estado:** **final** (GO Ingeniero Rust, 2026-09-13)
- **Fecha:** 2026-09-13
- **CUT surface:** `F2-OWNERSHIP-20260913` (`DOC/ADR/006-f2-ownership-surface.md`)
- **AST:** `F2-AST-20260913` (ADR-007; extiende ADR-004)
- **Anti-theater:** CUT `E021X-NEG-20260913` (`DOC/ADR/010-e021x-negative-oracles.md`)
- **Barra:** solo programas reales E2E bajo `ejemplos/f2/`. skip ≠ PASS. Sin theater decorativo.
- **Default si la tarea es F1.1:** usar `DOC/PACK-F1.1-FEWSHOT.md` — **no** mezclar F2.

## Uso

Pegar el **system prompt** + 3–7 turnos few-shot cuando la tarea pida F2 (`let`, `Vec`, `String`, `Bool`, arith, `fn` helper, `test`/`assert`).
Los bloques `.arita` son copias de archivos que **ya** pasan oráculo E2E.

Oráculos:

```bash
# positivos (build + run)
cargo run -p arita-cli -- build ejemplos/f2/<file>.arita
./target/arita-out/<module>

# test + assert
cargo run -p arita-cli -- test ejemplos/f2/07-assert.arita

# negativos (must FAIL parse)
cargo run -p arita-cli -- parse ejemplos/f2/neg/e0210-todo.arita
# → E0210: todo/unimplemented not allowed
```

## System prompt (canonical, English for the model)

```text
You write ARITA F2 (extends F1.1). Do not invent ownership/borrow checking as done — checker is DOC-only until HIR.

Allowed surface (canonical):
  module <ident>

  fn <name>(<params>) -> <Type> { ... }   // helpers OK; valued fn needs non-empty body
  fn main() -> Io<()> { ... }

  let [mut] name: Int = <int-lit | binary | ident | call>
  let [mut] name: String = "<lit-str>"
  let [mut] name: Bool = true | false
  let [mut] name: Vec<Int> = Vec::new()
  name.push(<int-lit | ident>)
  print(<ident> | <int-lit> | <bool_lit> | <lit-str> | <ident>.len() | <call>)

  // binary in let init: (ident|int_lit) (+|-|*|/|%) (ident|int_lit)

  test <name> {
    assert <evident-expr> == <evident-expr>
  }

Rules:
- F1.1 programs remain valid.
- Int = i64. No `if`/`while` here (F2.1 / ADR-014). No `match` here (F2.2 / ADR-015). No unsafe, no lifetimes in surface.
- print: one arg. assert only inside test; both sides evident (not assert true).
- No todo!/unimplemented!/empty valued fn/empty test (E021x).
- Ownership/borrow rules exist in DOC (E020x) but are NOT enforced yet without HIR — do not claim borrow PASS.

Diagnostics: "E0xxx: <English>" or "E02xx: <English>". Do not invent PASS. skip ≠ PASS.
Evidence: real arita build+run / arita test / parse reject for neg.

Canonical positives: ejemplos/f2/01 … 07
Canonical negatives: ejemplos/f2/neg/ (E0210–E0213) — see ADR-010

When asked for F2, output a complete .arita matching an ejemplos/f2 pattern.
If the task is F1.1-only, stay in F1.1 (PACK-F1.1).
```

## Códigos útiles (resumen)

### F1.1 / toolchain (sigue vigentes)

| Code | English (canonical) |
|------|---------------------|
| E0006 | construct not allowed in F1.1 |
| E0100 | emitted Rust failed rustc |
| E0101 | measure skipped |
| E0102 | measure failed |

### F2 / anti-theater

| Code | English (canonical) |
|------|---------------------|
| E0201 | use of moved value *(DOC; not enforced without HIR)* |
| E0202 | borrow conflict *(DOC; not enforced without HIR)* |
| E0203 | type mismatch |
| E0204 | unknown type or path |
| E0205 | invalid let / mut binding |
| E0206 | method not in F2 std whitelist |
| E0210 | todo/unimplemented not allowed |
| E0211 | assert requires evidence |
| E0212 | empty test not allowed |
| E0213 | stub function body not allowed |

## Few-shot 1 — let Int (`ejemplos/f2/01-let-int.arita`)

**User:** ARITA F2: let Int and print the values 42 then 7.

**Assistant:**

```arita
// F2 pest v0 — let Int + print(Path)
module let_int

fn main() -> Io<()> {
  let n: Int = 42
  let mut m: Int = 7
  print(n)
  print(m)
}
```

Path: `ejemplos/f2/01-let-int.arita`. Stdout oracle: `42` / `7`.

## Few-shot 2 — Vec (`ejemplos/f2/02-vec-len.arita`)

**User:** Vec of Int: push two elements and print len.

**Assistant:**

```arita
// F2 Vec pest — Vec::new / push / len
module vec_len

fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  v.push(2)
  print(v.len())
}
```

Path: `ejemplos/f2/02-vec-len.arita`. Stdout oracle: `2`.

## Few-shot 3 — String (`ejemplos/f2/03-string.arita`)

**User:** let String and print it.

**Assistant:**

```arita
// F2 String pest — let String + print(Path)
module str_hi

fn main() -> Io<()> {
  let s: String = "hi"
  print(s)
}
```

Path: `ejemplos/f2/03-string.arita`. Stdout oracle: `hi`.

## Few-shot 4 — Int binary (`ejemplos/f2/04-int-arith.arita`)

**User:** Add two Ints and print the sum.

**Assistant:**

```arita
// F2 Int binary — let + arith + print(Path)
module arith

fn main() -> Io<()> {
  let a: Int = 2
  let b: Int = 3
  let c: Int = a + b
  print(c)
}
```

Path: `ejemplos/f2/04-int-arith.arita`. Stdout oracle: `5`.

## Few-shot 5 — Bool (`ejemplos/f2/05-bool.arita`)

**User:** let Bool true and print it.

**Assistant:**

```arita
// F2 Bool — let Bool + print(Path)
module flag

fn main() -> Io<()> {
  let flag: Bool = true
  print(flag)
}
```

Path: `ejemplos/f2/05-bool.arita`. Stdout oracle: `true`.

## Few-shot 6 — fn call (`ejemplos/f2/06-fn-call.arita`)

**User:** Helper fn double(Int)->Int; print double(21).

**Assistant:**

```arita
// F2 fn + Call — helper Int→Int + print(user_call)
module fn_call

fn double(x: Int) -> Int {
  x + x
}

fn main() -> Io<()> {
  print(double(21))
}
```

Path: `ejemplos/f2/06-fn-call.arita`. Stdout oracle: `42`.

## Few-shot 7 — test + assert (`ejemplos/f2/07-assert.arita`)

**User:** main prints ok; add a test that asserts 2+2==4.

**Assistant:**

```arita
// F2 test + assert — Stmt::Assert only in Test (not main)
module assert_demo

fn main() -> Io<()> {
  print("ok")
}

test t {
  assert 2 + 2 == 4
}
```

Path: `ejemplos/f2/07-assert.arita`. Build stdout: `ok`. `arita test` → PASS.

## Counter-examples — E021x (`ejemplos/f2/neg/`)

These **must FAIL** at `arita parse` with the listed code. Never present as solutions.

| File | Code | English |
|------|------|---------|
| `ejemplos/f2/neg/e0210-todo.arita` | **E0210** | `todo/unimplemented not allowed` |
| `ejemplos/f2/neg/e0211-assert-true.arita` | **E0211** | `assert requires evidence` |
| `ejemplos/f2/neg/e0212-empty-test.arita` | **E0212** | `empty test not allowed` |
| `ejemplos/f2/neg/e0213-empty-fn.arita` | **E0213** | `stub function body not allowed` |

Also refuse: `if`/`while` (use F2.1), `match` (use F2.2 / PACK-F2.2), `unsafe`, claiming borrow-check PASS without HIR, decorative PASS without oracles.

## Checklist GO (Ingeniero Rust) — cerrado

- [x] System prompt F2 OK
- [x] Few-shots = real `ejemplos/f2/01`–`07` only
- [x] E02xx + E021x table OK; pointer to `neg/` OK
- [x] Counter-examples / no fake PASS OK
- [x] Separación clara vs PACK F1.1 OK
- [x] GO → estado **final** + Docs índice

## Enlaces

- `DOC/ADR/006-f2-ownership-surface.md`, `DOC/ADR/007-f2-ast-nodes.md`
- `DOC/ADR/010-e021x-negative-oracles.md`, `DOC/ADR/009-f2-borrow-checker-notes.md`
- `ejemplos/f2/`, `ejemplos/f2/neg/README.md`
- `DOC/PACK-F1.1-FEWSHOT.md` (default si no hace falta F2)
- `DOC/PACK-F2.2-FEWSHOT.md` (`match`; F2.2)
- `DOC/09-AI-PROGRAMMING.md`
