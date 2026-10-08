# Ergonomía para IAs

## Problema

Las IAs “hablan” Rust, pero generan theater (asserts vacíos, stubs, morph). Ver corpus <org> / rust-traps.

## Estrategia ARITA

1. **Gramática pequeña** + JSON Schema / Tree-sitter para constrained decoding.
2. **Surface F1.1** canónico (ADR-005 CUT `F1.1-SURFACE-20260913`): solo `module` + `fn main() -> Io<()>` + `print("…")`.
3. **Few-shot pack** oficial (PACK L estilo <org>): `AGENTS.md`, patrones, anti-theater → entregable **Fase 2** (F1.1 few-shots solo enseñan ADR-005 §1).
4. **`arita measure` v0** (verificado) + **`ejemplos/`** = oráculos E2E de producción (compile+run real + clippy-workspace; sin smoke/fake/theater; skip ≠ PASS; overall `accepted` needs clippy on host):
   - **F1:** parse + `arita build` + ejecución del binario / rustc (`E0100`/`E0101`/`E0102`).
   - **F2:** oráculo anti-theater (`E02xx+`).
   - **F3:** UNSAT / query fail → FAIL (`E03xx+`).
5. **Catálogo de trampas ARITA** → **Fase 4**.
6. **Salida estable:** códigos `E0xxx` machine-readable; **texto canónico en inglés** tras el código (tabla en ADR-005). Prosa DOC en español OK.

## “Innato”

No significa que el modelo nazca sabiendo ARITA. Significa:

- entrena / prompt / grammar más barato que Rust completo,
- menos grados de libertad para mentir en tests,
- specs lógicas donde el mentiroso se detecta por UNSAT / query fallida (cuando exista el motor en F3).
