# ADR-101 — E0285 checked overflow defaulted away

**Status:** Accepted (2026-09-18) — Lex **249/249**.

**CUT:** `TRAPS-CHECKED-UNWRAP-OR-20260918`

## Decision

- Reject `unwrap_or(lit)` when receiver is `checked_*` MethodCall or Path bound from `checked_*`.
- Message: `E0285: checked overflow defaulted away`.

## Oracles

- `neg-e0285-checked-add-unwrap-or-0` / `neg-e0285-checked-mul-unwrap-or-0`
- `std-checked-add-overflow-match` (pos)

## OUT

Trap A; get+default; Mutex; unwrap/?
