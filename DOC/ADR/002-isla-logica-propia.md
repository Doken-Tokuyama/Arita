# ADR-002 — Isla lógica propia (sin bridge IDNI en MVP)

- **Estado:** aceptada
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (constraint legal) + ARITA Arquitecto
- **Relacionados:** `DOC/08-LICENSES-IDNI.md` (sin copiar/link código IDNI), `DOC/ADR/001-emit-rust-mvp.md` (backend)

## Contexto

La isla lógica de ARITA se inspira en Tau/TML (IDNI). No copiar ni linkear runtime/submodule IDNI en el producto redistribuible sin acuerdo comercial (ver `08-LICENSES-IDNI.md`). Inspiración semántica Tau/TML: OK.

## Decisión

1. MVP usa **isla lógica propia**: sintaxis `spec` / `fact` / `rule` / `query`, IR lógico ARITA, verificador reducido **sin** código ni submodule IDNI.
2. Tau/TML = **inspiración semántica** (forma de specs, modelo mental) hasta un ADR legal que habilite binding comercial o licencia compatible.
3. Fase 3 del roadmap implementa lowering + UNSAT/query fail sobre ese motor propio; no sobre TML embebido.
4. El pipeline de sistemas sigue siendo **emit-Rust** (ADR-001); la isla lógica no cambia ese backend.

## Consecuencias

- Docs, few-shots y measure no presentan APIs Tau/TML como runtime ARITA.
- Si hay acuerdo comercial IDNI, abrir ADR nuevo de binding; hasta entonces manda esta ADR + `08-LICENSES-IDNI.md`.

## Nota de migración

Sustituye el borrador plano `DOC/ADR-0001-licencias-idni-e-isla-logica.md` (retirado). Licencias → `08-LICENSES-IDNI.md`; backend → ADR-001; isla → este ADR-002.
