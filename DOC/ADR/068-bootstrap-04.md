# ADR-068 — Bootstrap-04 dual-oracle (`fib7→13`)

- **Estado:** **aceptada** + **verified** Lex **150/150**
- **CUT-ID:** `BOOTSTRAP-04-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-037/042/057 bootstrap-01..03; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** dual-oracle measure + Rust twin; skip ≠ PASS; no self-host.

## Contexto

to_string Lex **149/149**. Tras racha std, otro dual-oracle bootstrap (forma 057).

## Decisión (pins GO)

### 1. Slice

**`fib7`**: F(7)=**13** (F0=0…F6=8, F7=13). Mismo estilo unrolled/iter surface-legal que fib6.

### 2. Dual oracle

| Lado | Evidencia |
|------|-----------|
| Rust | `cargo test` assert `fib7() == 13` / `GOLDEN_FIB7=13` |
| ARITA | measure stdout/assert **13** |

Acceptance = `arita measure`. Sin CLI bootstrap-check.

### 3. OUT

Self-host; Mutex; timed; morph control-flow clones.

## Oráculos

| Id | Expect |
|----|--------|
| `bootstrap-04-fib7` | ARITA → **13** accepted |
| Rust twin | fib7 == 13 |

## Checklist

- [x] Pins fib7→13 + dual-oracle
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **150/150** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
