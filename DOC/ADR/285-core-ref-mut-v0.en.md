Translation of `285-core-ref-mut-v0.md`; the original is normative. / Traducción de `285-core-ref-mut-v0.md`; el original es el normativo.

# ADR-285 — Core 0.9 slice 4: REF-MUT

- **Estado:** **CLOSED** Lex **834/834** (Ingeniero 2026-09-27 · [`GATE-CORE09-REF-MUT-20260926.md`](../GATE-CORE09-REF-MUT-20260926.md)) · **cierra Core 0.9** · GO IMPL Ingeniero 2026-09-27 · pins OK Ingeniero 2026-09-26
- **CUT-ID:** `CORE-0.9-REF-MUT-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero (pins OK 2026-09-26 19:54)
- **Padre:** [ADR-281](281-core-0.9-pins.md) §0.1 slice 4 · §2 programa ref
- **Prev:** slice 3 SCENARIO-MUT (ADR-284) — prereq CLOSED al GO IMPL; este CUT **no** reabre 282–284
- **Prereq surface (al GO IMPL):** MAP-ASSIGN + VEC-ASSIGN + SCENARIO-MUT en tree + packages 0.4 layout (255/258) + `fn → Result` / `?` (277/278) + CLI/JSON 0.1 (238/240)
- **Cierra:** vertical Core **0.9** (Mutación de colecciones) — al CLOSED de este slice
- **HOLD:** Mutex · threads · idle · TLS/WS · crates.io · repair · I/O-new · String.set · IndexMut residual (E0314) · `?` in Io main · Option? · surface unwrap/expect (241) · HTTP daemon · MCP/ACP · reopen E0272/E0291/E0340/E0341/E0342/E0343

## Goal

Non-trivial ref `arita-ref-mut` + evidence + Lex bar that **closes Core 0.9**: lib+bin workspace; lib `pub fn → Result<_, Int>` that mutates `Map` via `m[k] = v` and `Vec` via `v[i] = x` (OOB → propagated `Err(0)`) and composes with `?`; CLI main `Io<()>` **match-converts**; happy + Err paths; green scenarios; evidence hash. No new API. skip ≠ PASS. **Do not** invent PASS/N/N in this DOC.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Name** | `arita-ref-mut` (root `ejemplos/core09/ref-mut/`) |
| **Layout** | `arita.toml` → emit Cargo workspace `{lib,bin}` (255/258/268/280 style). Root `ejemplos/core09/ref-mut/` (lib+bin workspace, `fixture.json`, happy); second package `ejemplos/core09/ref-mut/edge/` (lib+bin, `fixture.json` `{"i": 7, "x": 9}`, OOB), because `arita contract` runs without arguments (ADR-280 precedent); negs under `ejemplos/core09/ref-mut-neg/`, outside the workspace (Engineer pin 2026-09-27 07:42) |
| **Lib** | ≥2 `pub` items `-> Result<…, Int>`: build `Vec`/`Map` (params in already-IN `fn_param` v0 types; **no** widen to Vec/Map or `&mut Map/Vec` — HOLD ADR-281 D1), mutate with `m[k] = v` + `v[i] = x`, compose with `?`; typed domain; **no** HTTP/Mutex; **no** unwrap |
| **CLI** | `fn main() -> Io<()>`: args and/or JSON (238/240) → lib → match Ok/Err → **deterministic** stdout (happy + OOB fail); **zero** `?` and **zero** `v[i] =` in main Io (`m[k] = v` allowed) |
| **Scenarios** | exactly two: `ref_mut_happy` (root) and `ref_mut_err` (edge), with the exact stdout of §2 (Engineer pin 2026-09-27 07:42) |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`); no dual-oracle (G4); gates §2.1 |
| **Emit ban** | zero Rust IndexMut / `x[..] =` · zero unwrap/expect/panic (241) · zero `?` in main Io · zero discard of the set + `as usize` at `__arita_vec_set` call sites and their helpers (ADR-283 R1; the guarded `as usize` of `v.get` stays out); exact counts in §2 |
| **OUT** | new host APIs · Mutex · String.set · net · crates.io · Option? · async Result · collection params |

## 1. Surface

Only already-IN APIs (281–284 + 276–280 + 264–270 + 237/238/240 + packages 0.4). No new std/keyword beyond the write sugar (282/283).

```text
// lib: pub fn apply(i: Int, x: Int) -> Result<Int, Int>   // m[k] = v ; v[i] = x (OOB → Err(0))
//      pub fn run(i: Int, x: Int) -> Result<Int, Int>     // apply(i, x)? ; …
// bin: args → run → match Ok/Err → print   // no ? / no v[i] = in main Io
scenario ref_mut_happy { acceptance { /* stdout via lib */ } }
scenario ref_mut_err   { acceptance { /* OOB Err(0) propagated, no panic */ } }
```

## 2. Oracles / evidence — CLOSED Core 0.9 gate

Required (to be measured at GO IMPL; **none** measured in this DOC):

| Id | Expect |
|----|--------|
| `core09-ref-mut-build` | root `ejemplos/core09/ref-mut/` and edge `ejemplos/core09/ref-mut/edge/`: green lib+bin workspace build **in both** (`crates/lib_*` and `crates/bin_*` exist). Source rules (anti-theater, §0 L23–24): (1) lib with ≥2 `pub fn … -> Result<…, Int>`; (2) lib with ≥1 `m[…] =` and ≥1 `v[…] =`; (3) lib with ≥1 composition `?`; (4) zero `unwrap`/`expect` in lib+bin; (5) `bin/main.arita` and `edge/bin/main.arita`: zero `?` and zero `v[…] =` (`m[…] =` allowed); (6) if `edge/lib/lib.arita` ≠ `lib/lib.arita`, Codegen justifies the difference and Measure records it in the evidence/review (not an automatic reject). Any failure of (1)–(5) ⇒ rejected; missing file ⇒ inconclusive (Engineer pin 2026-09-27 07:42) |
| `core09-ref-mut-cli-happy` | CLI via lib (Map insert/overwrite + Vec in-bounds) with `ejemplos/core09/ref-mut/fixture.json` **and** with no argument (default fixture): **exit 0**, stderr without “panicked” (Codegen READY: empty stderr), exact stdout `["2","9","3","ref-mut-happy"]` (= `2|9|3|ref-mut-happy`, one line per token) in both runs (Engineer pin 2026-09-27 07:42) |
| `core09-ref-mut-cli-oob` | edge workspace (`edge/fixture.json` `{"i": 7, "x": 9}`) with and without argument: Vec OOB → propagated `Err(0)` → main’s `Err` arm; **exit 0** (G2; ADR-280 `cli-fail` precedent and ADR-284 L24 “Err in main”), stderr without “panicked”, exact stdout `["ref-mut-err","0"]` (= `ref-mut-err|0`). Payload ≠ `0` → **REJECTED**; also rejected if `ref-mut-happy` appears or there are extra lines. **Anti-theater (same id):** the **root** bin with `-- ejemplos/core09/ref-mut/edge/fixture.json` must also yield exactly `["ref-mut-err","0"]` (Engineer pin 2026-09-27 07:42) |
| `core09-ref-mut-scenario` | exactly two `scenario` blocks: `ref_mut_happy` (`bin/main.arita`) and `ref_mut_err` (`edge/bin/main.arita`); their `acceptance` = the two exact stdout above (happy `["2","9","3","ref-mut-happy"]` · err `["ref-mut-err","0"]`); `arita contract` **Accepted** on both bins; both cli-* oracles Accepted. A count other than two or different names ⇒ rejected (skip ≠ PASS) (Engineer pin 2026-09-27 07:42) |
| `core09-ref-mut-evidence` | `ejemplos/core09/ref-mut/evidence.json`: `schema_version` = `arita.evidence.v1`, `cut_id` = `CORE-0.9-REF-MUT-20260926`, `adr` = `"285"`; declared sha256 == real for `lib/lib.arita`, `bin/main.arita`, `edge/bin/main.arita`, `edge/lib/lib.arita`, `arita.toml`, `edge/arita.toml`, `fixture.json` and `edge/fixture.json`; emit hash (`lib.rs`/`main.rs`) stable across two consecutive builds (if evidence declares `emit_*_sha256`, they must match); `scenarios` = exactly the 9 ids of §2; `measure_pass` informative: the oracle only requires it to be a bool (not `true`); it becomes `true` at CLOSED without touching the oracle (G3; gates §2.1) (Engineer pin 2026-09-27 07:42) |
| `core09-ref-mut-emit-ban` | over all emitted `lib.rs`/`main.rs` (root + edge): zero `.unwrap()`, `.expect(`, `panic!(` (241), zero `IndexMut`/`std::ops::Index` and Rust indexed assignment `ident[…] =`; zero `as usize` at `__arita_vec_set` call sites and in the bodies of helpers `__arita_vec_set`/`__arita_vec_insert` (same scope as ADR-283 R1’s oracle; **not** a whole-file ban): the `as usize` of the `v.get(sel)` read, guarded by `__i < 0`, is legitimate and stays out (ADR-283 §1.2). Each `main.rs`: exactly 0 `?`, 0 `__arita_vec_set` and 3 `.insert(`. `lib.rs`: exactly **4** `__arita_vec_set(&mut …)?` (one per source site `v[…] =`), all with `?`, no `let _ =`/discard of the set; exactly **6** `.insert(` (counts fixed by Measure/Codegen READY) (Engineer pin 2026-09-27 07:42) |
| `neg-core09-ref-vec-assign-outside` | `v[i] = x` in main Io → exactly `E0344: index assign outside result fn` (unique code; fixture `ejemplos/core09/ref-mut-neg/01-vec-assign-outside.arita`, span `@812..820` = `w[0] = 3`). Another code, more than one, or a green build ⇒ rejected (Engineer pin 2026-09-27 07:42) |
| `neg-core09-ref-map-assign-non-mut` | `m[k] = v` on a non-mut binding → exactly `E0202: borrow conflict` (= `put`; unique code; fixture `ejemplos/core09/ref-mut-neg/02-map-assign-non-mut.arita`). E0314/E0205/E0001 or other ⇒ rejected (Engineer pin 2026-09-27 07:42) |
| `neg-core09-ref-vec-assign-neg-lit` | `v[-1] = x` in fn→Result → exactly `E0319: negative set index` (unique code; fixture `ejemplos/core09/ref-mut-neg/03-vec-assign-neg-lit.arita`) (Engineer pin 2026-09-27 07:42) |

**Negs (G5):** messages from ADR-281 L124/L125/L128. New fixtures in sibling folder `ejemplos/core09/ref-mut-neg/`, outside the workspace (**not** inside `ref-mut/`, whose `arita.toml` has `[workspace]` and would hijack the build); location **approved** by the Orchestrator (`ref-*-neg` precedent). Each neg: a single code, exact and unique — E0344 · E0202 · E0319. No separate dual-oracle (G4).

**Status of §2 pins:** firm (Engineer 2026-09-27 07:42), on the Measure proposal (`MEASURE_PREP_ADR285_REF_MUT_20260927.md`, md5 `05c5eab0f4d5b1f6e88353f70cdf7e1d` when pinned; current version `5a88446ffe6d61e1b0e32bd4ed7654f2` with §11 added, no pin change) and Codegen READY (`ADR-285-CODEGEN-REF-MUT-20260927.md`, md5 `3d8add5d89c75fce1ee49e7d817823e5`). Happy and OOB stdout are re-verifiable in the final run; if they differ, they are corrected with Engineer sign-off, never by hand-editing the oracle. GO IMPL condition (a) met in DOC.

N = 834 fixed by the Engineer (2026-09-27 07:42). skip ≠ PASS. **Do not** invent PASS in draft.

### 2.1 Evidence gates (Engineer pin 2026-09-27 07:42)

- **source↔evidence:** sha256 declared in `evidence.json` == real sha256 of the sources listed in `core09-ref-mut-evidence`.
- **emit determinista:** sha256 of emitted `lib.rs`/`main.rs` identical across two consecutive builds.
- **scenarios:** `scenarios` == the measured ids (`tested`), exactly the 9 of §2.
- **`measure_pass`:** informational; the oracle only requires it to be a bool (not `true`). Becomes `true` at CLOSED and the oracle is **not** touched afterward (G3).

## 3. What NOT to touch (HOLDs + post-0.9)

| Forbidden | Reason |
|-----------|--------|
| Mutex / threads / idle / TLS/WS / crates.io / repair / I/O-new / String.set | Global HOLD |
| Residual IndexMut | HOLD (E0314) |
| `?` in Io main · Option? · unwrap surface | OUT / HOLD |
| HTTP daemon / compose · MCP/ACP | outside this vertical |
| GO IMPL before CLOSED 282–284 + Engineer OK | Orchestrator gate |
| Core 0.9 CLOSED without green Lex §2 | only this slice closes 0.9 when the bar passes |
| Claim Core 0.9 CLOSED in draft | **NO** |
| Edit ROADMAP in this CUT DOC | Engineer/Orchestrator do that at CLOSED |

## 4. CLOSED criterion (slice 4 = Core 0.9 CLOSED)

Green Lex §2 with exact N = **834** (825 + 9 ids from §2; Engineer 07:42); ROADMAP/ADR-281 Core 0.9 **CLOSED**; §3 HOLDs **not** unparked (except already-landed 282/283 sugar); ADR-281 §0.3 oracle migration applied.

**Close (met 2026-09-27):** exclusive final run 834/834 + freeze 946/946 · exclusive `cargo test -p arita-cli` 154/0 · Veyra `--timeout 2400` `20260927T063410Z` **ACCEPTED** exit 0 · Engineer OK.

## 5. Post-0.9 (not this CUT)

Next vertical: only with GO <person> + ADR with oracles. HOLD candidates: String.set (UTF-8 byte/char, own ADR) · Option `?` · `?` in main Io · must-use / discard-Result (backlog D2; no E0345) · relax `put` args · split E0314 · receiver loan / params `&mut Map/Vec` (HOLD D1) · Mutex (229) · idle-kill · TLS/WS · crates.io · repair. **Do not** rewrite 0.9 mid-flight. Vertical ≠ GP ceiling.

## Checklist

- [x] Ref pins `arita-ref-mut` + oracles/evidence + what NOT to touch (DOC GO-ready)
- [x] Engineer review 2026-09-26 19:54 — pins OK · HOLD
- [x] GO IMPL (post-CLOSED 282–284) — Engineer 2026-09-27
- [x] 2026-09-27 07:41 · §2 pins from Measure proposal written (9 ids, exact stdout/codes) · proposed N 834 · pending Engineer sign-off + Codegen READY
- [x] 2026-09-27 07:42 · §2 pins firm (Engineer): N=834 without dual-oracle · G2 cli-oob exit 0 + anti-theater root bin with edge/fixture.json · G3 informative measure_pass (bool), gates §2.1 · negs in ref-mut-neg/ approved (E0344/E0202/E0319) · edge/ layout in §0 · Codegen READY
- [x] 2026-09-27 07:44 · Measure re-verifies with Codegen READY, no pin changes · `as usize` scope = `__arita_vec_set` calls/helpers (as in 283) · emit counts: 4 `__arita_vec_set(&mut …)?`, `.insert(` 6 in lib.rs and 3 in each main.rs · exactly two scenarios
- [x] 2026-09-27 08:45 · IMPL + Lex 834/834 (measure 08:05–08:20 + exclusive cargo test 154/0, freeze 946/946) + Veyra `20260927T063410Z` ACCEPTED → **CLOSED** · **Core 0.9 CLOSED** (Engineer)

## Close

- **CLOSED** (Ingeniero 2026-09-27) · Lex **834/834** · [`MEASURE_ADR285_REF_MUT_20260927.json`](../reviews/MEASURE_ADR285_REF_MUT_20260927.json) md5 `a502f785…` · cargo test 154/0 · freeze 946/946 · Veyra `20260927T063410Z` ACCEPTED exit 0 · gate [`GATE-CORE09-REF-MUT-20260926.md`](../GATE-CORE09-REF-MUT-20260926.md) · **Core 0.9 CLOSED** · post-0.9 HOLD hasta GO <person> (§5).
