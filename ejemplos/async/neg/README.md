# Async negative oracles (ADR-027 / ADR-039)

| File | Code | Message |
|------|------|---------|
| `e0240-await-outside.arita` | E0240 | await outside async function |
| `e0241-async-illegal.arita` | E0241 | async feature not allowed here |
| `e0242-borrow-across-await.arita` | E0242 | borrow held across await |

Measure ids: `neg-e0240-await-outside`, `neg-e0241-async-illegal`, `neg-e0242-borrow-across-await`.

E0242 is rejected in ARITA HIR/`parse_lower_check` (before emit/rustc). Control: owned `String` move across `await` without borrow remains PASS.
