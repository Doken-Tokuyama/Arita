Translation of `284-core-scenario-mut-v0.md`; the original is normative. / Traducción de `284-core-scenario-mut-v0.md`; el original es el normativo.

# ADR-284 — Core 0.9 slice 3: SCENARIO-MUT

- **Estado:** **CLOSED** Lex **825/825** (Ingeniero 2026-09-27 · measure + `cargo test -p arita-cli` exclusivos · Veyra ACCEPTED `20260927T051022Z`) → [`GATE-CORE09-SCENARIO-MUT-20260926.md`](../GATE-CORE09-SCENARIO-MUT-20260926.md) · antes: GO IMPL 2026-09-27 tras CLOSED ADR-283
- **CUT-ID:** `CORE-0.9-SCENARIO-MUT-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero (pins OK 2026-09-26 19:54)
- **Padre:** [ADR-281](281-core-0.9-pins.md) §0.1 slice 3
- **Prev:** slice 2 VEC-ASSIGN (ADR-283) — prereq CLOSED al GO IMPL; este CUT **no** reabre 282/283
- **Prereq surface (al GO IMPL):** MAP-ASSIGN (282) + VEC-ASSIGN (283) en tree + `fn → Result` / `?` (277/278) + `set`/`put`/`get`/`[]` (265/266/237) + CLI/JSON 0.1 (238/240) si se usa entrada
- **HOLD:** Mutex · threads · idle · TLS/WS · crates.io · repair · I/O-new · String.set · IndexMut residual (E0314) · `?` in Io main · Option? · surface unwrap/expect (241) · HTTP scenarios · reopen E0272/E0291/E0340/E0341/E0342/E0343

## Goal

End-to-end scenarios/acceptance that mutate **Map + Vec** together via index-assign: happy (Map insert/overwrite + Vec in-bounds inside a helper `fn → Result<_, Int>`) · err (Vec OOB propagated as `Err(0)` → deterministic main match fail) · neg (E0344 / E0202 / E0314). Measure signs without theater. **No new API.** Bridge toward slice 4 REF-MUT (`arita-ref-mut`).

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Harness** | language-level `scenario`/`acceptance` (reuse 0.1 / 239 / 262 / 267 / 279 style) |
| **Input** | literal ints/strings **and/or** already-curated CLI args / minimal JSON (238/240) — **no** new I/O bindings |
| **Mutation** | `m[k] = v` (282) in any context · `v[i] = x` (283) **only** in a helper `fn → Result<_, Int>` · composition with `?` (278) |
| **Output** | deterministic stdout (print/assert); main `Io<()>` **match-converts** (no `?` and no `v[i] =` in main Io) |
| **Err in main** | “deterministic main match fail” = `main`’s `Err` arm: exit 0, fixed stdout, no panic (ADR-279 `02-err` precedent). The oracle also rejects if `panicked` appears on stderr (Engineer pin 05:27) |
| **Minimum** | the 9 exact ids of §2 (4 positives with pinned stdout + negs with exact code); see §2 (Engineer pin 05:27) |
| **Anti-theater** | caller does **not** swallow `Err` into a success lit (H4: exact **E0272** `result error swallowed`, as ADR-283; E0223 only if the pattern does not match the type; no reopen; no oracle in this slice) · no unwrap/expect (241) · no discard of the `Result` (must-use backlog, ADR-281 D2; no oracle) |
| **OUT** | Mutex · net/HTTP · String.set · residual IndexMut · `?` in main Io · crates.io · reopen 282/283 |

## 1. Surface

No new keyword. Reuses `scenario` + `m[k] = v` + `v[i] = x` + `fn → Result` + `?` + `[]`/`get` + print/assert.

Illustrative example: does not pin outputs. Exact stdout and ids are in §2 (Engineer pin 05:27).

```text
fn tally(i: Int, x: Int) -> Result<Int, Int> {
  let mut m: Map<Text, Int> = Map::new()
  m["hits"] = 1                    // 282 insert
  m["hits"] = 2                    // 282 overwrite
  let mut v: Vec<Int> = Vec::new()
  v.push(0)
  v.push(0)
  v[i] = x                         // 283: OOB → return Err(0)
  let n: Int = v.len()
  Ok(n)
}

scenario mut_happy {
  // tally(1, 9) → Ok → main match → fixed stdout
  acceptance { /* measure */ }
}
scenario mut_err {
  // tally(99, 9) → Err(0) propagated → main match deterministic fail; no panic
  acceptance { /* measure */ }
}
scenario mut_chain {
  // helper A (Map) ? → helper B (Vec OOB) ? → primer Err corta cadena
  acceptance { /* measure */ }
}
// neg: v[i] = x en main Io → E0344
// neg: m[k] = v on non-mut `let m` → E0202 (= put)
```

## 2. Oracles (Measure) — measurable success

Required (to be measured at GO IMPL; **none** measured in this DOC):

| Id | Expect |
|----|--------|
| `core09-scen-mut-happy` | Map insert/overwrite + Vec in-bounds in fn→Result → exact stdout `["2","9","2","mut-happy"]` + exit 0 + `arita contract` **Accepted** (Engineer pin 05:27) |
| `core09-scen-mut-err` | Vec OOB → propagated `Err(0)` → deterministic main match fail (`main`’s `Err` arm, §0 **Err in main**; no panic) → exact stdout `["0","mut-err"]` + exit 0 + `arita contract` **Accepted**; rejected if `panicked` appears on stderr (Engineer pin 05:27) |
| `core09-scen-mut-chain` | `?` chain: Map ok + Vec OOB → early-return Err; later effects do **not** occur → exact stdout `["40","err","77","err","0"]` + exit 0 + `arita contract` **Accepted**. Order: the `77` from `chain(1)` is a reachability control; **no** `77` after the third element; trailing `err`,`0` (`Err(0)` by early-return, **not** `Err(1)`). The only observable effect of `stage_post` is its return value (Engineer pin 05:27) |
| `core09-scen-mut-map-main` | `m[k] = v` directly in main `Io<()>` → OK (total) → exact stdout `["3","1","7","3"]` + exit 0 + `arita contract` **Accepted** (Engineer pin 05:27) |
| `neg-core09-scen-vec-assign-outside` | `v[i] = x` in main Io → **E0344** |
| `neg-core09-scen-map-assign-non-mut` | `m[k] = v` on a non-mut binding → **E0202** (= `put`) |
| `neg-core09-scen-unwrap-theater` | `unwrap`/`expect` on the `Result` of a `set`/assign inside a helper `fn → Result`, bound first (`let r = …; r.unwrap()`), without chaining (same pattern as 282/283) → exact **E0206** `method not in F2 std whitelist`; **Rejected** if it compiles, yields E0291, or any other code (emit-ban 241; no E0291 reopen; Engineer pin 05:27) |
| `core09-scen-mut-emit-ban` | emit grep: zero IndexMut-style Rust `x[..] =`, zero unwrap/expect/panic |
| `core09-scen-mut-build` | green measure build |

**Positives (happy / err / chain / map-main; Engineer pin 05:27):** each positive requires `arita contract` **Accepted** in addition to the pinned stdout and exit 0. The `acceptance` block declares the full real stdout, without removing or weakening the `scenario` block.

**Anti-theater:** skip ≠ PASS. **Do not** invent PASS. **Do not** use `Result` discard (E0340 is host IO; must-use → backlog; no E0345) as an oracle of this slice.

## 3. What NOT to touch

| Forbidden | Reason |
|-----------|--------|
| Mutex · threads · idle · TLS/WS · crates.io · repair · I/O-new · String.set | HOLD |
| Residual IndexMut | HOLD (E0314) |
| `?` / `v[i] =` in main Io | OUT (278 / 283 E0344) |
| GO IMPL before CLOSED 282+283 + Engineer OK | Orchestrator gate |
| Reopen E0272/E0291/E0340/E0341/E0342/E0343 | anti-collision |
| Declare Core 0.9 CLOSED | only slice 4 REF-MUT |
| Invent PASS / Lex N/N | DOC only |

## 4. CLOSED criterion

Green Lex §2 with **exact N = 825** (816 prior + the 9 ids of §2; Engineer pin 05:27); ADR-281 slice 3 tick; §3 HOLDs intact. **Close requires:** exclusive final run + freeze shasum · exclusive `cargo test -p arita-cli` · Veyra with `--timeout 2400` **ACCEPTED** exit 0 (pending; **not** CLOSED in this DOC). Next: slice 4 REF-MUT ([ADR-285](285-core-ref-mut-v0.en.md)).

## Checklist

- [x] Scenario pins Map+Vec happy/err/chain/neg + oracles (DOC GO-ready)
- [x] Engineer review 2026-09-26 19:54 — pins OK · HOLD
- [x] 2026-09-27 05:27 · pins signed off by the Engineer (05:27): unwrap-theater exact E0206 · exact stdout for 4 positives + chain order · deterministic main Err without panic · contract Accepted · N=825 · exclusive close + Veyra 2400
- [x] GO IMPL (Engineer 2026-09-27, after CLOSED ADR-283 Lex 816/816)
- [x] IMPL + Lex CLOSED — Lex 825/825 (Engineer 2026-09-27)

## Close

- **CLOSED** Lex 825/825 (Ingeniero 2026-09-27) · gate [`GATE-CORE09-SCENARIO-MUT-20260926.md`](../GATE-CORE09-SCENARIO-MUT-20260926.md) · siguiente: slice 4 REF-MUT GO IMPL · **sin** Core 0.9 CLOSED.
