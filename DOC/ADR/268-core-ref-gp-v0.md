# ADR-268 — Core 0.6 slice 4: REF-GP

- **Estado:** **CLOSED** Lex **728/728** = Core **0.6 CLOSED** (2026-09-26) · gate [`DOC/GATE-CORE06-REF-GP-20260926.md`](../GATE-CORE06-REF-GP-20260926.md)
- **CUT-ID:** `CORE-0.6-REF-GP-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-264](264-core-0.6-pins.md) §0.1 slice 4 · §2 programa ref
- **Prev:** slice 3 SCENARIO-GP (ADR-267) — **CLOSED** Lex **720/720** (gate `DOC/GATE-CORE06-SCENARIO-GP-20260926.md`). Este CUT **no** reabre 265/266/267.
- **Prereq surface (al GO IMPL):** SET (265) + MAP-INDEX (266) + SCENARIO-GP (267) en tree + packages 0.4 + collections 0.5 + CLI/JSON 0.1 (238/240)
- **Cierra:** vertical Core **0.6** (GP E2E / punta a punta)
- **HOLD:** Mutex · IndexMut assign · idle-kill · TLS/WS · crates.io · repair · I/O H1/H2 nuevo · threads · String.set · HTTP daemon (fuera de este vertical)

## Objetivo

Ref **no trivial** `arita-ref-gp` + evidence + Lex barra que **cierra Core 0.6**: workspace lib+bin, entrada args/JSON → mutación List/Map fallible (`set`/`insert`/`put`/`get`/`[]`) → stdout determinista; scenarios verdes; evidence hash. Sin API nueva. skip ≠ PASS.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Nombre** | `arita-ref-gp` (`ejemplos/core06/ref-gp/` o Lex equiv.) |
| **Layout** | `arita.toml` → emit Cargo workspace `{lib,bin}` (ADR-255/258/263 style) |
| **Lib** | ≥2 `pub` ítems: construyen/mutan `Vec`/`List` con **set**/insert fallible + `Map` con put + lecturas `get`/`[]` sugar; dominio puro (ints/strings/JSON-mini); **sin** HTTP/Mutex |
| **CLI** | args y/o JSON file (surface 238/240) → llama lib → stdout **determinista** (happy + edge OOB/Err/None) |
| **Scenarios** | ≥2 acceptance (happy set+Map+CLI/JSON · OOB/miss sin panic); reusar oracles 267 donde aplique |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`); dual-oracle vs Rust safe set/get si Measure lo exige (264 §0) |
| **Emit ban** | cero IndexMut / panic `set` / panic `[]` Map (grep suite 265/266) |
| **OUT** | Mutex · IndexMut assign · net/HTTP · crates.io · I/O H1/H2 bindings nuevos · generics avanzados · String.set |

## 1. Surface

Solo APIs ya IN (264–267 + 0.1/0.4/0.5). Sin std/keyword nueva.

```text
// lib: pub fn apply_patch(...) -> Result<State, Int>  // set/insert/put fallibles
//      pub fn peek(state, key) -> Option<...>         // get / [] sugar
// bin: parse args/JSON → apply_patch → peek → print
scenario ref_gp_happy { acceptance { /* stdout via lib */ } }
scenario ref_gp_oob   { acceptance { /* Err/None path, no panic */ } }
```

## 2. Oracles / evidence — gate CLOSED Core 0.6

| Id | Expect |
|----|--------|
| `core06-ref-gp-build` | workspace lib+bin build verde |
| `core06-ref-gp-cli-happy` | CLI/JSON → stdout esperado vía lib (set Ok + Map Some) |
| `core06-ref-gp-cli-oob` | set OOB Err y/o Map miss None estable (no panic) |
| `core06-ref-gp-scenario` | ≥2 scenarios PASS (alineados 267) |
| `core06-ref-gp-evidence` | evidence JSON + hash estable |
| `core06-ref-gp-emit-ban` | emit grep: cero IndexMut / panic set/`[]` |
| `neg-core06-ref-gp-index-mut` | IndexMut → diag estable (puede vivir en 265/266/267) |
| `neg-core06-ref-gp-set-neg` | lit `set(-1,…)` → **E0319** (puede vivir en 265/267; E0315 = missing field, no reasignar) |

Ingeniero fija N/N Lex. skip ≠ PASS. **No** inventar PASS.

## 3. Qué NO tocar (HOLDs + post-0.6)

| Prohibido | Motivo |
|-----------|--------|
| Mutex / threads | HOLD Núcleo 2 (229) |
| `v[i]=` / `m[k]=` (IndexMut) | HOLD global; solo neg oracle |
| idle-kill · TLS/WS · crates.io · repair | HOLD intactos |
| I/O H1/H2 file/CLI OOB traps / bindings nuevos | vertical **post-0.6** (264 nota MVP) |
| Reabrir SET (265) / MAP (266) / SCENARIO (267) surface | ya pinada |
| HTTP daemon / compose | Core 0.2–0.3; fuera de GP CLI |
| GO IMPL Codegen **antes** gate Orquestador (CLOSED 266 + 267) | coordinación |
| Declarar CLOSED Core 0.6 **sin** Lex §2 verde | solo este slice cierra 0.6 cuando barra pase |

## 4. Criterio CLOSED (slice 4 = Core 0.6 CLOSED)

Lex §2 verde; ROADMAP/ADR-264 Core 0.6 **CLOSED**; HOLDs §3 **no** unpark.

## 5. Post-0.6 (no este CUT)

Siguiente vertical: solo con GO <person> + ADR con oráculos. Candidatos (HOLD hasta GO): I/O files/JSON trampas H1/H2, Mutex (229), idle-kill, TLS/WS, crates.io, repair. **No** reescribir 0.6 mid-flight.

## Checklist

- [x] Pins ref `arita-ref-gp` + oracles/evidence + qué NO tocar (GO-listo)
- [x] GO IMPL Orquestador / Ingeniero (post-CLOSED 267)
- [x] IMPL + Lex barra → **Core 0.6 CLOSED** Lex **728/728** ([GATE](../GATE-CORE06-REF-GP-20260926.md))

## Cierre

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE06-REF-GP-20260926.md`](../GATE-CORE06-REF-GP-20260926.md)
- Lex measure **728/728** accepted → **Core 0.6 CLOSED** (evidence `DOC/reviews/MEASURE_ADR268_REF_GP_CLOSED_20260926.json`)
- Clippy workspace OK; Veyra companion REJECTED (`RUSTSEC-2026-0007` bytes + rustfmt-diff + cargo-test mid-flip) **non-blocking**
- HOLDs Mutex/IndexMut/idle/TLS/WS/crates.io/repair **no** unpark
- Post-0.6: HOLD draft pins hasta GO <person> · Do NOT invent Core 0.7
