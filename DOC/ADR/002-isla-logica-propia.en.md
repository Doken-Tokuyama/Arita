Translation of `002-isla-logica-propia.md`; the original is normative. / Traducción de `002-isla-logica-propia.md`; el original es el normativo.

# ADR-002 — Own logic island (no IDNI bridge in MVP)

- **Estado:** aceptada
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (constraint legal) + ARITA Arquitecto
- **Relacionados:** `DOC/08-LICENSES-IDNI.md` (sin copiar/link código IDNI), `DOC/ADR/001-emit-rust-mvp.md` (backend)

## Context

ARITA’s logic island is inspired by Tau/TML (IDNI). Do not copy or link IDNI runtime/submodule into the redistributable product without a commercial agreement (see `08-LICENSES-IDNI.md`). Tau/TML semantic inspiration: OK.

## Decision

1. MVP uses an **own logic island**: `spec` / `fact` / `rule` / `query` syntax, ARITA logic IR, reduced verifier **without** IDNI code or submodule.
2. Tau/TML = **semantic inspiration** (spec shape, mental model) until a legal ADR enables a commercial binding or compatible license.
3. Roadmap Phase 3 implements lowering + UNSAT/query fail on that own engine; not on embedded TML.
4. The systems pipeline remains **emit-Rust** (ADR-001); the logic island does not change that backend.

## Consequences

- Docs, few-shots, and measure do not present Tau/TML APIs as ARITA runtime.
- If there is a commercial IDNI agreement, open a new binding ADR; until then this ADR + `08-LICENSES-IDNI.md` govern.

## Migration note

Replaces the flat draft `DOC/ADR-0001-licencias-idni-e-isla-logica.md` (retired). Licenses → `08-LICENSES-IDNI.md`; backend → ADR-001; island → this ADR-002.
