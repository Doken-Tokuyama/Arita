# ADR-230 — Pins CUT evidencia v0.1 (post-ACK RFC)

- **Estado:** **aceptada** · **GO reinicio** <person> 2026-09-19 (cierra R4 → Core 0.1)
- **R4:** **CLOSED** Lex **583/583** (2026-09-19) — E0310 vec+string neg + emit-ban; STABLE + `_mirrors/_mirror_r4_e0310.tgz`
- **CUT-ID:** `EVIDENCE-CUTS-V0.1-20260919`
- **Fecha:** 2026-09-19
- **Relacionados:** RFC rev. 2 ACK; ADR-225 §12; ADR-227/228/229; `EVIDENCE-GAPS-V0.1.md`
- **Gobernanza:** Estos CUT son **evidencia / neg oracles / CI**, no features std aisladas. skip ≠ PASS. HOLD std feature hasta cada GO explícito.

## Orden obligatorio

| # | Gap | CUT-ID | ADR contrato | Deliverable | Lex |
|---|-----|--------|--------------|-------------|-----|
| **1** | **R4** | `EVIDENCE-R4-INDEX-NEG-20260919` | 227 | E0310 + `neg-e0310-index-vec` + `neg-e0310-index-string`; emit grep no-panic-index | +2 neg |
| **2** | **R0/R2** | `EVIDENCE-R0R2-EMIT-BAN-20260919` | 225 R0/R2 | Barrido emit: ban `unwrap`/`expect`/`panic!` no allowlist; oracle que **falle** si aparece; suite OOB no-lit documentada | CI + measure |
| **3** | **R7** | `EVIDENCE-R7-MUTEX-NEG-20260919` | 229 | E0312 + `neg-e0312-mutex` | +1 neg |
| **4** | **R5** | `EVIDENCE-R5-INSERT-NEG-20260919` | 228 | `neg-e0206-insert` (y drain/closure si aplica) dedicado hasta unpark insert | +1..n neg |
| **5** | **R8** | `EVIDENCE-R8-REMOTE-CI-20260919` | 225 R8 / CI.md | Origin/remote CI certifica measure + forbid(unsafe); artefacto reproducible | CI green |

## GO actual

**R4 CLOSED.** **R0/R2 CLOSED** (ADR-241). **GO #2 → Ingeniero:** Core 0.1 — [ADR-232](232-core-0.1-pins.md) (aceptada abajo).


## HOLD

- IMPL `insert` / sugar `[]` / `Mutex` tipo — hasta GO Núcleo 1/2 tras evidencia en orden
- Std CUT aislados (bootstrap/Int/Vec extras) — HOLD salvo que <person> repriorice

## Checklist

- [x] Orden R4→R0/R2→R7→R5→R8 pinado
- [x] Códigos E0310 / E0311 / E0312 reservados
- [x] R4 verde Lex **583/583**
- [x] R0/R2 emit-ban (ADR-241) — Lex gate `emit-ban-r0r2`
- [ ] R7 / R5 / R8 … cadena
