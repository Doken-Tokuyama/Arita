# ADR-260 — Core 0.5 slice 1: INSERT-FALLIBLE

- **Estado:** **CLOSED** Lex **688/688** (2026-09-20 ~17:37 CEST) · gate `DOC/GATE-CORE05-INSERT-20260920.md`
- **CUT-ID:** `CORE-0.5-INSERT-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-259](259-core-0.5-pins.md) §0.1 slice 1 · contrato [ADR-228](228-lang-contract-insert.md)
- **Prev:** Core **0.4 CLOSED** Lex **683/683**
- **Unpark (acotado):** método `insert` (antes E0206 park) — **solo** semántica Result; **no** unpark Mutex/`[]` sugar (sugar = slice 2)
- **HOLD:** Mutex · idle-kill · TLS/WS · crates.io · repair · String.insert · drain/closures

## Objetivo

Whitelist `Vec.insert` fallible según ADR-228: `Result<(), Int>`, E0311 lit negativo, emit **sin** panic.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Signature** | `fn insert(self: mut Vec<T>, i: Int, x: T) -> Result<(), Int>` |
| **OK** | `0 ≤ i ≤ len` → insert + `Ok(())` |
| **Err runtime** | `i > len` o `i < 0` no-lit → `Err(0)` (código v0 pin 228) |
| **Compile** | lit `i < 0` → **E0311** `negative insert index` |
| **Emit** | helper `__arita_vec_insert` (o equiv.) — **nunca** `Vec::insert` bare |
| **OUT** | `insert` → `()` · String.insert · IndexMut |

## 1. Surface ejemplo

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
| `core05-insert-ok` | insert in-range → Ok + orden esperado |
| `core05-insert-err-oob` | i > len → Err(0); vec intacto |
| `neg-core05-insert-neg-lit` | lit i < 0 → **E0311** |
| `core05-insert-emit-ban` | emit grep: cero `].insert(` panic path / bare `Vec::insert` sin check |
| `neg-core05-insert-was-e0206` | (histórico) park levantado solo para signature pinada |

## 3. Criterio CLOSED

Lex §2 verde; tick ADR-259 slice 1; HOLDs Mutex/`[]`-sugar intactos. Siguiente: slice 2 INDEX-SUGAR.

## Checklist

- [x] Pins insert Result + E0311 + oracles
- [x] IMPL + Lex **688/688** CLOSED
- [x] Slice 2 GO INDEX-SUGAR

## Cierre

- **GO Ingeniero** 2026-09-20 · gate [`DOC/GATE-CORE05-INSERT-20260920.md`](../GATE-CORE05-INSERT-20260920.md)
- Lex measure **688/688** accepted (delta 683→688; +5 oracles insert)
- Clippy workspace OK; Veyra companion REJECTED (bytes RUSTSEC) **non-blocking**
- Siguiente: ADR-259 GO slice 2 INDEX-SUGAR (`CORE-0.5-INDEX-SUGAR-20260920`)
- HOLDs intactos · Core 0.5 still open
