# Logic island ↔ Int arithmetic — PARK / OUT (v0)

**Status:** PARK / OUT for F3 v0  
**Date:** 2026-09-18  
**Audience:** Arquitecto / Logic / Imperativo (oleada wrapping/saturating)

## Pin (1 párrafo)

Los `fact` / `rule` / `query` de la isla lógica F3 son **relacionales** (símbolos + Datalog positivo). **OUT de v0:** bajar aritmética `Int` wrapping/saturating/`i64` wrap release a TML/Tau o al motor `arita-logic`. La oleada Imperativo (E0217, saturating_add/sub, wrapping APIs) **no bloquea** ni acopla la isla: viven en crates/surface distintos; Mutex PARK no aplica a Logic. Si un día se quieren enteros en lógica, hace falta ADR nuevo (semántica checked vs wrap vs saturating) — **no** reutilizar el wrap release de Rust como veredicto lógico.

## Checklist review

- [x] `crates/arita-logic`: sin `i64` / wrapping / saturating / overflow paths
- [x] ADR-012 / PACK-F3: sin claim de Int overflow en Logic (nada stale que desmarcar salvo este pin explícito)
- [x] ADR-045 / ADR-090 / ADR-091: Imperativo only — no reabrir desde Logic
