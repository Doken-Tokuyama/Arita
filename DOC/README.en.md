[Español](README.md) | English

# ARITA — Documentation

ARITA is a general-purpose language **designed to be written by AIs** that **compiles to safe Rust**. This folder gathers the vision, architecture, semantics, guides, and design decisions.

**Estado:** Core 0.9 cerrado (834/834 oráculos). Core 0.10 (ERRORES, fase 1) en curso: once etapas de la fase 1 están cerradas, todas las previstas para la versión 1 (barra actual 889/889); siguen las comprobaciones previas a la versión 1. Ver [`../ROADMAP.md`](../ROADMAP.md).

## Start here

| Document | Contents |
|-----------|-----------|
| [00-VISION.md](00-VISION.en.md) | Vision: AI-native language that compiles to real Rust |
| [PRODUCT-VISION.md](PRODUCT-VISION.en.md) | Product vision and high-level status |
| [RFC-AINATIVE-VERIFIED-MODEL.md](RFC-AINATIVE-VERIFIED-MODEL.en.md) | Thesis: language-first, verifiable, to safe Rust |
| [SEMANTICS-V0.1.md](SEMANTICS-V0.1.en.md) | Semantics and pipeline (AST → HIR → AIR → emit) |
| [09-AI-PROGRAMMING.md](09-AI-PROGRAMMING.en.md) | How an AI should program in ARITA and how to verify |
| [../BUILD.md](../BUILD.en.md) | How to build and test |
| [CI.md](CI.md) | Local continuous-integration gate |

## Foundations

| Document | Contents |
|-----------|-----------|
| [01-GOALS-NONGOALS.md](01-GOALS-NONGOALS.en.md) | Goals and non-goals |
| [02-ARCHITECTURE.md](02-ARCHITECTURE.en.md) | Architecture |
| [03-LANGUAGE-SKETCH.md](03-LANGUAGE-SKETCH.en.md) | Syntax sketch |
| [04-AI-ERGONOMICS.md](04-AI-ERGONOMICS.en.md) | Design for AIs |
| [05-COMPILATION-TARGETS.md](05-COMPILATION-TARGETS.en.md) | Cross-platform compilation |
| [06-RISKS.md](06-RISKS.en.md) | Risks |
| [07-REFERENCES.md](07-REFERENCES.en.md) | References |
| [08-LICENSES-IDNI.md](08-LICENSES-IDNI.en.md) | Note on licenses of external logic projects |

## Evidence and code safety

| Document | Contents |
|-----------|-----------|
| [THREAT_MODEL.md](THREAT_MODEL.en.md) | Threat model: fake evidence and “theater” |
| [TRAPS-CATALOG.md](TRAPS-CATALOG.md) | Trap catalog and measured defenses |
| [EVIDENCE-GAPS-V0.1.md](EVIDENCE-GAPS-V0.1.en.md) | Evidence gaps vs semantics v0.1 |
| [EVIDENCE-OOB-NOLIT-SUITE.md](EVIDENCE-OOB-NOLIT-SUITE.en.md) | Out-of-range / no-literal suite |
| [CODEGEN-EMIT-WHITELIST.md](CODEGEN-EMIT-WHITELIST.en.md) | Methods allowed in Rust emission |
| [REPAIR-ORACLE.md](REPAIR-ORACLE.en.md) | Compiler-guided repair (design proposal) |
| [PERF-HOST-BORDER.md](PERF-HOST-BORDER.en.md) | Performance: what stays outside the language |
| [LSP-EDITOR.md](LSP-EDITOR.en.md) | Language server for editors (`arita lsp`) |

## Logic island

| Document | Contents |
|-----------|-----------|
| [LOGIC-WHEN-TO-USE.md](LOGIC-WHEN-TO-USE.en.md) | When to use the logic island |
| [LOGIC-INT-ARITH-OUT.md](LOGIC-INT-ARITH-OUT.en.md) | Integer arithmetic outside the logic island |
| [LOGIC-PREDICATES-INT-SURFACE-NOTE.md](LOGIC-PREDICATES-INT-SURFACE-NOTE.md) | Logic predicates vs language surface |
| [../ejemplos/f3/](../ejemplos/f3/) | 12 logic-island oracles (`01` through `12`) |

## Few-shot example packs for AIs

| Document | Contents |
|-----------|-----------|
| [PACK-F1.1-FEWSHOT.md](PACK-F1.1-FEWSHOT.en.md) | Minimal surface |
| [PACK-F2-FEWSHOT.md](PACK-F2-FEWSHOT.en.md) | Variables, integers, strings, vectors, tests |
| [PACK-F2.2-FEWSHOT.md](PACK-F2.2-FEWSHOT.en.md) | `match` |
| [PACK-F3-FEWSHOT.md](PACK-F3-FEWSHOT.en.md) | Logic island |
| [PACK-ASYNC-FEWSHOT.md](PACK-ASYNC-FEWSHOT.en.md) | `async fn` / `await` |
| [PACK-STD-METHOD-SURFACE-F2.md](PACK-STD-METHOD-SURFACE-F2.md) | Standard-library methods |

## Guides

- [guides/aritmetica-int-v0.md](guides/aritmetica-int-v0.en.md) — integer arithmetic
- [guides/option-result-helpers-v0.md](guides/option-result-helpers-v0.en.md) — `Option` and `Result` helpers
- [guides/METHOD-CALL-PARSE.md](guides/METHOD-CALL-PARSE.md) — how method calls are parsed

## Design decisions

The [ADR/](ADR/) folder holds architecture decision records (ADR), one per decision. Some starting points:

- [ADR/001-emit-rust-mvp.md](ADR/001-emit-rust-mvp.md) — why Rust is emitted
- [ADR/002-isla-logica-propia.md](ADR/002-isla-logica-propia.md) — own logic engine
- [ADR/022-safe-only.md](ADR/022-safe-only.md) — ARITA is safe Rust only
- [ADR/225-semantics-v0.1.md](ADR/225-semantics-v0.1.md) — normative semantics map

## Examples

Programs under [`../ejemplos/`](../ejemplos/) are end-to-end oracles: they are built, run, and their output is compared. Index in [`../ejemplos/README.md`](../ejemplos/README.md). `arita measure` runs all oracles plus `clippy`; a skipped test never counts as passed.
