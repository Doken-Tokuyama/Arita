# ADR-234 — Core 0.1 enum + match exhaustivo v0

- **Estado:** **aceptada** + **IMPL** (CUT `CORE-0.1-ENUM-MATCH-20260919`)
- **CUT-ID:** `CORE-0.1-ENUM-MATCH-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 2)
- **Relacionados:** ADR-232; ADR-233 RECORD CLOSED Lex 586/586; HOLD `[]`/insert/Mutex
- **Gobernanza:** skip ≠ PASS; emit `#![forbid(unsafe_code)]`

## Objetivo

User `enum` de variantes **unitarias** + `match` exhaustivo (hueco vs Bool/Option/Result ya cubiertos).

## Surface IN (v0)

| Forma | Semántica |
|-------|-----------|
| `enum Name { V1, V2, ... }` | Decl módulo; solo unit variants |
| `Name::Variant` | Construct |
| `match e { V1 => { … } V2 => { … } }` | Exhaustivo; pats = nombres de variante o `_` |

## OUT

- payload variants (`V(Int)`), `if let` sobre enum, field-mut, `[]`/insert/Mutex

## Diagnósticos

| Código | Mensaje EN | Caso |
|--------|------------|------|
| **E0316** | `non-exhaustive enum match` | faltan variantes y no hay `_` |
| **E0317** | `unknown variant` | construct o pat inexistente |

## Emit

- `enum` → Rust `enum` + `#[derive(Copy, Clone, Debug, PartialEq, Eq)]`
- `Name::V` → `Name::V`
- match → Rust `match`

## Oráculos

- Pos: `ejemplos/core01/02-enum-match.arita` → stdout `1`
- Neg: `neg-e0316-nonexhaustive-enum` (E0316), `neg-e0317-unknown-variant` (E0317)

## Checklist

- [x] Syntax pest `enum_item` / `enum_path` / variant pats
- [x] HIR E0316/E0317 + Named enum table
- [x] Codegen
- [x] Oráculos + Lex barra **589/589**
