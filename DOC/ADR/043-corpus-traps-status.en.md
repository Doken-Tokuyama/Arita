Translation of `043-corpus-traps-status.md`; the original is normative. / Traducción de `043-corpus-traps-status.md`; el original es el normativo.

# ADR-043 — Traps corpus portable: status DOC (tick)

- **Estado:** **aceptada** (DOC-only; sin IMPL crates)
- **CUT-ID:** `CORPUS-STATUS-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto
- **Relacionados:** `TRAPS-CATALOG.md`, ROADMAP Fase 4 corpus, ADR-010…042
- **Gobernanza:** DOC-only. No inventar PASS. Ports Investigación siguen abiertos.
- **Barra:** skip ≠ PASS; checkbox ROADMAP no finge ports hechos.

## Decision

### 1. Portable wave **verified** (ARITA surface)

| Code | Tema | ADR / CUT |
|--------|------|-----------|
| E0214 | vacuous len/is_empty | ADR-030 |
| E0215 | vacuous comparison assert | ADR-038 |
| E0225 | vacuous match arms | ADR-031 |
| E0226 | vacuous while false | ADR-040 |
| E0227 | vacuous if false | ADR-041 |
| E0242 | borrow across await | ADR-039 |

(+ baseline E0210–13, E020x, E0220–24, E0231, E0240/41, E0250, E0260/61 already in catalog.)

Baseline measure post bootstrap-02: **83** accepted + **2** gated.

### 2. PARK (no CUT without brief)

| Item | Motivo |
|------|--------|
| More morphs if/while/else/break | Control-flow theater saturated |
| Mutex×await / Result swallow | No surface |
| More bootstrap-0N | PARK unless explicit GO |
| CI Origin remote | Bloqueado Origin |

### 3. ROADMAP tick

- Add **partial** checkbox for the portable verified wave (this ADR).
- Keep **`[ ]` Traps corpus (Investigation port)** until real ports with oracles.
- Veyra residual: nombre producto = **ARITA**; no CUT Veyra.

### 4. Next IMPL

Standby: trap **no-control** only with brief Investigator+Ingeniero; or unlocked ROADMAP CUT. No ADR-044 crates until then.

## Checklist

- [x] Inventario + PARK OK
- [x] GO DOC-only
- [x] No crate IMPL

## Addendum

**ADR-046** (`CORPUS-ARITH-20260915`): adds E0216/E0217 verified; baseline **85**+2.
