Translation of `272-core-io-parse-h1-v0.md`; the original is normative. / Traducción de `272-core-io-parse-h1-v0.md`; el original es el normativo.

# ADR-272 — Core 0.7 slice 1: IO-PARSE-H1

- **Estado:** **CLOSED** Lex **733/733** (2026-09-26) · gate [`DOC/GATE-CORE07-IO-PARSE-H1-20260926.md`](../GATE-CORE07-IO-PARSE-H1-20260926.md)
- **CUT-ID:** `CORE-0.7-IO-PARSE-H1-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-271](271-core-0.7-pins.md) §0.1 slice 1 · HALLAZGOS H1
- **Prev:** Core **0.6 CLOSED** Lex **728/728** — este CUT **no** reabre 264–268
- **Prereq surface:** host `read_text`/`write_text` → `Result` (ADR-238) **ya IN**
- **HOLD:** I/O API nueva · Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · String.set · `?` / E0291 unwrap · reopen E0272/E0285

## Objective

Pin **E0340** anti-theater: after `host.read_text` / `host.write_text` (Result), forbidden to discard `Err` or substitute empty lit/`Default`/`unwrap_or` theater **without** an Err/fail arm. Measure oracles. No new host API.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Diag** | **E0340** `io result discarded` (canonical EN text) |
| **Sujeto** | `host.read_text` / `host.write_text` expression whose `Result` is ignored, `unwrap_or`/`unwrap_or_else`’d to empty lit/`""`/`0`, or matched without an `Err` arm that fails |
| **OK** | `match` / `if let` with `Err` arm → explicit fail/propagate (typed print+exit or return Err) **without** inventing `?` |
| **Emit** | happy path: no `.unwrap()`/`.expect(`/panic (241) |
| **OUT** | new `host.json_parse` · Path API · reopen E0272 (generic Result) / E0285 (checked+default) |
| **No reabre** | 0.6 set/Map · insert/`[]` collections |

## 1. Surface example

```text
// POS — Err visible
match host.read_text(path) {
  Ok(t) => print(t),
  Err(_) => print("read_failed")   // deterministic fail (measure)
}

// NEG → E0340
let _ = host.read_text(path)              // discard
let t = host.read_text(path).unwrap_or("")  // default-as-success theater
match host.read_text(path) { Ok(t) => ..., Err(_) => "" }  // Err → empty-lit theater
```

`host.json_get_int` miss → Option: **outside** this slice (goes to H2 / E0341 slice 2) unless a host parse Result already exists IN (do not invent).

## 2. Oracles

| Id | Expect |
|----|--------|
| `core07-io-h1-read-ok` | read Ok → expected stdout → PASS |
| `core07-io-h1-read-err` | missing file → deterministic Err-arm fail → PASS (no panic) |
| `neg-core07-io-h1-discard` | discard Result → **E0340** |
| `neg-core07-io-h1-default` | unwrap_or/empty lit after read → **E0340** |
| `core07-io-h1-emit-ban` | emit grep: zero unwrap/expect/panic on user path (241) |

## 3. E0340 pin

**E0340** `io result discarded` — only host IO `Result` (read/write) discard or default-as-success.
**Do not** reassign E0272 / E0285 / E0315 / E0319.

## 4. CLOSED criterion

Lex §2 green; tick ADR-271 slice 1; HOLDs intact. Next: slice 2 CLI-ARGV-H2 ([ADR-273](273-core-cli-argv-h2-v0.en.md) DOC GO-ready).

## Checklist

- [x] E0340 pins + oracles (GO-ready)
- [x] GO IMPL Orchestrator / Engineer (after GO <person> “continue”)
- [x] IMPL + Lex (Codegen Lex smoke 5/5; measure_pass:false → Measure)
- [x] Slice 2 → ADR-273 **CLOSED** Lex **739/739** (pins already GO-ready)

## Close

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE07-IO-PARSE-H1-20260926.md`](../GATE-CORE07-IO-PARSE-H1-20260926.md)
- Lex measure **733/733** accepted
- Clippy workspace OK; Veyra companion REJECTED (RUSTSEC + rustfmt) **non-blocking**
- Next (historic): ADR-273 CLI-ARGV-H2 — **CLOSED** Lex **739/739** (Core **0.7 CLOSED** vertical via 275 Lex **752/752**)
- Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair/I/O-new/String.set/`?` HOLDs intact
