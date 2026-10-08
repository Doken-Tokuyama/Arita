Translation of `263-core-ref-coll-v0.md`; the original is normative. / Traducción de `263-core-ref-coll-v0.md`; el original es el normativo.

# ADR-263 — Core 0.5 slice 4: REF-COLL

- **Estado:** **CLOSED** Lex **705/705** (2026-09-25 ~20:53 CEST) · gate `DOC/GATE-CORE05-REF-COLL-20260925.md` · **Core 0.5 CLOSED**
- **CUT-ID:** `CORE-0.5-REF-COLL-20260920`
- **Fecha:** 2026-09-20 · **draft pins:** 2026-09-25
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-259](259-core-0.5-pins.md) §0.1 slice 4 · §2 programa ref
- **Prev:** slice 3 SCENARIO-COLL (ADR-262) — GO IMPL / land Measure-cola; este draft **no** espera CLOSED para existir; GO IMPL al gate Orquestador
- **Prereq surface:** INSERT (ADR-260) + INDEX-SUGAR (ADR-261) + SCENARIO-COLL (ADR-262) en tree
- **Cierra:** vertical Core **0.5** (fallible collections / Núcleo 1)
- **HOLD:** Mutex · IndexMut · idle-kill · TLS/WS · crates.io · repair · HTTP · String.insert

## Objective

Non-trivial ref `arita-ref-collections` + evidence + Lex bar that **closes Core 0.5**: workspace lib+bin (layout 0.4), uses `insert`→`Result` + index sugar/`get`→`Option`, green scenarios, evidence hash. No new API. skip ≠ PASS.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Nombre** | `arita-ref-collections` (`ejemplos/core05/ref-collections/` o Lex equiv.) |
| **Layout** | `arita.toml` → emit Cargo workspace `{lib,bin}` (ADR-255/258 style) |
| **Lib** | ≥2 `pub` items that build/read `Vec` with **fallible insert** + `get`/`[]` sugar reads; pure domain (short ints/strings); **no** HTTP/Mutex |
| **CLI** | args and/or file → calls lib → stdout **deterministic** (happy + edge OOB/Err) |
| **Scenarios** | ≥2 acceptance (happy insert+index · OOB None/Err no panic); reuse 262 oracles where they apply |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`); dual-oracle vs Rust safe helpers si Measure lo exige (259 §0) |
| **Emit ban** | cero `Vec::insert` bare panic · cero `Index`/`IndexMut` / panic `[]` (grep suite 260/261) |
| **OUT** | Mutex · IndexMut · net/HTTP · crates.io · generics avanzados · String.insert |

## 1. Surface

Only already-IN APIs (259–262 + packages 0.4). No new std/keyword.

```text
// lib: pub fn build_bag(...) -> Result<Vec<Int>, Int>  // inserts fallibles
//      pub fn peek(bag, i) -> Option<Int>               // get / [] sugar
// bin: parse → build → peek → print
scenario ref_coll_happy { acceptance { /* stdout via lib */ } }
scenario ref_coll_oob   { acceptance { /* Err/None path, no panic */ } }
```

## 2. Oracles — gate CLOSED Core 0.5

| Id | Expect |
|----|--------|
| `core05-ref-coll-build` | workspace lib+bin build green |
| `core05-ref-coll-cli-happy` | CLI → expected stdout via lib (insert Ok + index Some) |
| `core05-ref-coll-cli-oob` | OOB insert Err and/or index None stable (no panic) |
| `core05-ref-coll-scenario` | ≥2 scenarios PASS (alineados 262) |
| `core05-ref-coll-evidence` | evidence JSON + stable hash |
| `core05-ref-coll-emit-ban` | emit grep: cero Index/IndexMut / panic insert/`[]` |
| `neg-core05-ref-coll-index-mut` | IndexMut → stable diag (may live in the 261/262 suite) |

Engineer pins N/N Lex. skip ≠ PASS.

## 3. CLOSED criterion (slice 4 = Core 0.5 CLOSED)

Lex §2 green; ROADMAP/ADR-259 Core 0.5 **CLOSED**; HOLDs Mutex/idle/TLS/WS/crates.io/repair/IndexMut **no** unpark.

## 4. Post-0.5 (not this CUT)

Next vertical: only with GO <person> + ADR with oracles. Historic candidates (HOLD until GO): Mutex (ADR-229), idle-kill, TLS/WS, crates.io, repair.

## Checklist

- [x] Pins ref + oracles + evidence + CLOSE Core 0.5 (draft paralelo)
- [x] GO IMPL Orchestrator / Engineer
- [x] IMPL + Lex **705/705** → **Core 0.5 CLOSED**

## Close

- **GO Ingeniero** 2026-09-25 · gate [`DOC/GATE-CORE05-REF-COLL-20260925.md`](../GATE-CORE05-REF-COLL-20260925.md)
- Lex measure **705/705** accepted → **Core 0.5 CLOSED**
- Clippy workspace OK; Veyra companion REJECTED (bytes RUSTSEC + rustfmt) **non-blocking**
- HOLDs Mutex/IndexMut/idle/TLS/WS/crates.io/repair **no** unpark
- Post-0.5: HOLD draft pins until GO <person>
