# ADR-089 — Bootstrap-05 dual-oracle (`fib8→21`)

- **Estado:** **aceptada** + **verified** Lex **215/215**
- **CUT-ID:** `BOOTSTRAP-05-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-037/042/057/068 bootstrap-01..04; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** dual-oracle measure + Rust twin; skip ≠ PASS; no self-host.

## Contexto

Vec.remove Lex **214/214**. Siguiente dual-oracle bootstrap.

## Decisión (pins GO)

### 1. Slice

**`fib8`**: F(8)=**21** (F0=0…F7=13, F8=21). Mismo unrolled/iter surface-legal que fib7.

### 2. Dual oracle

| Lado | Evidencia |
|------|-----------|
| Rust | `cargo test` assert `fib8() == 21` / `GOLDEN_FIB8=21` |
| ARITA | measure stdout **21** |

### 3. OUT

Self-host; Mutex; timed; recursive.

## Oráculos

| Id | Expect |
|----|--------|
| `bootstrap-05` | ARITA → **21** accepted |
| Rust twin | fib8 == 21 |

## Checklist

- [x] Pins fib8→21 + dual-oracle
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **215/215** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
