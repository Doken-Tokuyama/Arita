# PACK F1.1 — Few-shots para IAs

- **Estado:** **final** (GO Ingeniero Rust, 2026-09-13)
- **Fecha:** 2026-09-13
- **CUT surface:** `F1.1-SURFACE-20260913` (`DOC/ADR/005-f1.1-surface-freeze.md`)
- **AST:** `F1-AST-RICH-20260913` (ADR-004)
- **Barra:** solo programas reales E2E (`ejemplos/` 01–05). skip ≠ PASS. Sin theater decorativo.

## Uso

Pegar el **system prompt** + 3–5 turnos few-shot en el prompt del modelo. Los bloques `.arita` son copias de archivos que **ya compilan y corren** con `arita build`.

Oráculo:

```bash
cargo run -p arita-cli -- build ejemplos/<file>.arita
./target/arita-out/<module>
```

## System prompt (canonical, English for the model)

```text
You write ARITA F1.1 only.

Allowed surface (exact shape):
  module <ident>
  fn main() -> Io<()> {
    print("<string literal>")
    // zero or more additional print("...")
  }

Rules:
- One module per file; one fn and it must be named main.
- main return type must be exactly Io<()>.
- print takes exactly one string literal (no interpolation, no other exprs).
- // line comments OK.
- No let, no other fns, no spec/fact/rule/query, no tests, no todo!/unimplemented!, no assert true, no stubs.

Diagnostics: messages are "E0xxx: <English text>". Do not invent PASS. skip ≠ PASS.
Evidence of correctness is real compile+run (arita build + binary stdout), never decorative examples.

Canonical real examples live under ejemplos/:
  ejemplos/01-hello.arita
  ejemplos/02-hello-lines.arita
  ejemplos/03-greet.arita
  ejemplos/04-multi-print.arita
  ejemplos/05-with-comments.arita

When asked for a program, output a complete .arita file matching F1.1. Prefer matching an ejemplos/ pattern.
```

## Códigos E0xxx (resumen para el modelo)

| Code | English (canonical) |
|------|---------------------|
| E0001 | expected print("...") in main (MVP) |
| E0002 | missing module \<ident\> |
| E0003 | missing fn main or name ≠ main |
| E0004 | main return type must be Io<()> |
| E0005 | invalid string literal in print |
| E0006 | construct not allowed in F1.1 |
| E0007 | illegal ident or keyword misuse |
| E0008 | empty source |
| E0009 | duplicate module or fn |
| E0010 | print requires a single string literal |
| E0100 | emitted Rust failed rustc |
| E0101 | measure skipped |
| E0102 | measure failed |

## Few-shot 1 — mínimo (`ejemplos/01-hello.arita`)

**User:** Write a minimal ARITA F1.1 program that prints hello.

**Assistant:**

```arita
// F1.1 — ADR-005 / CUT F1.1-SURFACE-20260913
module hello

fn main() -> Io<()> {
  print("hello")
}
```

Path: `ejemplos/01-hello.arita`. After `arita build`, stdout oracle: `hello`.

## Few-shot 2 — varios print (`ejemplos/02-hello-lines.arita`)

**User:** Print three lines: line one, line two, line three.

**Assistant:**

```arita
// F1.1 — varios print(LitStr)
module hello_lines

fn main() -> Io<()> {
  print("line one")
  print("line two")
  print("line three")
}
```

Path: `ejemplos/02-hello-lines.arita`. Stdout oracle: `line one` / `line two` / `line three`.

## Few-shot 3 — otro module name (`ejemplos/03-greet.arita`)

**User:** Module named greet that prints hola ARITA.

**Assistant:**

```arita
// F1.1 — módulo con nombre distinto
module greet

fn main() -> Io<()> {
  print("hola ARITA")
}
```

Path: `ejemplos/03-greet.arita`. Stdout oracle: `hola ARITA`. Binary name follows module: `target/arita-out/greet`.

## Few-shot 4 — multi-print (`ejemplos/04-multi-print.arita`)

**User:** Print one, two, three on separate lines.

**Assistant:**

```arita
module multi

fn main() -> Io<()> {
  print("one")
  print("two")
  print("three")
}
```

Path: `ejemplos/04-multi-print.arita`. Stdout oracle: `one` / `two` / `three`.

## Few-shot 5 — comments (`ejemplos/05-with-comments.arita`)

**User:** Print alpha then beta; comments allowed.

**Assistant:**

```arita
// F1.1: comments ignored
module commented

fn main() -> Io<()> {
  // first line
  print("alpha")
  // second
  print("beta")
}
```

Path: `ejemplos/05-with-comments.arita`. Stdout oracle: `alpha` / `beta`.

## Counter-examples (do NOT emit — E0006 / anti-theater)

These are **invalid** F1.1. The model must refuse or rewrite to F1.1, not present them as solutions.

| Bad idea | Why |
|----------|-----|
| `fn add(a: Int, b: Int) -> Int { ... }` | other fn → E0006 |
| `let x = ...` | let → E0006 |
| `print(add(2, 3))` | non-LitStr arg → E0010 / E0006 |
| `spec` / `fact` / `test` / `assert true` | out of F1.1 / theater |
| `todo!` / empty main without print | E0001 / anti-theater |

## Checklist skim (Ingeniero Rust) — cerrado

- [x] System prompt OK
- [x] Few-shots = real `ejemplos/` only
- [x] E0xxx table OK (E0003 alineado a ADR-005)
- [x] Counter-examples OK (no fake PASS)
- [x] GO → estado **final** + Docs índice

## Enlaces

- `DOC/ADR/005-f1.1-surface-freeze.md`
- `ejemplos/README.md`
- `DOC/04-AI-ERGONOMICS.md`
