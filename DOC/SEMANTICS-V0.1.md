# ARITA — Semántica y cara de producto v0.1

## One-liner

**Lenguaje AI-native de propósito general** → el modelo escribe ARITA; el compilador emite **workspaces Rust safe reales**. No es plataforma de agentes, ni DSL de contratos, ni wrapper de Cargo.

## Docs canónicos

1. [`RFC-AINATIVE-VERIFIED-MODEL.md`](RFC-AINATIVE-VERIFIED-MODEL.md) — **rev. 3** (tesis language-first)
2. [`PRODUCT-VISION.md`](PRODUCT-VISION.md)
3. [`ADR/225-semantics-v0.1.md`](ADR/225-semantics-v0.1.md) — superficie R0–R8 + gaps (si existe)
4. [`EVIDENCE-GAPS-V0.1.md`](EVIDENCE-GAPS-V0.1.md)

## Pipeline

`.arita` → lexer/parser → AST → HIR → AIR/CFG → checker → emit Rust determinista → `cargo` + **scenarios** + manifiesto de evidencia JSON (`target/arita/evidence/…`). UI/dashboard **después**.

## Core 0.1 (vertical)

Módulos · tipos canónicos · Option/Result · ownership simple · I/O acotado · scenarios · `#![forbid(unsafe_code)]` · un programa de referencia no trivial. **No** self-host primero.

## Estado

Propuesta rev. 3 (2026-09-19). **HOLD** reinicio de CUT/measure hasta aviso a <person> y su GO.

## Also

- [`REPAIR-ORACLE.md`](REPAIR-ORACLE.md) / [`ADR/231-repair-oracle-v0.md`](ADR/231-repair-oracle-v0.md) — compiler-guided autonomous repair (**propuesta**; HOLD IMPL)
