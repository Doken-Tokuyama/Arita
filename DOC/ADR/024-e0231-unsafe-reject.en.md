Translation of `024-e0231-unsafe-reject.md`; the original is normative. / Traducción de `024-e0231-unsafe-reject.md`; el original es el normativo.

# ADR-024 — E0231 unsafe/FFI surface reject

- **Estado:** **aceptada**
- **CUT-ID:** `E0231-UNSAFE-20260913`
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (GO)
- **Relacionados:** ADR-022 safe-only, ADR-023

## Decision

Surface ARITA rechaza with **E0231** (`unsafe / FFI not allowed in ARITA`):

- `unsafe { … }`, bare `unsafe`, `unsafe fn …`
- `extern … { … }`, `extern "C" fn …`, bare `extern`

Measure oracle: `neg-e0231-unsafe` → accepted iff parse fails with E0231.

Emit ya lleva `#![forbid(unsafe_code)]` (ADR-022); this CUT cierra el surface.
