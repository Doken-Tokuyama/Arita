# ADR-241 — Evidence R0/R2 emit-ban

- **Estado:** **aceptada** + **IMPL** (CUT `EVIDENCE-R0R2-EMIT-BAN-20260919`)
- **CUT-ID:** `EVIDENCE-R0R2-EMIT-BAN-20260919`
- **Fecha:** 2026-09-20
- **Autores:** Ingeniero (IMPL) · pins ADR-230/225
- **Gobernanza:** skip ≠ PASS; oracle unit + measure wiring

## Objetivo

Barrido emit: **prohibido** inyectar `.unwrap()` / `.expect(` / `panic!` en código usuario. Overflow no-lit → `unwrap_or` saturating (pow / next_multiple helpers).

## Fix aplicado

| Antes | Después |
|-------|---------|
| `checked_pow(...).unwrap()` | `...unwrap_or(i64::MAX)` |
| `checked_*.expect("… overflow")` | `unwrap_or(i64::MAX/MIN)` |

## Oráculos

- `cargo test -p arita-codegen evidence_r0r2_emit_ban`
- Measure: `emit-ban-r0r2` (cargo test gate)

## Suite OOB no-lit (documentada, ya verde)

`get` OOB → None; swap OOB no-op; neg E0287 unwrap_or-as-success; E0310 index ban (R4).
