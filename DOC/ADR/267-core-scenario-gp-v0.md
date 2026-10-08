# ADR-267 — Core 0.6 slice 3: SCENARIO-GP

- **Estado:** **CLOSED** Lex **720/720** (2026-09-26) — gate `DOC/GATE-CORE06-SCENARIO-GP-20260926.md`
- **CUT-ID:** `CORE-0.6-SCENARIO-GP-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-264](264-core-0.6-pins.md) §0.1 slice 3
- **Prev:** slice 2 MAP-INDEX (ADR-266) — **CLOSED** Lex **715/715** (gate `DOC/GATE-CORE06-MAP-INDEX-20260926.md`). Este CUT **no** reabre 265/266.
- **Prereq surface (al GO IMPL):** SET-FALLIBLE CLOSED (265) + MAP-INDEX CLOSED (266) en tree — `set`→`Result` + `m[k]`→`Option` + CLI/JSON/files Core 0.1 (238/240) + packages 0.4 + collections 0.5
- **HOLD:** Mutex · IndexMut assign (`v[i]=` / `m[k]=`) · idle-kill · TLS/WS · crates.io · repair · I/O H1/H2 nuevo (post-0.6) · threads · String.set

## Objetivo

Scenarios/acceptance que ejercitan **set fallible + Map index sugar + CLI/JSON ya IN** juntos (happy / OOB-Err / miss-None / neg). Measure firma sin theater. **Sin API nueva.** Puente hacia slice 4 REF-GP (`arita-ref-gp`).

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Harness** | `scenario`/`acceptance` language-level (reusar 0.1 / 239 / 262 style) |
| **Entrada** | args CLI **y/o** JSON mínimo ya curado (238/240) — **no** bindings I/O nuevos |
| **Mutación** | `List`/`Vec.set` → `Result` (270/265) · `Map.put` + `m[k]`/`get` → `Option` (266) · insert 0.5 OK |
| **Salida** | stdout determinista (print/assert) |
| **Mínimo** | ≥3 scenarios: happy set+Map+CLI/JSON · set-OOB Err y/o Map miss None · ≥1 neg (IndexMut **o** E0319 lit) |
| **Preferido** | bin CLI + stdout oracle **o** in-process scenario si emit lo permite |
| **OUT** | Mutex · net/HTTP scenarios · IndexMut assign · crates.io · path I/O H1/H2 · reopen 265/266 |

## 1. Surface

Sin keyword nueva. Reusa `scenario` + `set`→`Result` + `m[k]`→`Option` + CLI/JSON 0.1 + print/assert.

```text
// happy: CLI/JSON → List.set Ok + Map put + m[k] Some → stdout fijo
scenario gp_happy {
  // parse args o JSON mínimo (surface 238/240)
  // v.set(i, x) → Ok(()); m.put(k, y); match m[k] { Some(..) => .. }
  acceptance { /* measure stdout / assert */ }
}

// oob/miss: set OOB → Err(0) y/o m["missing"] → None; sin panic
scenario gp_oob {
  acceptance { /* measure */ }
}

// neg: v[i]= / m[k]= → IndexMut diag estable  OR  lit set(-1) → E0319
// (no reasignar E0315 — E0315 = missing field ADR-233)
```

## 2. Oracles (Measure) — éxito medible

| Id | Expect |
|----|--------|
| `core06-scen-gp-happy` | CLI/JSON → set Ok + Map `[]`/get Some → stdout determinista → **PASS** |
| `core06-scen-gp-oob` | set OOB → Err(0) y/o Map miss → None → **PASS** (no panic) |
| `neg-core06-scen-gp-index-mut` | IndexMut assign → diag estable (E0314 u vigente) |
| `neg-core06-scen-gp-set-neg` | lit `set(-1,…)` → **E0319** (si no cubierto solo en 265; skip≠PASS) |
| `core06-scen-gp-build` | measure build verde (workspace/bin si aplica) |

**Anti-theater:** skip ≠ PASS. No inventar PASS. Evidence dual-oracle donde aplique (emit-ban IndexMut en path happy).

## 3. Qué NO tocar

| Prohibido | Motivo |
|-----------|--------|
| Mutex / threads | HOLD Núcleo 2 (229) |
| `v[i]=` / `m[k]=` (IndexMut) | HOLD global; solo neg oracle |
| idle-kill · TLS/WS · crates.io · repair | HOLD intactos |
| I/O H1/H2 file/CLI OOB traps nuevas | vertical **post-0.6** (264 nota MVP) |
| Reabrir SET (265) / MAP-INDEX (266) | surface ya pinada |
| GO IMPL Codegen **antes** CLOSED 266 + OK Ingeniero | Orquestador gate |
| Declarar CLOSED Core 0.6 | cierra solo slice 4 REF-GP (268) |

## 4. Criterio CLOSED

Lex §2 verde; tick ADR-264 slice 3; HOLDs §3 intactos. Siguiente: slice 4 REF-GP (ADR-268) → cierra Core 0.6.

## Checklist

- [x] Pins scenarios set+Map+CLI/JSON + oracles + qué NO tocar (GO-listo)
- [x] GO IMPL Orquestador / Ingeniero (post-CLOSED 266)
- [x] IMPL + Lex barra **720/720** (gate `DOC/GATE-CORE06-SCENARIO-GP-20260926.md`)
- [x] Slice 4 GO → ADR-268 REF-GP (**GO IMPL**)

## Cierre

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE06-SCENARIO-GP-20260926.md`](../GATE-CORE06-SCENARIO-GP-20260926.md)
- Lex measure **720/720** accepted (~03:17 CEST; prior 715/715 +5 scenario oracles)
- Clippy workspace OK; Veyra companion REJECTED (RUSTSEC + rustfmt) **non-blocking**
- Siguiente: ADR-268 **GO IMPL** REF-GP (`CORE-0.6-REF-GP-20260926`)
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair intactos · Core 0.6 still open
