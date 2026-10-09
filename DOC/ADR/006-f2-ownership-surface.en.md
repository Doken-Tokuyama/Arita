Translation of `006-f2-ownership-surface.md`; the original is normative. / Traducción de `006-f2-ownership-surface.md`; el original es el normativo.

# ADR-006 — Surface F2 ownership

- **Estado:** **aceptada**
- **CUT-ID:** `F2-OWNERSHIP-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO)
- **Relacionados:** ADR-001, ADR-002, ADR-004 (`F1-AST-RICH-20260913`), ADR-005 (`F1.1-SURFACE-20260913`), ADR-007 (`F2-AST-20260913`)
- **Alcance:** DOC / contrato de surface F2. AST F1 = subset; AST F2 = ADR-007. Impl pest F2: `let Int` + `Vec` E2E (ver Verification).
- **Barra:** E2E compile+run oracles; skip ≠ PASS; sin smoke/fake/theater decorativo.

## Context

F1.1 freezes `module` + `fn main() -> Io<()>` + `print(LitStr)`. Phase 2 of the roadmap pide:

- move/borrow rules v0
- minimal std: print, Vec, String
- anti-theater
- pack few-shot

This ADR proposes the **F2 surface** (what an AI may write) and the language-level **ownership contract**. Implementation (HIR, checker, emit) is left to Ingeniero after GO.

## Pins GO (`F2-OWNERSHIP-20260913`)

1. `Int` = `i64`
2. Control flow OUT de F2 — **done split:** **F2.1** `if`/`while` (`DOC/ADR/014-f21-control-flow.md`) + **F2.2** `match` stmt (`DOC/ADR/015-f22-match.md`). Pin historical “→F2.1” superseded.
3. AST F1 intact until CUT AST + GO (Let/LitInt/Path/Binary/Borrow/…)
4. Anti-theater `E02xx` + std whitelist OK
5. `.arita` F2 only bajo `ejemplos/f2/` when they pass oracle E2E compile+run (not decorative)

## Decision (proposal)

### 1. Relation to F1.1

- Every valid F1.1 program **remains** valid in F2.
- F2 **widens** surface; does not redefine F1.1.
- Diagnostics F1.1 (`E0001`–`E0010`, `E010x`) se mantienen.

### 2. F2 surface (canonical form — draft)

```arita
module demo

fn double(x: Int) -> Int {
  x + x
}

fn main() -> Io<()> {
  let s: String = "hi"
  let n: Int = double(21)
  let mut v: Vec<Int> = Vec::new()
  v.push(n)
  print(s)
  print(v.len())
}

test double_ok {
  assert double(2) == 4
}
```

#### Constructos IN (F2)

| Constructo | Reglas v0 |
|------------|-----------|
| `let x: T = expr` | immutable binding; move de `x` invalid after move |
| `let mut x: T = expr` | mutable binding; requires `&mut` for exclusive borrow |
| `fn name(args…) -> T` | also de `main`; body with exprs/stmts F2 |
| `fn main() -> Io<()>` | remains the entry; effects I/O only here (v0) |
| Tipos | `Int`, `Bool`, `String`, `Vec<T>` (`T` ∈ {Int, Bool, String} en v0), `Io<()>` |
| Expr | literals, `Call`, local paths, `+` on `Int`, std whitelist methods |
| `print(expr)` | `expr`: `String` \| `Int` \| LitStr (coerce); un arg |
| `test name { assert a == b }` | both lados evidentes (literals o calls pure); ver anti-theater |
| Comentarios | `//` |

#### Ownership / borrow (rules v0 — semantics)

1. **Move:** tipos `String`, `Vec<_>` move by default on assignment and pass-by-value; use after move → **E0201**.
2. **Copy:** `Int`, `Bool` son copy (no invalidan el origen).
3. **Shared borrow:** `&x` — lecturas aliasadas; no `&mut` solapado → **E0202** si conflicto.
4. **Exclusive borrow:** `&mut x` — unique; dura until last use of the borrow (simplified lexical scope, without lifetimes on the surface).
5. **No explicit lifetimes** en F2 surface (codegen/HIR materializes them en Rust emit).
6. **No `unsafe`**, no raw pointers.

Checker ownership vive en **HIR** (crate future `arita-hir`); AST F1 may necesitar **nodes nuevos** (`Let`, `LitInt`, `Path`, `Binary`, `Borrow`, …) → eso exige **CUT-ID AST + GO** aparte (no this ADR only).

### 3. Anti-theater (F2 measure / hard-error)

Forbidden en dialecto base (parse o measure → FAIL, never PASS):

| Prohibition | Code draft |
|-------------|--------------|
| `todo!` / `unimplemented!` / `todo` keyword | **E0210** |
| `assert true` / `assert(true)` / assert without evidence | **E0211** |
| vacuous `len`/`is_empty` theater (`>= 0`, `len==len`, tautology) | **E0214** (ADR-030) |
| vacuous comparison assert (`n==n`, `n<=n`, lit==lit) | **E0215** (ADR-038) |
| `test` empty o only comentarios | **E0212** |
| stubs: `fn` body empty when se exige valor | **E0213** |
| `print` de valor no observable en oracle E2E | (measure; do not invent PASS) |

`arita measure` F2: oracles **compile+run** + reglas anti-theater; **skip ≠ PASS**.

### 4. Codes `E02xx` (draft — texto EN canonical)

| Code | English message (canonical) |
|--------|----------------------------|
| **E0201** | `use of moved value` |
| **E0202** | `borrow conflict` |
| **E0203** | `type mismatch` |
| **E0204** | `unknown type or path` |
| **E0205** | `invalid let / mut binding` |
| **E0206** | `method not in F2 std whitelist` |
| **E0210** | `todo/unimplemented not allowed` |
| **E0211** | `assert requires evidence` |
| **E0212** | `empty test not allowed` |
| **E0213** | `stub function body not allowed` |
| **E0214** | `vacuous length assert` |

### 5. Std whitelist F2 (emit → Rust)

| ARITA | Emit Rust (orientativo) |
|-------|-------------------------|
| `print(x)` | `println!("{}", …)` |
| `String` lit / tipo | `String` / `to_string` |
| `Vec::new`, `push`, `len`, `is_empty` | `Vec` APIs → `len`/`is_empty` as `i64`/`bool` |
| `String.len`, `String.is_empty` | bytes `len` → `Int`; `is_empty` → `Bool` (**ADR-026**) |
| `Int` | **`i64`** (pin GO) |

No hashmap, threads, filesystem, or free macros. Expanded whitelist: **ADR-026** (`STD-MIN-20260913`).

### 6. Fuera de F2 (OUT)

- Isla `spec`/`fact`/`rule`/`query` (F3)
- Lifetimes explicit, traits, generics user-defined; `async` = **OUT de F2** (IN Phase 4b: ADR-027)
- `unsafe`, FFI, raw pointers
- Control flow — **OUT de F2, done:** F2.1 (ADR-014) + F2.2 (ADR-015 `Stmt::Match`)
- Cambiar CUT AST F1 without nuevo CUT-ID
- Evidence decorativa / smoke PASS

### 7. Ejemplos E2E F2

- F1.1 canonical: `ejemplos/01`–`05` (PACK F1.1 — **without `let`**, correcto).
- F2: only `ejemplos/f2/` with oracles E2E reales. Verified: `01`…`06` + `07-assert` (`arita test` PASS; build → `ok`).

## Consequences

- Docs indexa ADR-006 **accepted** (CUT `F2-OWNERSHIP-20260913`).
- Impl pest F2 (`let`/`Vec`) bajo Ingeniero; ADR-009 = contrato DOC aceptado; checker real only after HIR + IMPL GO.
- AST F1 (`F1-AST-RICH-20260913`) intact until CUT AST + GO.
- F2 few-shot pack (when it exists) teaches only this surface + anti-theater.

## Checklist GO (Ingeniero Rust) — cerrado

- [x] Surface §2 OK (IN/OUT)
- [x] Ownership rules v0 OK
- [x] Anti-theater + `E02xx` + std whitelist OK
- [x] `Int` = **`i64`**
- [x] Control flow split **done:** while/if=F2.1 (014); match=F2.2 (015)
- [x] AST F1 intact until CUT AST + GO
- [x] No F2 `.arita` under `ejemplos/` until E2E toolchain
- [x] GO `F2-OWNERSHIP-20260913` → accepted + Docs index

## Verification (Ingeniero Rust, 2026-09-13)

- F2 **pest v0** verified E2E: `ejemplos/f2/01-let-int.arita` → `42` / `7`.
- F2 **Vec** pest verified E2E: `ejemplos/f2/02-vec-len.arita` → `2`.
- F2 **String** verified E2E: `ejemplos/f2/03-string.arita` → `hi` (oracle `f2-03-string` en measure).
- F2 **Int binary** verified E2E: `ejemplos/f2/04-int-arith.arita` → `5` (oracle `f2-04-int-arith` en measure).
- F2 **Bool** verified E2E: `ejemplos/f2/05-bool.arita` → `true` (oracle `f2-05-bool` en measure).
- F2 **fn-call** verified E2E: `ejemplos/f2/06-fn-call.arita` → `42` (`double(21)`; oracle `f2-06-fn-call` en measure).
- Measure: F1.1×5 + F2×6 oracles (+ clippy-workspace en host).
- F2 **Test+Assert** verified E2E: `ejemplos/f2/07-assert.arita` + `arita test` → PASS (build stdout `ok`).
- Oracles negativos **E021x** → `DOC/ADR/010-e021x-negative-oracles.md` (**accepted**, CUT `E021X-NEG-20260913`); **verified real** (`ejemplos/f2/neg/` E0210–E0213; without E020x).
- Borrow/`&mut` checker: `DOC/ADR/009-f2-borrow-checker-notes.md` (**accepted DOC-only**). **NO** IMPL GO; without crates; **no** fake ownership without HIR.
- PACK F1.1 (`DOC/PACK-F1.1-FEWSHOT.md`) remains **without let** — correcto (surface F1.1).

## Enlaces

- `ejemplos/f2/01-let-int.arita` … `07-assert.arita`
- `DOC/ADR/009-f2-borrow-checker-notes.md`

- `ROADMAP.md` — Phase 2
- `DOC/03-LANGUAGE-SKETCH.md`, `DOC/04-AI-ERGONOMICS.md`
- `DOC/ADR/005-f1.1-surface-freeze.md`
