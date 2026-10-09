Translation of `260-core-insert-fallible-v0.md`; the original is normative. / Traducción de `260-core-insert-fallible-v0.md`; el original es el normativo.

# ADR-260 — Core 0.5 slice 1: INSERT-FALLIBLE

- **Estado:** **CLOSED** Lex **688/688** (2026-09-20 ~17:37 CEST) · gate `DOC/GATE-CORE05-INSERT-20260920.md`
- **CUT-ID:** `CORE-0.5-INSERT-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-259](259-core-0.5-pins.md) §0.1 slice 1 · contrato [ADR-228](228-lang-contract-insert.md)
- **Prev:** Core **0.4 CLOSED** Lex **683/683**
- **Unpark (acotado):** `insert` method (formerly E0206 park) — **only** Result semantics; **do not** unpark Mutex/`[]` sugar (sugar = slice 2)
- **HOLD:** Mutex · idle-kill · TLS/WS · crates.io · repair · String.insert · drain/closures

## Objective

Whitelist `Vec.insert` fallible per ADR-228: `Result<(), Int>`, E0311 negative lit, emit **without** panic.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Signature** | `fn insert(self: mut Vec<T>, i: Int, x: T) -> Result<(), Int>` |
| **OK** | `0 ≤ i ≤ len` → insert + `Ok(())` |
| **Err runtime** | `i > len` o `i < 0` no-lit → `Err(0)` (v0 code pin 228) |
| **Compile** | lit `i < 0` → **E0311** `negative insert index` |
| **Emit** | helper `__arita_vec_insert` (or equiv.) — **never** `Vec::insert` bare |
| **OUT** | `insert` → `()` · String.insert · IndexMut |

## 1. Surface example

```text
let mut v = Vec.new()
v.push(1)
match v.insert(1, 9) { Ok(()) => ..., Err(_) => ... }
match v.insert(99, 0) { Ok(()) => ..., Err(0) => ... }  // OOB
// v.insert(-1, 0) → E0311
```

## 2. Oracles

| Id | Expect |
|----|--------|
| `core05-insert-ok` | insert in-range → Ok + expected order |
| `core05-insert-err-oob` | i > len → Err(0); vec intact |
| `neg-core05-insert-neg-lit` | lit i < 0 → **E0311** |
| `core05-insert-emit-ban` | emit grep: zero `].insert(` panic path / bare `Vec::insert` without check |
| `neg-core05-insert-was-e0206` | (historic) park lifted only for the pinned signature |

## 3. CLOSED criterion

Lex §2 green; tick ADR-259 slice 1; HOLDs Mutex/`[]`-sugar intact. Next: slice 2 INDEX-SUGAR.

## Checklist

- [x] Pins insert Result + E0311 + oracles
- [x] IMPL + Lex **688/688** CLOSED
- [x] Slice 2 GO INDEX-SUGAR

## Close

- **GO Ingeniero** 2026-09-20 · gate `DOC/GATE-CORE05-INSERT-20260920.md` (not in the public export / no incluido en el export público)
- Lex measure **688/688** accepted (delta 683→688; +5 oracles insert)
- Clippy workspace OK; Veyra companion REJECTED (bytes RUSTSEC) **non-blocking**
- Next: ADR-259 GO slice 2 INDEX-SUGAR (`CORE-0.5-INDEX-SUGAR-20260920`)
- HOLDs intact · Core 0.5 still open
