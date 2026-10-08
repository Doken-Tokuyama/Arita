Español | [English](README.md)

# Oráculos negativos async (ADR-027 / ADR-039)

| Archivo | Código | Mensaje |
|------|------|---------|
| `e0240-await-outside.arita` | E0240 | await outside async function |
| `e0241-async-illegal.arita` | E0241 | async feature not allowed here |
| `e0242-borrow-across-await.arita` | E0242 | borrow held across await |

Measure ids: `neg-e0240-await-outside`, `neg-e0241-async-illegal`, `neg-e0242-borrow-across-await`.

E0242 se rechaza en ARITA HIR/`parse_lower_check` (antes de emit/rustc). Control: un move de `String` owned a través de `await` sin borrow sigue siendo PASS.
