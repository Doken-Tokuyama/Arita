Translation of `234-core-enum-match-v0.md`; the original is normative. / Traducción de `234-core-enum-match-v0.md`; el original es el normativo.

# ADR-234 — Core 0.1 enum + exhaustive match v0

- **Estado:** **aceptada** + **IMPL** (CUT `CORE-0.1-ENUM-MATCH-20260919`)
- **CUT-ID:** `CORE-0.1-ENUM-MATCH-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 2)
- **Relacionados:** ADR-232; ADR-233 RECORD CLOSED Lex 586/586; HOLD `[]`/insert/Mutex
- **Gobernanza:** skip ≠ PASS; emit `#![forbid(unsafe_code)]`

## Objective

User `enum` of **unit** variants + exhaustive `match` (gap vs already-covered Bool/Option/Result).

## Surface IN (v0)

| Form | Semantics |
|------|-----------|
| `enum Name { V1, V2, ... }` | Module decl; unit variants only |
| `Name::Variant` | Construct |
| `match e { V1 => { … } V2 => { … } }` | Exhaustive; pats = variant names or `_` |

## OUT

- payload variants (`V(Int)`), `if let` on enum, field-mut, `[]`/insert/Mutex

## Diagnostics

| Code | EN message | Case |
|------|------------|------|
| **E0316** | `non-exhaustive enum match` | missing variants and no `_` |
| **E0317** | `unknown variant` | nonexistent construct or pat |

## Emit

- `enum` → Rust `enum` + `#[derive(Copy, Clone, Debug, PartialEq, Eq)]`
- `Name::V` → `Name::V`
- match → Rust `match`

## Oracles

- Pos: `ejemplos/core01/02-enum-match.arita` → stdout `1`
- Neg: `neg-e0316-nonexhaustive-enum` (E0316), `neg-e0317-unknown-variant` (E0317)

## Checklist

- [x] Syntax pest `enum_item` / `enum_path` / variant pats
- [x] HIR E0316/E0317 + Named enum table
- [x] Codegen
- [x] Oracles + Lex bar **589/589**
