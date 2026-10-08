# ADR-021 — Miri workspace oracle

- **Estado:** **aceptada**
- **CUT-ID:** `MIRI-ORACLE-20260913`
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (GO)
- **Relacionados:** measure clippy-workspace, THREAT_MODEL, ADR-017/018

## Decisión

`arita measure` runs a required toolchain oracle:

```text
cargo +nightly miri test -p arita-syntax -p arita-codegen -p arita-hir -p arita-logic --lib
```

- **accepted** — Miri finishes all lib tests without UB / failure
- **inconclusive** — nightly or miri component missing (never treated as accepted; skip ≠ PASS)
- **rejected** — Miri reports failure / undefined behavior

## Scope

Lib crates only. `arita-cli` integration tests spawn host binaries and are out of Miri scope for v0.

## Barra

Real Miri execution on Lex (or any host with nightly+miri). Box without rustup → inconclusive is correct.
