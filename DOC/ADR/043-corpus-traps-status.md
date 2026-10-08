# ADR-043 — Corpus trampas portable: status DOC (tick)

- **Estado:** **aceptada** (DOC-only; sin IMPL crates)
- **CUT-ID:** `CORPUS-STATUS-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto
- **Relacionados:** `TRAPS-CATALOG.md`, ROADMAP Fase 4 corpus, ADR-010…042
- **Gobernanza:** DOC-only. No inventar PASS. Ports Investigación siguen abiertos.
- **Barra:** skip ≠ PASS; checkbox ROADMAP no finge ports hechos.

## Decisión

### 1. Oleada portable **verified** (ARITA surface)

| Código | Tema | ADR / CUT |
|--------|------|-----------|
| E0214 | vacuous len/is_empty | ADR-030 |
| E0215 | vacuous comparison assert | ADR-038 |
| E0225 | vacuous match arms | ADR-031 |
| E0226 | vacuous while false | ADR-040 |
| E0227 | vacuous if false | ADR-041 |
| E0242 | borrow across await | ADR-039 |

(+ baseline E0210–13, E020x, E0220–24, E0231, E0240/41, E0250, E0260/61 ya en catálogo.)

Baseline measure post bootstrap-02: **83** accepted + **2** gated.

### 2. PARK (no CUT sin brief)

| Item | Motivo |
|------|--------|
| Más morphs if/while/else/break | Control-flow theater saturado |
| Mutex×await / Result swallow | Sin surface |
| Más bootstrap-0N | PARK salvo GO explícito |
| CI Origin remote | Bloqueado Origin |

### 3. ROADMAP tick

- Añadir checkbox **parcial** oleada portable verified (este ADR).
- Mantener **`[ ]` Corpus trampas (port Investigación)** hasta ports reales con oráculos.
- Veyra residual: nombre producto = **ARITA**; no CUT Veyra.

### 4. Siguiente IMPL

Standby: trampa **no-control** solo con brief Investigador+Ingeniero; o CUT ROADMAP desbloqueado. Sin ADR-044 crates hasta entonces.

## Checklist

- [x] Inventario + PARK OK
- [x] GO DOC-only
- [x] Sin IMPL crates

## Addendum

**ADR-046** (`CORPUS-ARITH-20260915`): añade E0216/E0217 verified; baseline **85**+2.
