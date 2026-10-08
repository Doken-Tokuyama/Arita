Translation of `277-core-fn-result-v0.md`; the original is normative. / Traducción de `277-core-fn-result-v0.md`; el original es el normativo.

# ADR-277 — Core 0.8 slice 1: FN-RESULT

- **Estado:** **CLOSED** Lex **758/758** (2026-09-26) · gate [`DOC/GATE-CORE08-FN-RESULT-20260926.md`](../GATE-CORE08-FN-RESULT-20260926.md)
- **CUT-ID:** `CORE-0.8-FN-RESULT-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero (OK 2026-09-26) · Orquestador (asigna Codegen/Measure)
- **Padre:** [ADR-276](276-core-0.8-pins.md) §0.1 slice 1 · HALLAZGOS H3
- **Prev:** Core **0.7 CLOSED** Lex **752/752** — este CUT **no** reabre 271–275
- **Prereq surface:** `Result<T,E>` match + Ok/Err (ADR-047) **ya IN** · `fn main() -> Io<()>` **ya IN**
- **HOLD:** Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · I/O-new · String.set · threads · `?` (**slice 2**) · surface unwrap/expect (241) · Option? · async Result · reopen E0272/E0340/E0341/E0291

## Goal

Unpark **`fn name(...) -> Result<T,E>`** (was OUT ADR-047 for a return usable in propagate). Surface: fn→Result + **match** propagate **without** `?` yet. Oracles: Result-returning fn works; neg invent unwrap / bad return theater → **E0342**. No new host API. No `?` (goes to slice 2).

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **IN** | `fn name(params) -> Result<T,E> { … }` with `T,E` aligned to ADR-047; body returns `Ok`/`Err` or match-propagates |
| **Keep** | `fn main() -> Io<()>` unchanged; main does **not** require Result |
| **Diag** | **E0342** `result return type required` (canonical EN text) — theater that fakes propagate without a Result return (e.g. claim Err path without Result type; return-type mismatch theater) |
| **OK** | helper `fn → Result` + caller `match` Ok/Err; Rust emit `Result` without unwrap |
| **Emit** | happy/fail path: **zero** `.unwrap()`/`.expect(`/panic (241) |
| **OUT** | `?` (slice 2) · invent surface `unwrap`/`expect` method · async Result · `Option?` · `?` in Io main |
| **Does not reopen** | E0272 / E0340 / E0341 / E0291 · Core 0.7 H1/H2 |

## 1. Surface example

```text
// POS — fn → Result + match propagate (no ? yet)
fn load(path: String) -> Result<String, Int> {
  match host.read_text(path) {
    Ok(t) => Ok(t),
    Err(_) => Err(1)
  }
}

fn main() -> Io<()> {
  match load("a.txt") {
    Ok(t) => print(t),
    Err(_) => print("load_failed")
  }
}

// NEG → E0342 (bad return theater) — ejemplos Measure
// fn load(...) -> Int { Err(1) }           // Err non-Result return
// fn load(...) -> Result<String, Int> { 0 } // return lit no Result
// invent: r.unwrap() / r.expect("x")         // surface method → reject (+ emit-ban 241)
```

## 2. Oracles

| Id | Expect |
|----|--------|
| `core08-fn-result-ok` | helper `fn → Result` Ok path → expected stdout → PASS |
| `core08-fn-result-err` | helper Err → deterministic caller match fail → PASS (no panic) |
| `core08-fn-result-main-io` | `fn main() -> Io<()>` stays OK calling a Result helper via match |
| `neg-core08-fn-result-theater` | bad return / Err without Result type → **E0342** |
| `neg-core08-fn-result-unwrap` | invent surface unwrap/expect method → reject / stable diag (≠ E0291 reopen; document at gate) |
| `core08-fn-result-emit-ban` | emit grep: zero unwrap/expect/panic on the user path (241) |

## 3. E0342 pin

**E0342** `result return type required` — propagate/return theater without a typed `Result` where the pin requires `fn → Result`, or a return payload incompatible with Ok/Err.
**Do not** reassign E0272 / E0340 / E0341 / E0291 / E0270 / E0271.

**Soft note (Engineer):** **E0342** only for theater pinned in this ADR — **do not** retag generic type-errors already covered (**E0203**/etc.).

## 4. What NOT to touch

| Forbidden | Reason |
|-----------|--------|
| `?` desugar | slice 2 ADR-278 |
| Mutex / IndexMut / idle / TLS/WS / crates.io / repair / I/O-new / String.set / threads | Global HOLD |
| Surface unwrap/expect | emit-ban 241 HOLD |
| GO IMPL authorized; IMPL/Lex pending | coordination |
| Reopen E0272 / E0340 / E0341 | anti-collision |
| Invent PASS / Lex N/N | DOC only |

## 5. CLOSED criterion

Green Lex §2; ADR-276 slice 1 tick; §4 HOLDs intact. Next: slice 2 QMARK ([ADR-278](278-core-question-mark-v0.en.md)).

## Checklist

- [x] Pins `fn → Result` + E0342 + oracles (DOC GO-ready)
- [x] GO IMPL authorized — Engineer OK 2026-09-26; Orchestrator assigns Codegen/Measure
- [x] IMPL + Lex **758/758** CLOSED (Measure)
- [ ] Slice 2 → ADR-278 (pins already DOC GO-ready; **GO IMPL** post-CLOSED 277)

## Close

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE08-FN-RESULT-20260926.md`](../GATE-CORE08-FN-RESULT-20260926.md)
- Lex measure **758/758** accepted → slice 1 **CLOSED** (evidence `DOC/reviews/MEASURE_ADR277_FN_RESULT_20260926.json`; exit 0)
- `measure_pass:false` pre-CLOSED was “await Measure” metadata (not a material fail); flipped `true` on CLOSED
- Clippy workspace OK; Veyra companion **REJECTED** exit 1 (`rustfmt-diff` + VT*) **non-blocking** · `.veyra/evidence/20260926T132822Z/`
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair/I/O-new/String.set/threads/unwrap-surface/`?` **not** unparked
- Core **0.8** still open · Next: **GO IMPL** ADR-278 QMARK only (279–280 HOLD)
