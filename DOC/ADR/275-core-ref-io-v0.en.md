Translation of `275-core-ref-io-v0.md`; the original is normative. / Traducción de `275-core-ref-io-v0.md`; el original es el normativo.

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

## Goal

Non-trivial ref `arita-ref-io` + evidence + Lex bar that **closes Core 0.7**: lib+bin workspace; path/args/JSON **required** via host 238 → explicit fail on Err/None; deterministic happy stdout; green scenarios; evidence hash. No new API. skip ≠ PASS.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Name** | `arita-ref-io` (`ejemplos/core07/ref-io/` or Lex equiv.) |
| **Layout** | `arita.toml` → emit Cargo workspace `{lib,bin}` (255/258/268 style) |
| **Lib** | ≥2 `pub` items: read path/JSON/args **required** with match Err/None→fail; pure Text/Int domain; **no** HTTP/Mutex |
| **CLI** | args + optional file path → call lib → **deterministic** stdout (happy + miss/Err fail) |
| **Scenarios** | ≥2 acceptance (happy · fail); reuse 274 oracles where they apply |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`); dual-oracle if Measure requires it |
| **Emit ban** | zero unwrap/expect/panic (241) · zero argv Index · zero Result discard theater |
| **OUT** | new host APIs · Mutex · IndexMut · net · crates.io · advanced generics |

## 1. Surface

Only already-IN APIs (271–274 + 238 + packages 0.4). No new std/keyword.

```text
// lib: pub fn load_required(path, key) -> Result<State, Int>  // read + json_get; Err on miss
//      pub fn from_args(...) -> Result<State, Int>            // cli_arg required
// bin: match load → print Ok / fail Err
scenario ref_io_happy { acceptance { /* stdout via lib */ } }
scenario ref_io_fail  { acceptance { /* Err/None path */ } }
```

## 2. Oracles / evidence — CLOSED Core 0.7 gate

| Id | Expect |
|----|--------|
| `core07-ref-io-build` | green lib+bin workspace build |
| `core07-ref-io-cli-happy` | CLI → expected stdout via lib (read Ok + args/JSON Some) |
| `core07-ref-io-cli-fail` | miss/Err → deterministic fail (no panic / no default theater) |
| `core07-ref-io-scenario` | ≥2 scenarios PASS (aligned with 274) |
| `core07-ref-io-evidence` | evidence JSON + stable hash |
| `core07-ref-io-emit-ban` | emit grep: zero unwrap/expect/panic / argv Index |
| `neg-core07-ref-io-h1` | discard Result → **E0340** (may live in 272/274) |
| `neg-core07-ref-io-h2` | required miss-as-ok → **E0341** (may live in 273/274) |

The Engineer sets Lex N/N. skip ≠ PASS. **Do not** invent PASS.

## 3. What NOT to touch (HOLDs + post-0.7)

| Forbidden | Reason |
|-----------|--------|
| New I/O bindings | HOLD H1/H2 *new* |
| Mutex / IndexMut / idle / TLS/WS / crates.io / repair / String.set / `?` | Global HOLD |
| HTTP daemon / compose | outside this vertical |
| GO IMPL before Orchestrator gate (CLOSED 272–274) | coordination |
| Core 0.7 CLOSED without green Lex §2 | only this slice closes 0.7 when the bar passes |

## 4. CLOSED criterion (slice 4 = Core 0.7 CLOSED)

Green Lex §2; ROADMAP/ADR-271 Core 0.7 **CLOSED**; §3 HOLDs **not** unparked.

## 5. Post-0.7 (not this CUT)

Next vertical: only with GO <person> + ADR with oracles. Candidates (HOLD until GO): Mutex (229), idle-kill, TLS/WS, crates.io, repair, `?`/H3–H5. **Do not** rewrite 0.7 mid-flight.

## Checklist

- [x] Ref pins `arita-ref-io` + oracles/evidence + what NOT to touch (GO-ready)
- [x] GO IMPL Orchestrator / Engineer (post-CLOSED 272–274)
- [x] IMPL + Lex **752/752** → **Core 0.7 CLOSED**

## Close

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE07-REF-IO-20260926.md`](../GATE-CORE07-REF-IO-20260926.md)
- Lex measure **752/752** accepted → **Core 0.7 CLOSED** (evidence `DOC/reviews/MEASURE_ADR275_REF_IO_20260926.json`)
- Clippy workspace OK; Veyra companion REJECTED (`RUSTSEC-2026-0007` bytes + rustfmt-diff) **non-blocking**
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair/I/O-new/String.set/`?` **not** unparked
- Post-0.7: HOLD draft pins until GO <person> · Do NOT invent Core 0.8
