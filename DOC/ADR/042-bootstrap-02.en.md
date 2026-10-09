Translation of `042-bootstrap-02.md`; the original is normative. / Traducción de `042-bootstrap-02.md`; el original es el normativo.

# ADR-042 — Bootstrap-02 dual-oracle (`sum5`)

- **Estado:** **aceptada** + **verified** (Lex measure **83** accepted + **2** gated; `sum5→15`)
- **CUT-ID:** `BOOTSTRAP-02-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-037 (bootstrap-01 `fact5→120`), Fase 5
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Sin self-host. Park más clones `if`/`while` false.
- **Barra:** dual-oracle; acceptance = `arita measure`; Rust `cargo test` auxiliar; skip ≠ PASS.

## Context

Control-flow theater (E0225/26/27 + E0214/15) **suficiente** por ahora. Pipeline: segunda fn pura dogfood (no self-host).

## Decision (pins GO)

### 1. Slice

Dual-oracle **`sum5`**: unrolled `1+2+3+4+5` → **`15`** (`GOLDEN_SUM5 = 15`).

Distinct from `fact5→120` (ADR-037). Same shape: ARITA + Rust same algorithm.

### 2. Invocation

- `ejemplos/bootstrap/02-sum.arita` (nombre exacto OK si Ingeniero ajusta)
- Rust: extend `bootstrap.rs` (o sibling module) with `sum5()` + test
- **Solo measure** + `cargo test`; no new CLI

### 3. OUT

- More `if false` / `while false` morphs (park control-flow clones)
- Self-host / pest rewrite
- Mutex×await / Result / CI Origin (siguen park/bloqueados)
- Catalog E0xxx

## Checklist

- [x] sum5→15 + layout OK
- [x] GO DOC+IMPL
- [x] Landed + measure (Ingeniero)

## IMPL evidence (`BOOTSTRAP-02-20260915`)

| Id | Path | Expect | Verdict |
|----|------|--------|---------|
| `bootstrap-02` | `ejemplos/bootstrap/02-sum.arita` | E2E stdout `15` (`sum5` unrolled 1‥5) | accepted (measure) |
| Rust `cargo test -p arita-cli bootstrap` | `crates/arita-cli/src/bootstrap.rs` | `sum5() == 15` (`GOLDEN_SUM5`) | pass (aux) |

**Algorithm:** sum-lite `sum5` = `1+2+3+4+5` (unrolled; Int helpers = Expr/Let only).  
**Golden:** `15`. No CLI `arita bootstrap-check`. Parser/Codegen HOLD.  
**Measure:** Lex **83 accepted** + **2 gated** (from **82**+2).

HOLD crates → review-only post verify.
