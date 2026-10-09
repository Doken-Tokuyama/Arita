Translation of `095-bootstrap-06.md`; the original is normative. / Traducción de `095-bootstrap-06.md`; el original es el normativo.

# ADR-095 — Bootstrap-06 dual-oracle (`fib9→34`)

**Status:** Accepted (2026-09-18) — Lex measure **235/235**.

**CUT:** `BOOTSTRAP-06-20260918`

## Decision

- Dual-oracle `fib9 → 34` (measure `bootstrap-06` + Rust twin `GOLDEN_FIB9=34`).
- Unrolled Int helpers (Expr/Let only; no while).

## OUT / PARK

- self-host / timed / recursive; Mutex PARK.

## Oracles

- `bootstrap-06` → stdout **34**
- Rust `fib9() == GOLDEN_FIB9`
