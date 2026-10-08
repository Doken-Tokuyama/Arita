# ADR-280 — Core 0.8 slice 4: REF-ERRPROP

- **Estado:** **CLOSED** Lex **778/778** = Core **0.8 CLOSED** (2026-09-26) · gate [`DOC/GATE-CORE08-REF-ERRPROP-20260926.md`](../GATE-CORE08-REF-ERRPROP-20260926.md)
- **CUT-ID:** `CORE-0.8-REF-ERRPROP-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero · Orquestador (GO IMPL)
- **Padre:** [ADR-276](276-core-0.8-pins.md) §0.1 slice 4 · §2 programa ref
- **Prev:** slice 3 SCENARIO-ERRPROP (ADR-279) — **prereq CLOSED** al GO IMPL; este CUT **no** reabre 277–279
- **Prereq surface (al GO IMPL):** FN-RESULT + QMARK + SCENARIO-ERRPROP en tree + packages 0.4 layout + host 238
- **Cierra:** vertical Core **0.8** (Error propagation / H3) — **CLOSED** Lex **778/778**
- **HOLD:** Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · I/O-new · String.set · threads · `?` in Io main · Option? · surface unwrap/expect · HTTP daemon · H4 VOID reopen · H5 IMPL

## Objetivo

Ref **no trivial** `arita-ref-errprop` + evidence + Lex barra que **cierra Core 0.8**: workspace lib+bin; lib `fn → Result` usando `?`; CLI main `Io<()>` **match-convierte** (no `?` en main Io); happy + Err paths; scenarios verdes; evidence hash. Sin API nueva. skip ≠ PASS. **No** inventar PASS/N/N en este DOC.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Nombre** | `arita-ref-errprop` (`ejemplos/core08/ref-errprop/` o Lex equiv.) |
| **Layout** | `arita.toml` → emit Cargo workspace `{lib,bin}` (255/258/275 style) |
| **Lib** | ≥2 `pub` ítems `-> Result<…>`: usan `?` sobre Result (host 238 y/o helpers); dominio tipado; **sin** HTTP/Mutex; **sin** unwrap |
| **CLI** | `fn main() -> Io<()>`: match Ok/Err desde lib → stdout **determinista** (happy + fail); **cero** `?` en main Io |
| **Scenarios** | ≥2 acceptance (happy · Err); reusar oracles 279 donde aplique |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`); dual-oracle si Measure lo exige |
| **Emit ban** | cero unwrap/expect/panic (241) · cero `?` en Io main · cero discard Result theater (E0340 sigue) |
| **OUT** | new host APIs · Mutex · IndexMut · net · crates.io · Option? · async Result |

## 1. Surface

Solo APIs ya IN (276–279 + 238 + packages 0.4 + Result 047). Sin std/keyword nueva más allá de `?` (278).

```text
// lib: pub fn load_required(path) -> Result<State, Int>  // ? on host.read_text
//      pub fn process(...) -> Result<State, Int>         // ? chain
// bin: match load/process → print Ok / fail Err   // no ? in main Io
scenario ref_errprop_happy { acceptance { /* stdout vía lib */ } }
scenario ref_errprop_err   { acceptance { /* Err path */ } }
```

## 2. Oracles / evidence — gate CLOSED Core 0.8

| Id | Expect |
|----|--------|
| `core08-ref-errprop-build` | workspace lib+bin build verde |
| `core08-ref-errprop-cli-happy` | CLI → stdout esperado vía lib (`?` chain Ok + main match) |
| `core08-ref-errprop-cli-fail` | Err via `?` → fail determinista (no panic / no unwrap) |
| `core08-ref-errprop-scenario` | ≥2 scenarios PASS (alineados 279) |
| `core08-ref-errprop-evidence` | evidence JSON + hash estable |
| `core08-ref-errprop-emit-ban` | emit grep: cero unwrap/expect/panic; cero `?` en main Io |
| `neg-core08-ref-qmark-outside` | `?` outside Result fn → **E0343** (puede vivir en 278/279) |
| `neg-core08-ref-fn-theater` | bad Result return theater → **E0342** (puede vivir en 277) |

Ingeniero fija N/N Lex al GO IMPL. skip ≠ PASS. **No** inventar PASS en draft.

## 3. Qué NO tocar (HOLDs + post-0.8)

| Prohibido | Motivo |
|-----------|--------|
| Mutex / IndexMut / idle / TLS/WS / crates.io / repair / I/O-new / String.set / threads | HOLD global |
| `?` in Io main · Option? · unwrap surface | OUT / HOLD |
| H4 VOID reopen E0272 · H5 IMPL · scout diag JSON language slice | OUT / DOC companion |
| HTTP daemon / compose | fuera de este vertical |
| GO IMPL (granted) post-CLOSED 277–279 Lex **770/770** | **CLOSED** Lex **778/778** = Core 0.8 CLOSED |
| CLOSED Core 0.8 sin Lex §2 verde | solo este slice cierra 0.8 cuando barra pase |
| Claim Core 0.8 CLOSED en draft | **NO** |

## 4. Criterio CLOSED (slice 4 = Core 0.8 CLOSED)

Lex §2 verde; ROADMAP/ADR-276 Core 0.8 **CLOSED**; HOLDs §3 **no** unpark (salvo `?`/fn→Result ya land).

## 5. Post-0.8 (no este CUT)

Siguiente vertical: solo con GO <person> + ADR con oráculos. Candidatos HOLD: Mutex (229), idle-kill, TLS/WS, crates.io, repair, H5 DOC→pin, scout diag JSON DOC, H4-enum residual. **No** reescribir 0.8 mid-flight. Vertical ≠ techo GP.

## Checklist

- [x] Pins ref `arita-ref-errprop` + oracles/evidence + qué NO tocar (GO-listo DOC)
- [x] GO IMPL Orquestador / Ingeniero (post-CLOSED 277–279 Lex **770/770**)
- [x] IMPL + Lex **778/778** → **Core 0.8 CLOSED** ([GATE](../GATE-CORE08-REF-ERRPROP-20260926.md))

## Cierre

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE08-REF-ERRPROP-20260926.md`](../GATE-CORE08-REF-ERRPROP-20260926.md)
- Lex measure **778/778** accepted → **Core 0.8 CLOSED** (evidence `DOC/reviews/MEASURE_ADR280_REF_ERRPROP_CLOSED_20260926.json`; prior mid-slice `MEASURE_ADR280_REF_ERRPROP_20260926.json`)
- Clippy workspace OK; Veyra companion **ACCEPTED** exit **0** · `.veyra/evidence/20260926T153538Z/` (trail `153102Z` rustfmt-diff → `cargo fmt` → `153538Z`; residual VT008×12 info)
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair/I/O-new/String.set/threads/unwrap-surface/Option?/`?-in-Io-main` **no** unpark
- Post-0.8: **HOLD** draft pins hasta GO <person> · Do NOT invent Core 0.9 · **No** next GO IMPL
- Unpark land: `?` + `fn → Result` only (277–280)
- H4 VOID · H5 DOC · scout diag OUT · Packaging MCP/ACP **HOLD** intacto
- Vertical ≠ techo GP
