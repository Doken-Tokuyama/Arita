[Español](04-AI-ERGONOMICS.md) | English

# Ergonomics for AIs

## Problem

AIs “speak” Rust, but they generate theater (empty asserts, stubs, morph). See corpus <org> / rust-traps.

## ARITA strategy

1. **Small grammar** + JSON Schema / Tree-sitter for constrained decoding.
2. **Canonical F1.1 surface** (ADR-005 CUT `F1.1-SURFACE-20260913`): only `module` + `fn main() -> Io<()>` + `print("…")`.
3. **Official few-shot pack** (PACK L style <org>): `AGENTS.md`, patterns, anti-theater → deliverable **Fase 2** (F1.1 few-shots only teach ADR-005 §1).
4. **`arita measure` v0** (verified) + **`ejemplos/`** = production E2E oracles (real compile+run + clippy-workspace; no smoke/fake/theater; skip ≠ PASS; overall `accepted` needs clippy on host):
   - **F1:** parse + `arita build` + binary / rustc execution (`E0100`/`E0101`/`E0102`).
   - **F2:** anti-theater oracle (`E02xx+`).
   - **F3:** UNSAT / query fail → FAIL (`E03xx+`).
5. **ARITA trap catalog** → **Fase 4**.
6. **Stable output:** machine-readable `E0xxx` codes; **canonical English text** after the code (table in ADR-005). DOC prose in Spanish is OK.

## “Innate”

It does not mean the model is born knowing ARITA. It means:

- train / prompt / grammar is cheaper than full Rust,
- fewer degrees of freedom to lie in tests,
- logic specs where the liar is caught by UNSAT / failed query (when the F3 engine exists).
