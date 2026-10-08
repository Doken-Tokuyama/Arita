Translation of `266-core-map-index-v0.md`; the original is normative. / Traducción de `266-core-map-index-v0.md`; el original es el normativo.

# ADR-266 — Core 0.6 slice 2: MAP-INDEX

- **Estado:** **CLOSED** Lex **715/715** (2026-09-26) — gate [`DOC/GATE-CORE06-MAP-INDEX-20260926.md`](../GATE-CORE06-MAP-INDEX-20260926.md)
- **CUT-ID:** `CORE-0.6-MAP-INDEX-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-264](264-core-0.6-pins.md) §0.1 slice 2 · contrato [ADR-227](227-lang-contract-indexget.md) extendido a `Map`
- **Prev:** slice 1 SET-FALLIBLE (ADR-265) — **CLOSED** Lex **710/710** (gate `DOC/GATE-CORE06-SET-FALLIBLE-20260926.md`)
- **Prereq surface:** Map `put`/`get` IN (ADR-237); Vec/String index sugar CLOSED (ADR-261)
- **HOLD:** IndexMut assign (`m[k]=` / `v[i]=`) · Mutex · idle · TLS/WS · crates.io · repair · String.set · path I/O nuevo (I/O H1/H2 → vertical post-0.6)

## Objective

Sugar `m[k]` → **same** semantics as `m.get(k)` → `Option`. Emit identical to `get`. **Forbidden** Index/IndexMut panic. Symmetry with Vec/String sugar (261). No new Map write API (`put` stays).

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Desugar** | `m[k]` ⇒ `m.get(k)` → `Option` (Text/String keys v0) |
| **Miss / tipo** | `None` (same as get); no panic |
| **Emit** | only path `get` / Option — **cero** `map[k]`, `Index`, `IndexMut`, `unwrap` |
| **OUT** | `m[k] = v` (IndexMut) · sugar on non-Map · Mutex |
| **No reabre** | SET (265) · insert (260) · Vec `[]` (261) |

## 1. Surface

```text
let mut m = Map.new()
m.put("a", 7)
match m["a"] { Some(x) => print(x), None => print(0) }   // Some(7)
match m["z"] { Some(_) => ..., None => ... }            // None
// m["a"] = 1  → reject IndexMut (stable diag, e.g. E0314)
```

`get(k)` remains the canonical API; sugar not a second semantics.

## 2. Oracles

| Id | Expect |
|----|--------|
| `core06-map-index-some` | `m[k]` hit → `Some` (= get) |
| `core06-map-index-none` | miss → `None` (no panic) |
| `core06-map-index-eq-get` | golden: `m[k]` ≡ `m.get(k)` |
| `core06-map-emit-ban` | emit grep: zero Index/IndexMut / panic `[]` en Map |
| `neg-core06-map-index-mut` | `m[k] = …` → stable diag |

## 3. CLOSED criterion

Lex §2 green; tick ADR-264 slice 2; HOLDs IndexMut/Mutex/I/O-new intact. Next: slice 3 SCENARIO-GP.

## Anti-traps (DOC; Measure)

Do not reopen E0272/E0285/E0225/E0291. H3 Result check-then-unwrap: if it appears in examples, DOC trap (do not unpark `?`). H1/H2 file/CLI OOB: **outside** this slice (post-0.6 I/O vertical).

## Checklist

- [x] Pins `m[k]`→get/Option + emit-ban + oracles (GO-ready)
- [x] GO IMPL Orchestrator (after gate 265 CLOSED)
- [x] IMPL + Lex **CLOSED** 715/715
- [x] Slice 3 GO IMPL (ADR-267)


## Close

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE06-MAP-INDEX-20260926.md`](../GATE-CORE06-MAP-INDEX-20260926.md)
- Lex measure **715/715** accepted (~03:00 CEST; prior 710/710 +5 map oracles)
- Clippy workspace OK; Veyra companion REJECTED (RUSTSEC + rustfmt) **non-blocking**
- Next: ADR-267 **GO IMPL** SCENARIO-GP (`CORE-0.6-SCENARIO-GP-20260926`)
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair intact · Core 0.6 still open
