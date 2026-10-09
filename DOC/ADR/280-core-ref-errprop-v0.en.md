Translation of `280-core-ref-errprop-v0.md`; the original is normative. / Traducción de `280-core-ref-errprop-v0.md`; el original es el normativo.

# ADR-280 — Core 0.8 slice 4: REF-ERRPROP

- **Estado:** **CLOSED** Lex **778/778** = Core **0.8 CLOSED** (2026-09-26) · gate `DOC/GATE-CORE08-REF-ERRPROP-20260926.md` (not in the public export / no incluido en el export público)
- **CUT-ID:** `CORE-0.8-REF-ERRPROP-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero · Orquestador (GO IMPL)
- **Padre:** [ADR-276](276-core-0.8-pins.md) §0.1 slice 4 · §2 programa ref
- **Prev:** slice 3 SCENARIO-ERRPROP (ADR-279) — **prereq CLOSED** al GO IMPL; este CUT **no** reabre 277–279
- **Prereq surface (al GO IMPL):** FN-RESULT + QMARK + SCENARIO-ERRPROP en tree + packages 0.4 layout + host 238
- **Cierra:** vertical Core **0.8** (Error propagation / H3) — **CLOSED** Lex **778/778**
- **HOLD:** Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · I/O-new · String.set · threads · `?` in Io main · Option? · surface unwrap/expect · HTTP daemon · H4 VOID reopen · H5 IMPL

## Goal

Non-trivial ref `arita-ref-errprop` + evidence + Lex bar that **closes Core 0.8**: lib+bin workspace; lib `fn → Result` using `?`; CLI main `Io<()>` **match-converts** (no `?` in main Io); happy + Err paths; green scenarios; evidence hash. No new API. skip ≠ PASS. **Do not** invent PASS/N/N in this DOC.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Name** | `arita-ref-errprop` (`ejemplos/core08/ref-errprop/` or Lex equiv.) |
| **Layout** | `arita.toml` → emit Cargo workspace `{lib,bin}` (255/258/275 style) |
| **Lib** | ≥2 `pub` items `-> Result<…>`: use `?` on Result (host 238 and/or helpers); typed domain; **no** HTTP/Mutex; **no** unwrap |
| **CLI** | `fn main() -> Io<()>`: match Ok/Err from lib → **deterministic** stdout (happy + fail); **zero** `?` in main Io |
| **Scenarios** | ≥2 acceptance (happy · Err); reuse 279 oracles where they apply |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`); dual-oracle if Measure requires it |
| **Emit ban** | zero unwrap/expect/panic (241) · zero `?` in Io main · zero Result discard theater (E0340 remains) |
| **OUT** | new host APIs · Mutex · IndexMut · net · crates.io · Option? · async Result |

## 1. Surface

Only already-IN APIs (276–279 + 238 + packages 0.4 + Result 047). No new std/keyword beyond `?` (278).

```text
// lib: pub fn load_required(path) -> Result<State, Int>  // ? on host.read_text
//      pub fn process(...) -> Result<State, Int>         // ? chain
// bin: match load/process → print Ok / fail Err   // no ? in main Io
scenario ref_errprop_happy { acceptance { /* stdout via lib */ } }
scenario ref_errprop_err   { acceptance { /* Err path */ } }
```

## 2. Oracles / evidence — CLOSED Core 0.8 gate

| Id | Expect |
|----|--------|
| `core08-ref-errprop-build` | green lib+bin workspace build |
| `core08-ref-errprop-cli-happy` | CLI → expected stdout via lib (`?` chain Ok + main match) |
| `core08-ref-errprop-cli-fail` | Err via `?` → deterministic fail (no panic / no unwrap) |
| `core08-ref-errprop-scenario` | ≥2 scenarios PASS (aligned with 279) |
| `core08-ref-errprop-evidence` | evidence JSON + stable hash |
| `core08-ref-errprop-emit-ban` | emit grep: zero unwrap/expect/panic; zero `?` in main Io |
| `neg-core08-ref-qmark-outside` | `?` outside Result fn → **E0343** (may live in 278/279) |
| `neg-core08-ref-fn-theater` | bad Result return theater → **E0342** (may live in 277) |

The Engineer sets Lex N/N at GO IMPL. skip ≠ PASS. **Do not** invent PASS in draft.

## 3. What NOT to touch (HOLDs + post-0.8)

| Forbidden | Reason |
|-----------|--------|
| Mutex / IndexMut / idle / TLS/WS / crates.io / repair / I/O-new / String.set / threads | Global HOLD |
| `?` in Io main · Option? · unwrap surface | OUT / HOLD |
| H4 VOID reopen E0272 · H5 IMPL · scout diag JSON language slice | OUT / DOC companion |
| HTTP daemon / compose | outside this vertical |
| GO IMPL (granted) post-CLOSED 277–279 Lex **770/770** | **CLOSED** Lex **778/778** = Core 0.8 CLOSED |
| Core 0.8 CLOSED without green Lex §2 | only this slice closes 0.8 when the bar passes |
| Claim Core 0.8 CLOSED in draft | **NO** |

## 4. CLOSED criterion (slice 4 = Core 0.8 CLOSED)

Green Lex §2; ROADMAP/ADR-276 Core 0.8 **CLOSED**; §3 HOLDs **not** unparked (except already-landed `?`/fn→Result).

## 5. Post-0.8 (not this CUT)

Next vertical: only with GO <person> + ADR with oracles. HOLD candidates: Mutex (229), idle-kill, TLS/WS, crates.io, repair, H5 DOC→pin, scout diag JSON DOC, residual H4-enum. **Do not** rewrite 0.8 mid-flight. Vertical ≠ GP ceiling.

## Checklist

- [x] Ref pins `arita-ref-errprop` + oracles/evidence + what NOT to touch (DOC GO-ready)
- [x] GO IMPL Orchestrator / Engineer (post-CLOSED 277–279 Lex **770/770**)
- [x] IMPL + Lex **778/778** → **Core 0.8 CLOSED** (GATE (not in the public export / no incluido en el export público))

## Close

- **GO Ingeniero** 2026-09-26 · gate `DOC/GATE-CORE08-REF-ERRPROP-20260926.md` (not in the public export / no incluido en el export público)
- Lex measure **778/778** accepted → **Core 0.8 CLOSED** (evidence `DOC/reviews/MEASURE_ADR280_REF_ERRPROP_CLOSED_20260926.json`; prior mid-slice `MEASURE_ADR280_REF_ERRPROP_20260926.json`)
- Clippy workspace OK; Veyra companion **ACCEPTED** exit **0** · `.veyra/evidence/20260926T153538Z/` (trail `153102Z` rustfmt-diff → `cargo fmt` → `153538Z`; residual VT008×12 info)
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair/I/O-new/String.set/threads/unwrap-surface/Option?/`?-in-Io-main` **not** unparked
- Post-0.8: **HOLD** draft pins until GO <person> · Do NOT invent Core 0.9 · **No** next GO IMPL
- Unpark land: `?` + `fn → Result` only (277–280)
- H4 VOID · H5 DOC · scout diag OUT · Packaging MCP/ACP **HOLD** intact
- Vertical ≠ GP ceiling
