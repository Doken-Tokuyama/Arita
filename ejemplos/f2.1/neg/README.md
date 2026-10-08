# F2.1 negative oracles

| Archivo | Código | Mensaje |
|---------|--------|---------|
| `e0220-bad-cond.arita` | E0220 | `if/while condition type ≠ Bool` |
| `e0226-while-false.arita` | E0226 | `vacuous while false` (ADR-040; measure `neg-e0226-while-false`) |
| `e0226-while-false-empty.arita` | E0226 | same (empty body companion; HIR unit-tested) |
| `e0227-if-false.arita` | E0227 | `vacuous if false` (ADR-041; measure `neg-e0227-if-false`) |
| `e0227-if-false-empty.arita` | E0227 | same (empty then companion; HIR unit-tested) |
| `e0227-if-false-assert.arita` | E0227 | print-morph companion (assert-in-fn not in surface; HIR covers) |
