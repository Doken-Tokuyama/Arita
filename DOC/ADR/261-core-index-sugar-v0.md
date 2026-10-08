# ADR-261 — Core 0.5 slice 2: INDEX-SUGAR

- **Estado:** **CLOSED** Lex **705/705** (2026-09-25 ~20:53 CEST) · gate `DOC/GATE-CORE05-INDEX-SUGAR-20260925.md`
- **CUT-ID:** `CORE-0.5-INDEX-SUGAR-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-259](259-core-0.5-pins.md) §0.1 slice 2 · contrato [ADR-227](227-lang-contract-indexget.md) **Fase B**
- **Prev:** slice 1 INSERT **CLOSED** Lex **688/688** (ADR-260) — este CUT no reabre 260
- **Prereq evidencia:** R4 Fase A **CLOSED** Lex **583/583** (E0310) — GO Arquitecto Fase B = este ADR
- **HOLD:** IndexMut · `get_mut` · slices · Mutex · idle · TLS/WS · crates.io · repair · String.insert

## Objetivo

Sugar `v[i]` / `s[i]` → **misma** semántica que `get` → `Option<T>`. Emit idéntico a `get`. **Prohibido** `Index`/`IndexMut` panic.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Desugar** | `v[i]` / `s[i]` ⇒ `v.get(i)` / `s.get(i)` → `Option` |
| **OOB / i<0** | `None` (igual que get) |
| **Emit** | solo path `get` / Option — **cero** `values[i]`, `Index`, `IndexMut`, `unwrap` en index |
| **E0310** | **retirado** para `Vec`/`String` index sugar; sigue OUT para tipos sin `get` |
| **OUT** | `v[i] = x` (IndexMut) · slice syntax · Mutex |

## 1. Surface

```text
let v = vec_of_ints()   // o construcción ya IN
match v[0] { Some(x) => print(x), None => print(0) }
match v[999] { Some(_) => ..., None => ... }  // None, no panic
// v[0] = 1  → reject (IndexMut OUT) — diag estable (sugerido E0314)
```

`get(i)` permanece API canónica (ADR-085/227); sugar es azúcar, no segunda semántica.

## 2. Oracles

| Id | Expect |
|----|--------|
| `core05-index-sugar-some` | `v[i]` in-range → `Some` (= get) |
| `core05-index-sugar-none` | OOB / i<0 → `None` (no panic) |
| `core05-index-sugar-eq-get` | golden: `v[i]` ≡ `v.get(i)` |
| `core05-index-emit-ban` | emit grep: cero Index/IndexMut / panic `[]` |
| `neg-core05-index-mut` | `v[i] = …` → diag estable (E0314 sugerido) |

## 3. Criterio CLOSED

Lex §2 verde; tick ADR-259 slice 2; Fase B ADR-227 marcada; HOLDs IndexMut/Mutex intactos. Siguiente: slice 3 SCENARIO-COLL.

## Checklist

- [x] Pins `[]`→get/Option + emit-ban + oracles
- [x] IMPL + Lex **705/705** CLOSED
- [x] Slice 3 GO SCENARIO-COLL

## Cierre

- **GO Ingeniero** 2026-09-25 · gate [`DOC/GATE-CORE05-INDEX-SUGAR-20260925.md`](../GATE-CORE05-INDEX-SUGAR-20260925.md)
- Lex measure **705/705** accepted
- Clippy workspace OK; Veyra companion REJECTED (bytes RUSTSEC + rustfmt) **non-blocking**
- HOLDs IndexMut/Mutex/idle/TLS/WS/crates.io/repair intactos
