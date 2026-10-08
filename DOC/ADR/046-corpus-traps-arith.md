# ADR-046 — Corpus trampas: oleada arith (DOC tick)

- **Estado:** **aceptada** (DOC-only; sin IMPL crates)
- **CUT-ID:** `CORPUS-ARITH-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto
- **Relacionados:** ADR-043 (corpus status), ADR-044 / E0216, ADR-045 / E0217
- **Gobernanza:** DOC-only. Pipeline continuo vía tick inventario; siguiente IMPL = sondeo HOLE no-control (Investigador).
- **Barra:** no inventar PASS; ports Investigación siguen `[ ]`.

## Decisión

### 1. Addendum oleada portable (post-043)

| Código | Tema | ADR | Measure |
|--------|------|-----|---------|
| E0216 | integer division by zero (`/` `%`) | ADR-044 | **84**+2 |
| E0217 | integer overflow (`+` `-` `*`; add/sub/mul oracles) | ADR-045 | **87**+2 |

Baseline actual: **87** accepted + **2** gated (post SUBMUL).

### 2. PARK (sin cambio)

Control-flow morphs; Mutex×await; Result swallow; más bootstrap; CI Origin.

### 3. Siguiente IMPL

No ADR-047 crates hasta brief Investigador + sondeo **HOLE** (matriz como div0/overflow). Arquitecto pinnea ADR-047 tras matriz.

## Checklist

- [x] Inventario E0216/17 OK
- [x] GO DOC-only
- [x] Sin IMPL crates
