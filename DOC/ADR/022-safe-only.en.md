Translation of `022-safe-only.md`; the original is normative. / Traducción de `022-safe-only.md`; el original es el normativo.

# ADR-022 — ARITA is safe-only

- **Estado:** **aceptada** (hard rule <person> 2026-09-13)
- **CUT-ID:** `SAFE-ONLY-20260913`
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (GO) · Arquitecto (align)
- **Relacionados:** ADR-005/006/009, GOALS-NONGOALS, THREAT_MODEL, ADR-021

## Decision

Every ARITA program is **safe**. There is no escape hatch in the dialect we measure.

1. **Surface:** no `unsafe` keyword, no raw pointers, no FFI, no transmute, no explicit lifetimes as a door to unsafety.
2. **Emit:** Rust generated for user programs carries `#![forbid(unsafe_code)]` (or per-crate equivalent); the backend does not emit `unsafe` for user code.
3. **Checker:** ownership/borrow + types reject unsound patterns; measure + Miri (ADR-021) are judges, not theater.
4. **Explicit OUT:** any CUT that introduces `unsafe` in user surface or emit → **reject** / no merge to SoT.

## Note on older docs

`01-GOALS-NONGOALS.md` spoke of “explicit unsafe (later phase)”. That phase is **out of product** under this rule until a new explicit <person> decision. Safe-by-default becomes **safe-only**.

## Consequences

- Codegen / Parser / HIR: reject and diagnose unsafety attempts.
- Measure: negative oracles if `unsafe` appears in emit (add when a stable hook exists).
- Logic island: no bypass via hidden effects; explicit `Io` remains the effects channel.

## Architect alignment (with ownership / emit ADRs)

| Source | What it said | Under ADR-022 |
|--------|-----------|--------------|
| ADR-006 § ownership | No `unsafe`, no raw pointers (F2 OUT) | **Elevated to product hard rule** (all measured phases) |
| ADR-009 | No `unsafe` / raw / explicit lifetimes in F2 | Same; checker rejects unsound; not an escape hatch |
| ADR-001 emit-Rust §4 | `forbid(unsafe_code)` *except* dialect `unsafe` modules (late phase) | **Superseded:** no user `unsafe` modules; forbid always on program emit |
| ADR-021 Miri | UB oracle on lib crates | Complementary judge; does not replace forbid on user emit |
| THREAT_MODEL | Bypass via `unsafe`/FFI | Mitigation = this ADR + measure |

**Draft diagnostic (if CUT IMPL is needed):** `E0231` — `unsafe / FFI not allowed in ARITA` (user surface or emit). **Do not use E0230** (already CONTRACT-TARGET-FN / ADR-020). Exact codes = separate CUT; do not invent PASS.

**Governance:** any surface/emit CUT that introduces `unsafe` → reject (Engineer). Changing this rule = explicit <person> decision + new ADR.
