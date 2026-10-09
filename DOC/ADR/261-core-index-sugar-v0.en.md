Translation of `261-core-index-sugar-v0.md`; the original is normative. / Traducción de `261-core-index-sugar-v0.md`; el original es el normativo.

# ADR-261 — Core 0.5 slice 2: INDEX-SUGAR

- **Estado:** **CLOSED** Lex **705/705** (2026-09-25 ~20:53 CEST) · gate `DOC/GATE-CORE05-INDEX-SUGAR-20260925.md`
- **CUT-ID:** `CORE-0.5-INDEX-SUGAR-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-259](259-core-0.5-pins.md) §0.1 slice 2 · contrato [ADR-227](227-lang-contract-indexget.md) **Fase B**
- **Prev:** slice 1 INSERT **CLOSED** Lex **688/688** (ADR-260) — este CUT no reabre 260
- **Prereq evidencia:** R4 Phase A **CLOSED** Lex **583/583** (E0310) — Architect GO Phase B = this ADR
- **HOLD:** IndexMut · `get_mut` · slices · Mutex · idle · TLS/WS · crates.io · repair · String.insert

## Objective

Sugar `v[i]` / `s[i]` → **same** semantics as `get` → `Option<T>`. Emit identical to `get`. **Forbidden** `Index`/`IndexMut` panic.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Desugar** | `v[i]` / `s[i]` ⇒ `v.get(i)` / `s.get(i)` → `Option` |
| **OOB / i<0** | `None` (same as get) |
| **Emit** | only path `get` / Option — **cero** `values[i]`, `Index`, `IndexMut`, `unwrap` en index |
| **E0310** | **retired** for `Vec`/`String` index sugar; stays OUT for types without `get` |
| **OUT** | `v[i] = x` (IndexMut) · slice syntax · Mutex |

## 1. Surface

```text
let v = vec_of_ints()   // or already-IN construction
match v[0] { Some(x) => print(x), None => print(0) }
match v[999] { Some(_) => ..., None => ... }  // None, no panic
// v[0] = 1  → reject (IndexMut OUT) — stable diag (suggested E0314)
```

`get(i)` remains the canonical API (ADR-085/227); sugar is sugar, not a second semantics.

## 2. Oracles

| Id | Expect |
|----|--------|
| `core05-index-sugar-some` | `v[i]` in-range → `Some` (= get) |
| `core05-index-sugar-none` | OOB / i<0 → `None` (no panic) |
| `core05-index-sugar-eq-get` | golden: `v[i]` ≡ `v.get(i)` |
| `core05-index-emit-ban` | emit grep: zero Index/IndexMut / panic `[]` |
| `neg-core05-index-mut` | `v[i] = …` → stable diag (E0314 suggested) |

## 3. CLOSED criterion

Lex §2 green; tick ADR-259 slice 2; ADR-227 Phase B marked; HOLDs IndexMut/Mutex intact. Next: slice 3 SCENARIO-COLL.

## Checklist

- [x] Pins `[]`→get/Option + emit-ban + oracles
- [x] IMPL + Lex **705/705** CLOSED
- [x] Slice 3 GO SCENARIO-COLL

## Close

- **GO Ingeniero** 2026-09-25 · gate `DOC/GATE-CORE05-INDEX-SUGAR-20260925.md` (not in the public export / no incluido en el export público)
- Lex measure **705/705** accepted
- Clippy workspace OK; Veyra companion REJECTED (bytes RUSTSEC + rustfmt) **non-blocking**
- HOLDs IndexMut/Mutex/idle/TLS/WS/crates.io/repair intact
