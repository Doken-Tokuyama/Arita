[Original](README.md) | Español | [English](README.en.md)

# Ejemplos bootstrap (ADR-037 / ADR-042 / ADR-057 / ADR-068)

Fns puras dual-oracle: el mismo algoritmo en Rust (`crates/arita-cli/src/bootstrap.rs`) y aquí.

| Archivo | Fn | Golden stdout | Measure id |
|---------|-----|---------------|------------|
| `01-fact.arita` | `fact5` unrolled `1*2*3*4*5` | `120` | `bootstrap-01` |
| `02-sum.arita` | `sum5` unrolled `1+2+3+4+5` | `15` | `bootstrap-02` |
| `03-fib.arita` | `fib6` unrolled F0‥F6 | `8` | `bootstrap-03` |
| `04-fib7.arita` | `fib7` unrolled F0‥F7 | `13` | `bootstrap-04` |

No es self-host; no hay `arita bootstrap-check`. Helpers Int: solo Expr/Let (sin `while`).
- `06-fib9.arita` — fib9→34 (ADR-095)
- `07-fib10.arita` — fib10→55 (ADR-104)
