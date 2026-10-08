[Español](SEMANTICS-V0.1.md) | English

# ARITA — Semantics and product face v0.1

## One-liner

**General-purpose AI-native language** → the model writes ARITA; the compiler emits **real safe Rust workspaces**. Not an agent platform, nor a contract DSL, nor a Cargo wrapper.

## Canonical docs

1. [`RFC-AINATIVE-VERIFIED-MODEL.md`](RFC-AINATIVE-VERIFIED-MODEL.en.md) — **rev. 3** (language-first thesis)
2. [`PRODUCT-VISION.md`](PRODUCT-VISION.en.md)
3. [`ADR/225-semantics-v0.1.md`](ADR/225-semantics-v0.1.md) — R0–R8 surface + gaps (if it exists)
4. [`EVIDENCE-GAPS-V0.1.md`](EVIDENCE-GAPS-V0.1.en.md)

## Pipeline

`.arita` → lexer/parser → AST → HIR → AIR/CFG → checker → deterministic Rust emit → `cargo` + **scenarios** + JSON evidence manifest (`target/arita/evidence/…`). UI/dashboard **afterwards**.

## Core 0.1 (vertical)

Modules · canonical types · Option/Result · simple ownership · bounded I/O · scenarios · `#![forbid(unsafe_code)]` · one non-trivial reference program. **Not** self-host first.

## Estado

Propuesta rev. 3 (2026-09-19). **HOLD** reinicio de CUT/measure hasta aviso a <person> y su GO.

## Also

- [`REPAIR-ORACLE.md`](REPAIR-ORACLE.en.md) / [`ADR/231-repair-oracle-v0.md`](ADR/231-repair-oracle-v0.md) — compiler-guided autonomous repair (**propuesta**; HOLD IMPL)
