# ADR-099 — F3 oracles `f3-11` / `f3-12` (anti partial-PASS)

- **Estado:** **aceptada** + **verified** Logic `f3_` **12/12** (2026-09-18)
- **Evidencia:** [`F3_UNSAT_FAIL_NEXT_EVIDENCE.md`](../../F3_UNSAT_FAIL_NEXT_EVIDENCE.md)
- **CUT-ID:** `F3-UNSAT-FAIL-NEXT-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Logic (propuesta) · ARITA Arquitecto (pins) · Logic sole IMPL oracles
- **Relacionados:** ADR-012; PACK-F3; E0301 `query failed`
- **Gobernanza:** **No pisa** Imperativo / ADR-098 wrapping_mul / **Mutex PARK**. Parser/Codegen HOLD crates Imperativo.
- **Barra:** E2E `arita logic` + measure; skip ≠ PASS; inconclusive ≠ accepted.

## Contexto

Isla F3 GREEN (f3-01…10). Gap anti-theater: multi-query y join-miss deben **fail E0301**, nunca partial-PASS.

## Decisión (pins GO)

### 1. IN v0 — ambos oráculos

| Id | Semántica | Expect |
|----|-----------|--------|
| `f3-11-multi-query-fail` | ≥1 query sat + ≥1 query fail en mismo run | **rejected E0301** (no PASS parcial) |
| `f3-12-join-miss-fail` | join con dept/relación distinta → sin cierre | **rejected E0301** |

Pins:
- Reusar **E0301** `query failed` (ADR-012). No inventar E03xx nuevo en este CUT.
- Surface portable = misma gramática F3 actual (facts/rules/queries).
- Measure: exit ≠ 0 + stderr contiene `E0301`.
- Soft IN: unit tests `f3_05..08` si faltan (sin theater).

### 2. OUT v0

- Mutex / Imperativo / wrapping_div; cambiar semántica E0301; IDNI; inconclusive→accepted

### 3. DOC paralelo

- Draft `DOC/LOGIC-WHEN-TO-USE.md` (humano+IA): cuándo isla lógica vs Imperativo; never invent PASS; skip≠PASS.
- Detalle propuesta Logic: `DOC/PROPOSAL-F3-UNSAT-FAIL-NEXT.md` (si aún no en SoT Lex, Logic aterriza).

## Checklist

- [x] Pins GO 11+12
- [x] Oráculos + measure verdes
- [x] PACK-F3 / TRAPS-CATALOG tick (Docs)
- [x] Soft: unit tests f3_05..08

## Cola

Oráculos `f3-11`/`f3-12` verdes; barra Logic **12/12**. Imperativo: barra **244/244** (ADR-098); ADR-101 IMPL GO (aparte).
