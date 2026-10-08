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

## Objetivo

Ref **no trivial** `arita-ref-collections` + evidence + Lex barra que **cierra Core 0.5**: workspace lib+bin (layout 0.4), usa `insert`→`Result` + index sugar/`get`→`Option`, scenarios verdes, evidence hash. Sin API nueva. skip ≠ PASS.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Nombre** | `arita-ref-collections` (`ejemplos/core05/ref-collections/` o Lex equiv.) |
| **Layout** | `arita.toml` → emit Cargo workspace `{lib,bin}` (ADR-255/258 style) |
| **Lib** | ≥2 `pub` ítems que construyen/leen `Vec` con **insert fallible** + lecturas `get`/`[]` sugar; dominio puro (ints/strings cortos); **sin** HTTP/Mutex |
| **CLI** | args y/o file → llama lib → stdout **determinista** (happy + edge OOB/Err) |
| **Scenarios** | ≥2 acceptance (happy insert+index · OOB None/Err sin panic); reusar oracles 262 donde aplique |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`); dual-oracle vs Rust safe helpers si Measure lo exige (259 §0) |
| **Emit ban** | cero `Vec::insert` bare panic · cero `Index`/`IndexMut` / panic `[]` (grep suite 260/261) |
| **OUT** | Mutex · IndexMut · net/HTTP · crates.io · generics avanzados · String.insert |

## 1. Surface

Solo APIs ya IN (259–262 + packages 0.4). Sin std/keyword nueva.

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
| `core05-ref-coll-build` | workspace lib+bin build verde |
| `core05-ref-coll-cli-happy` | CLI → stdout esperado vía lib (insert Ok + index Some) |
| `core05-ref-coll-cli-oob` | OOB insert Err y/o index None estable (no panic) |
| `core05-ref-coll-scenario` | ≥2 scenarios PASS (alineados 262) |
| `core05-ref-coll-evidence` | evidence JSON + hash estable |
| `core05-ref-coll-emit-ban` | emit grep: cero Index/IndexMut / panic insert/`[]` |
| `neg-core05-ref-coll-index-mut` | IndexMut → diag estable (puede vivir en suite 261/262) |

Ingeniero fija N/N Lex. skip ≠ PASS.

## 3. Criterio CLOSED (slice 4 = Core 0.5 CLOSED)

Lex §2 verde; ROADMAP/ADR-259 Core 0.5 **CLOSED**; HOLDs Mutex/idle/TLS/WS/crates.io/repair/IndexMut **no** unpark.

## 4. Post-0.5 (no este CUT)

Siguiente vertical: solo con GO <person> + ADR con oráculos. Candidatos históricos (HOLD hasta GO): Mutex (ADR-229), idle-kill, TLS/WS, crates.io, repair.

## Checklist

- [x] Pins ref + oracles + evidence + CLOSE Core 0.5 (draft paralelo)
- [x] GO IMPL Orquestador / Ingeniero
- [x] IMPL + Lex **705/705** → **Core 0.5 CLOSED**

## Cierre

- **GO Ingeniero** 2026-09-25 · gate [`DOC/GATE-CORE05-REF-COLL-20260925.md`](../GATE-CORE05-REF-COLL-20260925.md)
- Lex measure **705/705** accepted → **Core 0.5 CLOSED**
- Clippy workspace OK; Veyra companion REJECTED (bytes RUSTSEC + rustfmt) **non-blocking**
- HOLDs Mutex/IndexMut/idle/TLS/WS/crates.io/repair **no** unpark
- Post-0.5: HOLD draft pins hasta GO <person>
