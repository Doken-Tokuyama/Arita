Translation of `267-core-scenario-gp-v0.md`; the original is normative. / Traducción de `267-core-scenario-gp-v0.md`; el original es el normativo.

# ADR-267 — Core 0.6 slice 3: SCENARIO-GP

- **Estado:** **CLOSED** Lex **720/720** (2026-09-26) — gate `DOC/GATE-CORE06-SCENARIO-GP-20260926.md`
- **CUT-ID:** `CORE-0.6-SCENARIO-GP-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-264](264-core-0.6-pins.md) §0.1 slice 3
- **Prev:** slice 2 MAP-INDEX (ADR-266) — **CLOSED** Lex **715/715** (gate `DOC/GATE-CORE06-MAP-INDEX-20260926.md`). Este CUT **no** reabre 265/266.
- **Prereq surface (al GO IMPL):** SET-FALLIBLE CLOSED (265) + MAP-INDEX CLOSED (266) en tree — `set`→`Result` + `m[k]`→`Option` + CLI/JSON/files Core 0.1 (238/240) + packages 0.4 + collections 0.5
- **HOLD:** Mutex · IndexMut assign (`v[i]=` / `m[k]=`) · idle-kill · TLS/WS · crates.io · repair · I/O H1/H2 nuevo (post-0.6) · threads · String.set

## Objective

Scenarios/acceptance that exercise **fallible set + Map index sugar + already-IN CLI/JSON** together (happy / OOB-Err / miss-None / neg). Measure signs without theater. **No new API.** Bridge toward slice 4 REF-GP (`arita-ref-gp`).

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Harness** | `scenario`/`acceptance` language-level (reuse 0.1 / 239 / 262 style) |
| **Entrada** | CLI args **and/or** already-curated minimal JSON (238/240) — **no** new I/O bindings |
| **Mutation** | `List`/`Vec.set` → `Result` (270/265) · `Map.put` + `m[k]`/`get` → `Option` (266) · insert 0.5 OK |
| **Salida** | deterministic stdout (print/assert) |
| **Minimum** | ≥3 scenarios: happy set+Map+CLI/JSON · set-OOB Err and/or Map miss None · ≥1 neg (IndexMut **or** E0319 lit) |
| **Preferido** | CLI bin + stdout oracle **or** in-process scenario if emit allows it |
| **OUT** | Mutex · net/HTTP scenarios · IndexMut assign · crates.io · path I/O H1/H2 · reopen 265/266 |

## 1. Surface

No new keyword. Reuses `scenario` + `set`→`Result` + `m[k]`→`Option` + CLI/JSON 0.1 + print/assert.

```text
// happy: CLI/JSON → List.set Ok + Map put + m[k] Some → fixed stdout
scenario gp_happy {
  // parse args or JSON minimal (surface 238/240)
  // v.set(i, x) → Ok(()); m.put(k, y); match m[k] { Some(..) => .. }
  acceptance { /* measure stdout / assert */ }
}

// oob/miss: set OOB → Err(0) and/or m["missing"] → None; no panic
scenario gp_oob {
  acceptance { /* measure */ }
}

// neg: v[i]= / m[k]= → IndexMut stable diag  OR  lit set(-1) → E0319
// (do not reassign E0315 — E0315 = missing field ADR-233)
```

## 2. Oracles (Measure) — measurable success

| Id | Expect |
|----|--------|
| `core06-scen-gp-happy` | CLI/JSON → set Ok + Map `[]`/get Some → deterministic stdout → **PASS** |
| `core06-scen-gp-oob` | set OOB → Err(0) and/or Map miss → None → **PASS** (no panic) |
| `neg-core06-scen-gp-index-mut` | IndexMut assign → stable diag (E0314 or current) |
| `neg-core06-scen-gp-set-neg` | lit `set(-1,…)` → **E0319** (if not covered only in 265; skip≠PASS) |
| `core06-scen-gp-build` | measure build green (workspace/bin si aplica) |

**Anti-theater:** skip ≠ PASS. Do not invent PASS. Dual-oracle evidence where it applies (IndexMut emit-ban on the happy path).

## 3. What NOT to touch

| Forbidden | Reason |
|-----------|--------|
| Mutex / threads | HOLD Core 2 (229) |
| `v[i]=` / `m[k]=` (IndexMut) | global HOLD; neg oracle only |
| idle-kill · TLS/WS · crates.io · repair | HOLD intactos |
| New I/O H1/H2 file/CLI OOB traps | **post-0.6** vertical (264 MVP note) |
| Reopen SET (265) / MAP-INDEX (266) | surface already pinned |
| GO IMPL Codegen **before** CLOSED 266 + OK Engineer | Orchestrator gate |
| Declare Core 0.6 CLOSED | only slice 4 REF-GP (268) closes it |

## 4. CLOSED criterion

Lex §2 green; tick ADR-264 slice 3; HOLDs §3 intactos. Next: slice 4 REF-GP (ADR-268) → closes Core 0.6.

## Checklist

- [x] Pins scenarios set+Map+CLI/JSON + oracles + what NOT to touch (GO-ready)
- [x] GO IMPL Orchestrator / Engineer (post-CLOSED 266)
- [x] IMPL + Lex bar **720/720** (gate `DOC/GATE-CORE06-SCENARIO-GP-20260926.md`)
- [x] Slice 4 GO → ADR-268 REF-GP (**GO IMPL**)

## Close

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE06-SCENARIO-GP-20260926.md`](../GATE-CORE06-SCENARIO-GP-20260926.md)
- Lex measure **720/720** accepted (~03:17 CEST; prior 715/715 +5 scenario oracles)
- Clippy workspace OK; Veyra companion REJECTED (RUSTSEC + rustfmt) **non-blocking**
- Next: ADR-268 **GO IMPL** REF-GP (`CORE-0.6-REF-GP-20260926`)
- Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair HOLDs intact · Core 0.6 still open
