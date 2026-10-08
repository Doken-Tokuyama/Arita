# ADR-275 — Core 0.7 slice 4: REF-IO

- **Estado:** **CLOSED** Lex **752/752** = Core **0.7 CLOSED** (2026-09-26) · gate [`DOC/GATE-CORE07-REF-IO-20260926.md`](../GATE-CORE07-REF-IO-20260926.md)
- **CUT-ID:** `CORE-0.7-REF-IO-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-271](271-core-0.7-pins.md) §0.1 slice 4 · §2 programa ref
- **Prev:** slice 3 SCENARIO-IO (ADR-274) **CLOSED** Lex **744/744**; este CUT **no** reabre 272–274
- **Prereq surface (al GO IMPL):** E0340 + E0341 + SCENARIO-IO (274) en tree + packages 0.4 layout + host 238
- **Cierra:** vertical Core **0.7** (I/O anti-theater / H1+H2)
- **HOLD:** I/O API nueva · Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · String.set · `?` · HTTP daemon · threads

## Objetivo

Ref **no trivial** `arita-ref-io` + evidence + Lex barra que **cierra Core 0.7**: workspace lib+bin; path/args/JSON **requeridos** vía host 238 → fail explícito en Err/None; happy stdout determinista; scenarios verdes; evidence hash. Sin API nueva. skip ≠ PASS.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Nombre** | `arita-ref-io` (`ejemplos/core07/ref-io/` o Lex equiv.) |
| **Layout** | `arita.toml` → emit Cargo workspace `{lib,bin}` (255/258/268 style) |
| **Lib** | ≥2 `pub` ítems: leen path/JSON/args **required** con match Err/None→fail; dominio puro Text/Int; **sin** HTTP/Mutex |
| **CLI** | args + optional file path → llama lib → stdout **determinista** (happy + fail miss/Err) |
| **Scenarios** | ≥2 acceptance (happy · fail); reusar oracles 274 donde aplique |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`); dual-oracle si Measure lo exige |
| **Emit ban** | cero unwrap/expect/panic (241) · cero argv Index · cero discard Result theater |
| **OUT** | new host APIs · Mutex · IndexMut · net · crates.io · generics avanzados |

## 1. Surface

Solo APIs ya IN (271–274 + 238 + packages 0.4). Sin std/keyword nueva.

```text
// lib: pub fn load_required(path, key) -> Result<State, Int>  // read + json_get; Err on miss
//      pub fn from_args(...) -> Result<State, Int>            // cli_arg required
// bin: match load → print Ok / fail Err
scenario ref_io_happy { acceptance { /* stdout via lib */ } }
scenario ref_io_fail  { acceptance { /* Err/None path */ } }
```

## 2. Oracles / evidence — gate CLOSED Core 0.7

| Id | Expect |
|----|--------|
| `core07-ref-io-build` | workspace lib+bin build verde |
| `core07-ref-io-cli-happy` | CLI → stdout esperado vía lib (read Ok + args/JSON Some) |
| `core07-ref-io-cli-fail` | miss/Err → fail determinista (no panic / no default theater) |
| `core07-ref-io-scenario` | ≥2 scenarios PASS (alineados 274) |
| `core07-ref-io-evidence` | evidence JSON + hash estable |
| `core07-ref-io-emit-ban` | emit grep: cero unwrap/expect/panic / argv Index |
| `neg-core07-ref-io-h1` | discard Result → **E0340** (puede vivir en 272/274) |
| `neg-core07-ref-io-h2` | required miss-as-ok → **E0341** (puede vivir en 273/274) |

Ingeniero fija N/N Lex. skip ≠ PASS. **No** inventar PASS.

## 3. Qué NO tocar (HOLDs + post-0.7)

| Prohibido | Motivo |
|-----------|--------|
| I/O bindings nuevos | HOLD H1/H2 *new* |
| Mutex / IndexMut / idle / TLS/WS / crates.io / repair / String.set / `?` | HOLD global |
| HTTP daemon / compose | fuera de este vertical |
| GO IMPL antes gate Orquestador (CLOSED 272–274) | coordinación |
| CLOSED Core 0.7 sin Lex §2 verde | solo este slice cierra 0.7 cuando barra pase |

## 4. Criterio CLOSED (slice 4 = Core 0.7 CLOSED)

Lex §2 verde; ROADMAP/ADR-271 Core 0.7 **CLOSED**; HOLDs §3 **no** unpark.

## 5. Post-0.7 (no este CUT)

Siguiente vertical: solo con GO <person> + ADR con oráculos. Candidatos (HOLD hasta GO): Mutex (229), idle-kill, TLS/WS, crates.io, repair, `?`/H3–H5. **No** reescribir 0.7 mid-flight.

## Checklist

- [x] Pins ref `arita-ref-io` + oracles/evidence + qué NO tocar (GO-listo)
- [x] GO IMPL Orquestador / Ingeniero (post-CLOSED 272–274)
- [x] IMPL + Lex **752/752** → **Core 0.7 CLOSED**

## Cierre

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE07-REF-IO-20260926.md`](../GATE-CORE07-REF-IO-20260926.md)
- Lex measure **752/752** accepted → **Core 0.7 CLOSED** (evidence `DOC/reviews/MEASURE_ADR275_REF_IO_20260926.json`)
- Clippy workspace OK; Veyra companion REJECTED (`RUSTSEC-2026-0007` bytes + rustfmt-diff) **non-blocking**
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair/I/O-new/String.set/`?` **no** unpark
- Post-0.7: HOLD draft pins hasta GO <person> · Do NOT invent Core 0.8
