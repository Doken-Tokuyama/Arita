Translation of `057-bootstrap-03.md`; the original is normative. / Traducción de `057-bootstrap-03.md`; el original es el normativo.

# ADR-057 — Bootstrap-03 dual-oracle (`fib6→8`)

- **Estado:** **aceptada** + **verified** Lex **119/119**
- **CUT-ID:** `BOOTSTRAP-03-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-037 bootstrap-01 (`fact5→120`); ADR-042 bootstrap-02 (`sum5→15`); ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK** (no inventar). Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** dual-oracle measure + `cargo test` Rust mismo algoritmo; skip ≠ PASS; no self-host.

## Context

while-let trap closed Lex **118/118**. Mutex/Origin PARK. Next unlocked CUT: **otro dual-oracle bootstrap** (same shape as 037/042), no surface nueva ni Mutex.

## Decision (GO pins)

### 1. Slice

Pure function **`fib6`**: iterative (or surface-legal recursive) Fibonacci such that **`fib(6) = 8`**
(definition: F(0)=0, F(1)=1, F(2)=1, F(3)=2, F(4)=3, F(5)=5, **F(6)=8**).

### 2. Dual oracle

| Side | Evidence |
|------|-----------|
| Rust | `cargo test` (o unit en crate bootstrap) assert `fib6() == 8` |
| ARITA | `ejemplos/bootstrap/` + measure stdout/`assert` **8** |

Golden SoT = constante **8** documentada. Acceptance suite = `arita measure`.

### 3. Invocation

**measure** only (+ Rust test). No CLI `arita bootstrap-check` (same as 037/042).

### 4. OUT

- Self-host / full ARITA frontend
- Mutex / Origin
- Timed perf thresholds
- Control-flow morph clones por this CUT

## Oracles

| Id | Expect |
|----|--------|
| `bootstrap-03-fib6` | ARITA → print/`assert` **8** accepted |
| Rust twin | `fib6 == 8` en test workspace |

## Checklist

- [x] Pins fib6→8 + dual-oracle + CUT
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **119/119** (STABLE_VERIFY); Rust twin fib6

## Queue

Mutex **PARK**. while-let Result / insert = posteriores si aplica.
