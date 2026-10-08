Translation of `273-core-cli-argv-h2-v0.md`; the original is normative. / Traducción de `273-core-cli-argv-h2-v0.md`; el original es el normativo.

# ADR-273 — Core 0.7 slice 2: CLI-ARGV-H2

- **Estado:** **CLOSED** Lex **739/739** (2026-09-26 remasure) · gate [`DOC/GATE-CORE07-CLI-ARGV-H2-20260926.md`](../GATE-CORE07-CLI-ARGV-H2-20260926.md) · FIX E0341-CORE06
- **CUT-ID:** `CORE-0.7-CLI-ARGV-H2-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-271](271-core-0.7-pins.md) §0.1 slice 2 · HALLAZGOS H2
- **Prev:** slice 1 IO-PARSE-H1 (ADR-272) **CLOSED** Lex **733/733**; este CUT **no** reabre 272
- **Prereq surface (al GO IMPL):** host `cli_arg` → `Option` + `json_get_int` → `Option` (ADR-238) **ya IN** + E0340 land (272 CLOSED)
- **HOLD:** I/O API nueva · `args[i]` / Index argv · IndexMut-assign · Mutex · idle · TLS/WS · crates.io · repair · String.set · `?` / E0291 · reopen E0287 (Vec get+default) / E0340

## Objective

Pin **E0341** anti-theater: a **required** argv/JSON value cannot theater-succeed on miss (`None` → default/empty lit without fail). Prefer `host.cli_arg` / `json_get_int` + match None→fail. **Forbidden** to index argv with `[]`. No new host API.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Diag** | **E0341** `required arg miss as ok` (canonical EN text) |
| **Sujeto** | (a) `host.cli_arg(i)` / `host.json_get_int(doc,key)` on a **required** path: `None` → default lit/`""`/`0` / `unwrap_or` without fail; (b) argv indexing `args[i]` / `[]` on args |
| **OK** | `match` / `if let` None → deterministic usage/fail; optional args explicitly DOC’d outside E0341 |
| **Emit** | only `cli_arg`/`json_get_*` Option path — **zero** `Index`/`args[i]` panic |
| **OUT** | new argv parser API · IndexMut · reopen E0287 (collections) · E0340 (IO Result) |
| **No reabre** | SET/MAP 0.6 · IO-PARSE-H1 (272) |

## 1. Surface example

```text
// POS — required arg / key
match host.cli_arg(1) {
  Some(p) => ...,
  None => print("usage")   // deterministic fail
}
match host.json_get_int(doc, "n") {
  Some(n) => ...,
  None => print("missing_n")
}

// NEG → E0341
let p = host.cli_arg(1).unwrap_or("")     // miss-as-ok theater
let n = host.json_get_int(doc, "n").unwrap_or(0)
// args[1] / argv[i] → E0341 (or E0310 index-ban if already applies; preferred pin E0341 on CLI)
```

Optional flags (miss = default **documented** as optional) are **outside** E0341 — Measure DOC marks required vs optional in the oracle.

## 2. Oracles

| Id | Expect |
|----|--------|
| `core07-io-h2-arg-ok` | cli_arg Some → expected stdout → PASS |
| `core07-io-h2-arg-miss` | required miss → deterministic None-arm fail → PASS |
| `core07-io-h2-json-miss` | required json_get_int None → deterministic fail → PASS |
| `neg-core07-io-h2-default` | unwrap_or/lit after required cli_arg/json_get → **E0341** |
| `neg-core07-io-h2-index` | `args[i]` / Index argv → **E0341** (or stable documented index-ban diag) |
| `core07-io-h2-emit-ban` | emit grep: zero Index/argv`[]` / unwrap theater |

## 3. E0341 pin

**E0341** `required arg miss as ok` — only host CLI/JSON **required** Option miss-as-ok or argv Index.
**Do not** reassign E0287 / E0340 / E0310 (if E0310 already covers the generic index ban, oracle `neg-core07-io-h2-index` may defer to E0310 — document in the gate; slice preference = E0341 on the CLI subject).

## 4. What NOT to touch

| Forbidden | Reason |
|-----------|--------|
| New I/O bindings | HOLD H1/H2 *new* |
| Mutex / IndexMut-assign / idle / TLS/WS / crates.io / repair | Global HOLD |
| GO IMPL before CLOSED 272 + Engineer OK | Orchestrator gate |
| Reopen E0340 / E0272 / E0285 / E0287 | anti-collision |

## 5. CLOSED criterion

Lex §2 green; tick ADR-271 slice 2; §4 HOLDs intact. Next: slice 3 SCENARIO-IO ([ADR-274](274-core-scenario-io-v0.en.md) GO-ready) → slice 4 [ADR-275](275-core-ref-io-v0.en.md).

## Checklist

- [x] E0341 pins + oracles + what NOT to touch (GO-ready)
- [x] GO IMPL Orchestrator / Engineer (post-CLOSED 272)
- [x] IMPL + Lex **739/739** CLOSED (remeasure)
- [x] Slice 3 GO (ADR-274) IMPL → ADR-274 (pins already GO-ready)

## Close

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE07-CLI-ARGV-H2-20260926.md`](../GATE-CORE07-CLI-ARGV-H2-20260926.md) · remasure post FIX E0341-CORE06
- Lex measure **739/739** accepted
- Clippy workspace OK; Veyra companion REJECTED (RUSTSEC + rustfmt) **non-blocking**
- Next (historic): ADR-274 SCENARIO-IO — **CLOSED** Lex **744/744** (Core **0.7 CLOSED** vertical via 275 Lex **752/752**)
- Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair/I/O-new/String.set/`?` HOLDs intact · E0341 intact
