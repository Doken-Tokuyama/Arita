[Español](LOGIC-INT-ARITH-OUT.md) | English

# Logic island ↔ Int arithmetic — PARK / OUT (v0)

**Status:** PARK / OUT for F3 v0  
**Date:** 2026-09-18  
**Audience:** Arquitecto / Logic / Imperativo (oleada wrapping/saturating)

## Pin (1 paragraph)

The `fact` / `rule` / `query` of the F3 logic island are **relational** (symbols + positive Datalog). **OUT of v0:** lower `Int` wrapping/saturating/`i64` wrap-release arithmetic to TML/Tau or the `arita-logic` engine. The Imperativo wave (E0217, saturating_add/sub, wrapping APIs) does **not** block or couple the island: they live in different crates/surface; Mutex PARK does not apply to Logic. If integers in logic are wanted someday, a new ADR is required (checked vs wrap vs saturating semantics) — **do not** reuse Rust wrap-release as a logical verdict.

## Checklist review

- [x] `crates/arita-logic`: no `i64` / wrapping / saturating / overflow paths
- [x] ADR-012 / PACK-F3: no Int-overflow claim in Logic (nothing stale to uncheck except this explicit pin)
- [x] ADR-045 / ADR-090 / ADR-091: Imperative only — do not reopen from Logic
