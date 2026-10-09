Translation of `101-traps-checked-unwrap-or-v0.md`; the original is normative. / Traducción de `101-traps-checked-unwrap-or-v0.md`; el original es el normativo.

# ADR-101 — E0285 checked overflow defaulted away

**Status:** Accepted (2026-09-18) — Lex **249/249**.

**CUT:** `TRAPS-CHECKED-UNWRAP-OR-20260918`

## Decisión

- Rechazar `unwrap_or(lit)` cuando el receptor es un MethodCall `checked_*` o un Path enlazado desde `checked_*`.
- Mensaje: `E0285: checked overflow defaulted away`.

## Oráculos

- `neg-e0285-checked-add-unwrap-or-0` / `neg-e0285-checked-mul-unwrap-or-0`
- `std-checked-add-overflow-match` (pos)

## OUT

Trap A; get+default; Mutex; unwrap/?
