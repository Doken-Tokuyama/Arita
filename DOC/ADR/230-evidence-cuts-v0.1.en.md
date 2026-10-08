Translation of `230-evidence-cuts-v0.1.md`; the original is normative. / Traducción de `230-evidence-cuts-v0.1.md`; el original es el normativo.

# ADR-230 — Evidence CUT pins v0.1 (post-ACK RFC)

- **Estado:** **aceptada** · **GO reinicio** <person> 2026-09-19 (cierra R4 → Core 0.1)
- **R4:** **CLOSED** Lex **583/583** (2026-09-19) — E0310 vec+string neg + emit-ban; STABLE + `_mirrors/_mirror_r4_e0310.tgz`
- **CUT-ID:** `EVIDENCE-CUTS-V0.1-20260919`
- **Fecha:** 2026-09-19
- **Relacionados:** RFC rev. 2 ACK; ADR-225 §12; ADR-227/228/229; `EVIDENCE-GAPS-V0.1.md`
- **Gobernanza:** Estos CUT son **evidencia / neg oracles / CI**, no features std aisladas. skip ≠ PASS. HOLD std feature hasta cada GO explícito.

## Mandatory order

| # | Gap | CUT-ID | Contract ADR | Deliverable | Lex |
|---|-----|--------|--------------|-------------|-----|
| **1** | **R4** | `EVIDENCE-R4-INDEX-NEG-20260919` | 227 | E0310 + `neg-e0310-index-vec` + `neg-e0310-index-string`; emit grep no-panic-index | +2 neg |
| **2** | **R0/R2** | `EVIDENCE-R0R2-EMIT-BAN-20260919` | 225 R0/R2 | Emit sweep: ban non-allowlisted `unwrap`/`expect`/`panic!`; oracle that **fails** if they appear; documented OOB no-lit suite | CI + measure |
| **3** | **R7** | `EVIDENCE-R7-MUTEX-NEG-20260919` | 229 | E0312 + `neg-e0312-mutex` | +1 neg |
| **4** | **R5** | `EVIDENCE-R5-INSERT-NEG-20260919` | 228 | Dedicated `neg-e0206-insert` (and drain/closure if applicable) until insert unpark | +1..n neg |
| **5** | **R8** | `EVIDENCE-R8-REMOTE-CI-20260919` | 225 R8 / CI.md | Origin/remote CI certifies measure + forbid(unsafe); reproducible artifact | CI green |

## Current GO

**R4 CLOSED.** **R0/R2 CLOSED** (ADR-241). **GO #2 → Engineer:** Core 0.1 — [ADR-232](232-core-0.1-pins.md) (accepted below).


## HOLD

- IMPL `insert` / `[]` sugar / `Mutex` type — until Core 1/2 GO after evidence in order
- Isolated std CUTs (bootstrap/Int/Vec extras) — HOLD unless <person> reprioritizes

## Checklist

- [x] Order R4→R0/R2→R7→R5→R8 pinned
- [x] Codes E0310 / E0311 / E0312 reserved
- [x] R4 green Lex **583/583**
- [x] R0/R2 emit-ban (ADR-241) — Lex gate `emit-ban-r0r2`
- [ ] R7 / R5 / R8 … chain
