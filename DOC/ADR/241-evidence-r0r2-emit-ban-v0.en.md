Translation of `241-evidence-r0r2-emit-ban-v0.md`; the original is normative. / Traducción de `241-evidence-r0r2-emit-ban-v0.md`; el original es el normativo.

# ADR-241 — Evidence R0/R2 emit-ban

- **Estado:** **aceptada** + **IMPL** (CUT `EVIDENCE-R0R2-EMIT-BAN-20260919`)
- **CUT-ID:** `EVIDENCE-R0R2-EMIT-BAN-20260919`
- **Fecha:** 2026-09-20
- **Autores:** Ingeniero (IMPL) · pins ADR-230/225
- **Gobernanza:** skip ≠ PASS; oracle unit + measure wiring

## Objective

Emit sweep: **forbid** injecting `.unwrap()` / `.expect(` / `panic!` into user code. Non-lit overflow → saturating `unwrap_or` (pow / next_multiple helpers).

## Fix applied

| Before | After |
|--------|-------|
| `checked_pow(...).unwrap()` | `...unwrap_or(i64::MAX)` |
| `checked_*.expect("… overflow")` | `unwrap_or(i64::MAX/MIN)` |

## Oracles

- `cargo test -p arita-codegen evidence_r0r2_emit_ban`
- Measure: `emit-ban-r0r2` (cargo test gate)

## Non-lit OOB suite (documented, already green)

`get` OOB → None; swap OOB no-op; neg E0287 unwrap_or-as-success; E0310 index ban (R4).
