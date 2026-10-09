Translation of `046-corpus-traps-arith.md`; the original is normative. / Traducción de `046-corpus-traps-arith.md`; el original es el normativo.

# ADR-046 — Traps corpus: wave arith (DOC tick)

- **Estado:** **aceptada** (DOC-only; sin IMPL crates)
- **CUT-ID:** `CORPUS-ARITH-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto
- **Relacionados:** ADR-043 (corpus status), ADR-044 / E0216, ADR-045 / E0217
- **Gobernanza:** DOC-only. Pipeline continuo vía tick inventario; siguiente IMPL = sondeo HOLE no-control (Investigador).
- **Barra:** no inventar PASS; ports Investigación siguen `[ ]`.

## Decision

### 1. Portable wave addendum (post-043)

| Code | Tema | ADR | Measure |
|--------|------|-----|---------|
| E0216 | integer division by zero (`/` `%`) | ADR-044 | **84**+2 |
| E0217 | integer overflow (`+` `-` `*`; add/sub/mul oracles) | ADR-045 | **87**+2 |

Baseline current: **87** accepted + **2** gated (post SUBMUL).

### 2. PARK (no change)

Control-flow morphs; Mutex×await; Result swallow; more bootstrap; CI Origin.

### 3. Next IMPL

No ADR-047 crates until brief Investigator + sondeo **HOLE** (matriz as div0/overflow). Arquitecto pins ADR-047 after the matrix.

## Checklist

- [x] Inventario E0216/17 OK
- [x] GO DOC-only
- [x] No crate IMPL
