Translation of `262-core-scenario-coll-v0.md`; the original is normative. / Traducción de `262-core-scenario-coll-v0.md`; el original es el normativo.

# ADR-262 — Core 0.5 slice 3: SCENARIO-COLL

- **Estado:** **CLOSED** Lex **705/705** (2026-09-25 ~20:53 CEST) · gate `DOC/GATE-CORE05-SCENARIO-COLL-20260925.md`
- **CUT-ID:** `CORE-0.5-SCENARIO-COLL-20260920`
- **Fecha:** 2026-09-20 · **GO IMPL:** 2026-09-25 (Orquestador: Codegen land INDEX-SUGAR Lex OK; gap pins)
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-259](259-core-0.5-pins.md) §0.1 slice 3
- **Prev:** slice 2 INDEX-SUGAR (ADR-261) — land Codegen Lex OK; Measure re-run en curso; **no** reabre 261
- **Prereq surface:** INSERT (ADR-260 CLOSED) + INDEX-SUGAR (ADR-261) en tree
- **HOLD:** Mutex · IndexMut · idle-kill · TLS/WS · crates.io · repair · HTTP scenarios

## Objective

Scenarios/acceptance that exercise **fallible insert + index sugar** together (happy / OOB None / neg). Measure signs without theater. No new API.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Harness** | `scenario`/`acceptance` language-level (reuse 0.1 / 257 style) |
| **Minimum** | ≥3 scenarios: insert-ok+index-Some · insert-OOB/Err or index-None · ≥1 neg (IndexMut or E0311) |
| **Preferido** | CLI bin + stdout oracle **or** in-process scenario if emit allows it |
| **OUT** | Mutex · net/HTTP · IndexMut assign · crates.io |

## 1. Surface

No new keyword. Reuses `scenario` + `Vec.insert`→`Result` + `v[i]`→`Option` + print/assert.

```text
scenario coll_happy {
  // insert in-range Ok; v[i] Some; stdout deterministic
  acceptance { /* measure */ }
}
scenario coll_oob {
  // insert OOB → Err(0) and/or v[999] → None; no panic
  acceptance { /* measure */ }
}
// neg: v[i] = x → E0314 (or pinned diag)  OR  insert(-1 lit) → E0311
```

## 2. Oracles

| Id | Expect |
|----|--------|
| `core05-scen-coll-happy` | insert Ok + index Some → PASS |
| `core05-scen-coll-oob` | OOB insert Err and/or index None → PASS (no panic) |
| `neg-core05-scen-coll-index-mut` | IndexMut → stable diag |
| `neg-core05-scen-coll-insert-neg` | lit i<0 insert → E0311 (if not covered only in 260) |
| `core05-scen-coll-build` | measure build green |

## 3. CLOSED criterion

Lex §2 green; tick ADR-259 slice 3; HOLDs intact. Next: slice 4 REF-COLL (closes Core 0.5).

## Checklist

- [x] Pins scenarios insert+index + oracles
- [x] GO IMPL Orchestrator / Architect (2026-09-25)
- [x] IMPL + Lex **705/705** CLOSED
- [x] Slice 4 GO REF-COLL

## Close

- **GO Ingeniero** 2026-09-25 · gate [`DOC/GATE-CORE05-SCENARIO-COLL-20260925.md`](../GATE-CORE05-SCENARIO-COLL-20260925.md)
- Lex measure **705/705** accepted
- Clippy workspace OK; Veyra companion REJECTED (bytes RUSTSEC + rustfmt) **non-blocking**
- HOLDs Mutex/IndexMut/idle/TLS/WS/crates.io/repair intactos
