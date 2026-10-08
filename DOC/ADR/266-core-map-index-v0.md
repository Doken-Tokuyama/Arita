# ADR-266 — Core 0.6 slice 2: MAP-INDEX

- **Estado:** **CLOSED** Lex **715/715** (2026-09-26) — gate [`DOC/GATE-CORE06-MAP-INDEX-20260926.md`](../GATE-CORE06-MAP-INDEX-20260926.md)
- **CUT-ID:** `CORE-0.6-MAP-INDEX-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-264](264-core-0.6-pins.md) §0.1 slice 2 · contrato [ADR-227](227-lang-contract-indexget.md) extendido a `Map`
- **Prev:** slice 1 SET-FALLIBLE (ADR-265) — **CLOSED** Lex **710/710** (gate `DOC/GATE-CORE06-SET-FALLIBLE-20260926.md`)
- **Prereq surface:** Map `put`/`get` IN (ADR-237); Vec/String index sugar CLOSED (ADR-261)
- **HOLD:** IndexMut assign (`m[k]=` / `v[i]=`) · Mutex · idle · TLS/WS · crates.io · repair · String.set · path I/O nuevo (I/O H1/H2 → vertical post-0.6)

## Objetivo

Sugar `m[k]` → **misma** semántica que `m.get(k)` → `Option`. Emit idéntico a `get`. **Prohibido** Index/IndexMut panic. Simetría con Vec/String sugar (261). Sin API nueva de escritura Map (sigue `put`).

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Desugar** | `m[k]` ⇒ `m.get(k)` → `Option` (claves Text/String v0) |
| **Miss / tipo** | `None` (igual que get); sin panic |
| **Emit** | solo path `get` / Option — **cero** `map[k]`, `Index`, `IndexMut`, `unwrap` |
| **OUT** | `m[k] = v` (IndexMut) · sugar sobre no-Map · Mutex |
| **No reabre** | SET (265) · insert (260) · Vec `[]` (261) |

## 1. Surface

```text
let mut m = Map.new()
m.put("a", 7)
match m["a"] { Some(x) => print(x), None => print(0) }   // Some(7)
match m["z"] { Some(_) => ..., None => ... }            // None
// m["a"] = 1  → reject IndexMut (diag estable, p.ej. E0314)
```

`get(k)` permanece API canónica; sugar no segunda semántica.

## 2. Oracles

| Id | Expect |
|----|--------|
| `core06-map-index-some` | `m[k]` hit → `Some` (= get) |
| `core06-map-index-none` | miss → `None` (no panic) |
| `core06-map-index-eq-get` | golden: `m[k]` ≡ `m.get(k)` |
| `core06-map-emit-ban` | emit grep: cero Index/IndexMut / panic `[]` en Map |
| `neg-core06-map-index-mut` | `m[k] = …` → diag estable |

## 3. Criterio CLOSED

Lex §2 verde; tick ADR-264 slice 2; HOLDs IndexMut/Mutex/I/O-new intactos. Siguiente: slice 3 SCENARIO-GP.

## Anti-trampas (DOC; Measure)

No reabrir E0272/E0285/E0225/E0291. H3 Result check-then-unwrap: si aparece en examples, trap DOC (no unpark `?`). H1/H2 file/CLI OOB: **fuera** de este slice (vertical I/O post-0.6).

## Checklist

- [x] Pins `m[k]`→get/Option + emit-ban + oracles (GO-listo)
- [x] GO IMPL Orquestador (post gate 265 CLOSED)
- [x] IMPL + Lex **CLOSED** 715/715
- [x] Slice 3 GO IMPL (ADR-267)


## Cierre

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE06-MAP-INDEX-20260926.md`](../GATE-CORE06-MAP-INDEX-20260926.md)
- Lex measure **715/715** accepted (~03:00 CEST; prior 710/710 +5 map oracles)
- Clippy workspace OK; Veyra companion REJECTED (RUSTSEC + rustfmt) **non-blocking**
- Siguiente: ADR-267 **GO IMPL** SCENARIO-GP (`CORE-0.6-SCENARIO-GP-20260926`)
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair intactos · Core 0.6 still open
