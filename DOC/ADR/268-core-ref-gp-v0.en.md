Translation of `268-core-ref-gp-v0.md`; the original is normative. / Traducción de `268-core-ref-gp-v0.md`; el original es el normativo.

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

## Objective

Non-trivial ref `arita-ref-gp` + evidence + Lex bar that **closes Core 0.6**: workspace lib+bin, args/JSON input → fallible List/Map mutation (`set`/`insert`/`put`/`get`/`[]`) → deterministic stdout; green scenarios; evidence hash. No new API. skip ≠ PASS.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Nombre** | `arita-ref-gp` (`ejemplos/core06/ref-gp/` o Lex equiv.) |
| **Layout** | `arita.toml` → emit Cargo workspace `{lib,bin}` (ADR-255/258/263 style) |
| **Lib** | ≥2 `pub` items: build/mutate `Vec`/`List` with fallible **set**/insert + `Map` with put + reads `get`/`[]` sugar; pure domain (ints/strings/JSON-mini); **no** HTTP/Mutex |
| **CLI** | args and/or JSON file (surface 238/240) → calls lib → stdout **deterministic** (happy + edge OOB/Err/None) |
| **Scenarios** | ≥2 acceptance (happy set+Map+CLI/JSON · OOB/miss no panic); reuse 267 oracles where they apply |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`); dual-oracle vs Rust safe set/get si Measure lo exige (264 §0) |
| **Emit ban** | cero IndexMut / panic `set` / panic `[]` Map (grep suite 265/266) |
| **OUT** | Mutex · IndexMut assign · net/HTTP · crates.io · new I/O H1/H2 bindings · advanced generics · String.set |

## 1. Surface

Only already-IN APIs (264–267 + 0.1/0.4/0.5). No new std/keyword.

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
| `core06-ref-gp-build` | workspace lib+bin build green |
| `core06-ref-gp-cli-happy` | CLI/JSON → expected stdout via lib (set Ok + Map Some) |
| `core06-ref-gp-cli-oob` | set OOB Err and/or Map miss None stable (no panic) |
| `core06-ref-gp-scenario` | ≥2 scenarios PASS (alineados 267) |
| `core06-ref-gp-evidence` | evidence JSON + stable hash |
| `core06-ref-gp-emit-ban` | emit grep: cero IndexMut / panic set/`[]` |
| `neg-core06-ref-gp-index-mut` | IndexMut → stable diag (may live in 265/266/267) |
| `neg-core06-ref-gp-set-neg` | lit `set(-1,…)` → **E0319** (may live in 265/267; E0315 = missing field, do not reassign) |

Engineer pins N/N Lex. skip ≠ PASS. Do **not** invent PASS.

## 3. What NOT to touch (HOLDs + post-0.6)

| Forbidden | Reason |
|-----------|--------|
| Mutex / threads | HOLD Core 2 (229) |
| `v[i]=` / `m[k]=` (IndexMut) | global HOLD; neg oracle only |
| idle-kill · TLS/WS · crates.io · repair | HOLD intactos |
| I/O H1/H2 file/CLI OOB traps / new bindings | **post-0.6** vertical (264 MVP note) |
| Reopen SET (265) / MAP (266) / SCENARIO (267) surface | already pinned |
| HTTP daemon / compose | Core 0.2–0.3; outside GP CLI |
| GO IMPL Codegen **before** Orchestrator gate (CLOSED 266 + 267) | coordination |
| Declare Core 0.6 CLOSED **without** Lex §2 green | only this slice closes 0.6 when the bar passes |

## 4. CLOSED criterion (slice 4 = Core 0.6 CLOSED)

Lex §2 green; ROADMAP/ADR-264 Core 0.6 **CLOSED**; HOLDs §3 **no** unpark.

## 5. Post-0.6 (not this CUT)

Next vertical: only with GO <person> + ADR with oracles. Candidates (HOLD until GO): I/O files/JSON traps H1/H2, Mutex (229), idle-kill, TLS/WS, crates.io, repair. Do **not** rewrite 0.6 mid-flight.

## Checklist

- [x] Pins ref `arita-ref-gp` + oracles/evidence + what NOT to touch (GO-ready)
- [x] GO IMPL Orchestrator / Engineer (post-CLOSED 267)
- [x] IMPL + Lex bar → **Core 0.6 CLOSED** Lex **728/728** ([GATE](../GATE-CORE06-REF-GP-20260926.md))

## Close

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE06-REF-GP-20260926.md`](../GATE-CORE06-REF-GP-20260926.md)
- Lex measure **728/728** accepted → **Core 0.6 CLOSED** (evidence `DOC/reviews/MEASURE_ADR268_REF_GP_CLOSED_20260926.json`)
- Clippy workspace OK; Veyra companion REJECTED (`RUSTSEC-2026-0007` bytes + rustfmt-diff + cargo-test mid-flip) **non-blocking**
- HOLDs Mutex/IndexMut/idle/TLS/WS/crates.io/repair **no** unpark
- Post-0.6: HOLD draft pins until GO <person> · Do NOT invent Core 0.7
