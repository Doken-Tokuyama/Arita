Translation of `023-int-cmp.md`; the original is normative. / Traducción de `023-int-cmp.md`; el original es el normativo.

# ADR-023 — Int comparisons → Bool (measured)

- **Estado:** **aceptada**
- **CUT-ID:** `INT-CMP-20260913`
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (GO)
- **Relacionados:** ADR-014 (F2.1), ADR-006, ADR-022

## Decision

Surface medido incluye comparaciones **Int → Bool**:

```text
primary (==|!=|<|>|<=|>=) primary
```

Usos admitidos:
1. Condition of `if` / `while` (already in `ejemplos/f2.1/03-while-count` via `i < 3`)
2. Init de `let …: Bool = cmp_expr`
3. Directo en `if cmp_expr { … }`

Operands v0: `ident` | `int_lit` (Int). No String/Bool/Vec as operandos de cmp.

## Oracles

| Id | Path | stdout |
|----|------|--------|
| `f21-04-cmp-if` | `ejemplos/f2.1/04-cmp-if.arita` | `yes` |
| `f21-05-cmp-let` | `ejemplos/f2.1/05-cmp-let.arita` | `eq` |

Negativos no-Bool en cond siguen **E0220** (p.ej. cmp mal tipado que no produce Bool).

## OUT

- Cmp de String/Bool/Vec
- Encadenados `a < b < c`
- `match` on result cmp as expression (stmt match remains ADR-015)
