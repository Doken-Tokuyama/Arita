Translation of `099-f3-unsat-fail-next.md`; the original is normative. / Traducción de `099-f3-unsat-fail-next.md`; el original es el normativo.

# ADR-099 — F3 oracles `f3-11` / `f3-12` (anti partial-PASS)

- **Estado:** **aceptada** + **verified** Logic `f3_` **12/12** (2026-09-18)
- **Evidence:** `F3_UNSAT_FAIL_NEXT_EVIDENCE.md` (not in the public export / no incluido en el export público)
- **CUT-ID:** `F3-UNSAT-FAIL-NEXT-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Logic (propuesta) · ARITA Arquitecto (pins) · Logic sole IMPL oracles
- **Relacionados:** ADR-012; PACK-F3; E0301 `query failed`
- **Gobernanza:** **No pisa** Imperativo / ADR-098 wrapping_mul / **Mutex PARK**. Parser/Codegen HOLD crates Imperativo.
- **Barra:** E2E `arita logic` + measure; skip ≠ PASS; inconclusive ≠ accepted.

## Context

F3 island GREEN (f3-01…10). Anti-theater gap: multi-query and join-miss must **fail E0301**, never partial-PASS.

## Decision (GO pins)

### 1. IN v0 — both oracles

| Id | Semantics | Expect |
|----|-----------|--------|
| `f3-11-multi-query-fail` | ≥1 query sat + ≥1 query fail in the same run | **rejected E0301** (no partial PASS) |
| `f3-12-join-miss-fail` | join with different dept/relation → no closure | **rejected E0301** |

Pins:
- Reuse **E0301** `query failed` (ADR-012). Do not invent a new E03xx in this CUT.
- Portable surface = same current F3 grammar (facts/rules/queries).
- Measure: exit ≠ 0 + stderr contains `E0301`.
- Soft IN: unit tests `f3_05..08` if missing (no theater).

### 2. OUT v0

- Mutex / Imperativo / wrapping_div; change E0301 semantics; IDNI; inconclusive→accepted

### 3. Parallel DOC

- Draft `DOC/LOGIC-WHEN-TO-USE.md` (human+AI): when logic island vs Imperativo; never invent PASS; skip≠PASS.
- Logic proposal detail: `DOC/PROPOSAL-F3-UNSAT-FAIL-NEXT.md` (if not yet in Lex SoT, Logic lands it).

## Checklist

- [x] GO pins 11+12
- [x] Oracles + green measure
- [x] PACK-F3 / TRAPS-CATALOG tick (Docs)
- [x] Soft: unit tests f3_05..08

## Queue

Oracles `f3-11`/`f3-12` green; Logic bar **12/12**. Imperativo: bar **244/244** (ADR-098); ADR-101 IMPL GO (separate).
