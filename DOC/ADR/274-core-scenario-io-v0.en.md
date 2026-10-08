Translation of `274-core-scenario-io-v0.md`; the original is normative. / Traducción de `274-core-scenario-io-v0.md`; el original es el normativo.

# ADR-274 — Core 0.7 slice 3: SCENARIO-IO

- **Estado:** **CLOSED** Lex **744/744** (2026-09-26) · gate [`DOC/GATE-CORE07-SCENARIO-IO-20260926.md`](../GATE-CORE07-SCENARIO-IO-20260926.md)
- **CUT-ID:** `CORE-0.7-SCENARIO-IO-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-271](271-core-0.7-pins.md) §0.1 slice 3
- **Prev:** slice 2 CLI-ARGV-H2 (ADR-273) — **CLOSED** Lex **739/739**; este CUT **no** reabre 272/273
- **Prereq surface (al GO IMPL):** E0340 (272) + E0341 (273) en tree + host ADR-238
- **HOLD:** I/O API nueva · Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · String.set · `?` · HTTP scenarios · reopen 0.6 set/Map

## Objective

Scenarios/acceptance that exercise **H1+H2 together** on host 238 (happy read+args/JSON · Err/None fail · neg E0340/E0341). Measure signs without theater. **No new API.** Bridge toward slice 4 REF-IO.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Harness** | language-level `scenario`/`acceptance` (reuse 0.1 / 267 / 262 style) |
| **Entrada** | file path + **required** args/JSON via host 238 only |
| **Mínimo** | ≥3 scenarios: happy IO+CLI/JSON · read Err and/or required miss fail · ≥1 neg (E0340 **or** E0341) |
| **Preferido** | CLI bin + stdout oracle **or** in-process scenario if emit allows |
| **OUT** | new host APIs · Mutex · net/HTTP · IndexMut · crates.io |

## 1. Surface

No new keyword. Reuses `scenario` + `host.read_text`/`write_text` Result + `cli_arg`/`json_get_int` Option + print/assert.

```text
scenario io_happy {
  // cli_arg Some + read Ok (+ json_get Some) → fixed stdout
  acceptance { /* measure */ }
}
scenario io_fail {
  // read Err and/or required miss None → deterministic fail; no panic
  acceptance { /* measure */ }
}
// neg: discard Result → E0340  OR  unwrap_or after cli_arg required → E0341
```

## 2. Oracles (Measure) — measurable success

| Id | Expect |
|----|--------|
| `core07-scen-io-happy` | args/JSON + read Ok → deterministic stdout → **PASS** |
| `core07-scen-io-fail` | read Err and/or required miss → fail path → **PASS** (no panic) |
| `neg-core07-scen-io-h1` | discard/default Result → **E0340** (if not covered only in 272) |
| `neg-core07-scen-io-h2` | required miss-as-ok / argv Index → **E0341** (if not covered only in 273) |
| `core07-scen-io-build` | green measure build |

**Anti-theater:** skip ≠ PASS. Do not invent PASS.

## 3. What NOT to touch

| Forbidden | Reason |
|-----------|--------|
| New I/O bindings · Mutex · IndexMut · idle · TLS/WS · crates.io · repair | HOLD |
| GO IMPL before CLOSED 272+273 + Engineer OK | Orchestrator gate |
| Reopen E0340/E0341 surface pins | already in 272/273 |
| Declare Core 0.7 CLOSED | only slice 4 REF-IO |

## 4. CLOSED criterion

Lex §2 green; tick ADR-271 slice 3; §3 HOLDs intact. Next: slice 4 REF-IO (ADR-275).

## Checklist

- [x] H1+H2 scenario pins + oracles (GO-ready)
- [x] GO IMPL Orchestrator / Engineer (post-CLOSED 272+273)
- [x] IMPL + Lex **744/744** CLOSED (Measure)
- [x] Slice 4 → ADR-275 **CLOSED** Lex **752/752** = Core **0.7 CLOSED**

## Close

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE07-SCENARIO-IO-20260926.md`](../GATE-CORE07-SCENARIO-IO-20260926.md)
- Lex measure **744/744** accepted
- Clippy workspace OK; Veyra companion REJECTED (RUSTSEC + rustfmt) **non-blocking**
- Next (historic): ADR-275 REF-IO — **CLOSED** Lex **752/752** = **Core 0.7 CLOSED**
- Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair/I/O-new/String.set/`?` HOLDs intact
