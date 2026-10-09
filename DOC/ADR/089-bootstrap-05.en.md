Translation of `089-bootstrap-05.md`; the original is normative. / Traducción de `089-bootstrap-05.md`; el original es el normativo.

# ADR-089 — Bootstrap-05 dual-oracle (`fib8→21`)

- **Estado:** **aceptada** + **verified** Lex **215/215**
- **CUT-ID:** `BOOTSTRAP-05-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-037/042/057/068 bootstrap-01..04; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** dual-oracle measure + Rust twin; skip ≠ PASS; no self-host.

## Context

Vec.remove Lex **214/214**. Next dual-oracle bootstrap.

## Decision (GO pins)

### 1. Slice

**`fib8`**: F(8)=**21** (F0=0…F7=13, F8=21). Same surface-legal unrolled/iter as fib7.

### 2. Dual oracle

| Side | Evidence |
|------|-----------|
| Rust | `cargo test` assert `fib8() == 21` / `GOLDEN_FIB8=21` |
| ARITA | measure stdout **21** |

### 3. OUT

Self-host; Mutex; timed; recursive.

## Oracles

| Id | Expect |
|----|--------|
| `bootstrap-05` | ARITA → **21** accepted |
| Rust twin | fib8 == 21 |

## Checklist

- [x] Pins fib8→21 + dual-oracle
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **215/215** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
