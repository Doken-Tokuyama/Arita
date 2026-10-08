Translation of `233-core-record-v0.md`; the original is normative. / Traducción de `233-core-record-v0.md`; el original es el normativo.

# ADR-233 — Core 0.1 record v0 (declare + construct + field get)

- **Estado:** **aceptada** + **IMPL** (CUT `CORE-0.1-RECORD-20260919`)
- **CUT-ID:** `CORE-0.1-RECORD-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 1)
- **Relacionados:** ADR-232 Core 0.1 vertical; HOLD `[]`/insert/Mutex (ADR-227–229)
- **Gobernanza:** skip ≠ PASS; emit `#![forbid(unsafe_code)]`; no field-mut / enum / scenario en este CUT.

## Objective

Minimal **user `record` types** surface for the Core 0.1 vertical (CLI JSON): declare, construct, read fields (shared get).

## Surface IN (v0)

| Form | Semantics |
|------|-----------|
| `record Name { field: Type, ... }` | Module decl; `Type` ∈ {`Int`,`Bool`,`String`} MVP |
| `Name { field: expr, ... }` | Construct (all fields required) |
| `x.field` | Shared field get (no Index / no `[]`) |

## OUT (this CUT)

- field-mut / field assign
- `[]` sugar, `insert`, `Mutex`
- `enum`, nested record fields, `List`/`Map` rename
- `scenario` / files / JSON

## Diagnostics

| Code | EN message (stable) | Case |
|------|---------------------|------|
| **E0314** | `unknown field` | nonexistent field in construct or access; duplicate field in lit |
| **E0315** | `missing field` | required field missing in construct |

(`E0310` stays reserved for Index/`[]` — ADR-227; do not reuse.)

## Emit

- `record` → Rust `struct` + `#[derive(Copy, Clone, Debug)]` if all fields Copy; else `Clone, Debug`
- Field get → `base.field`
- User crates: `#![forbid(unsafe_code)]`

## Measure oracles

- Pos: `ejemplos/core01/01-record-construct-access.arita` → stdout `3` / `7`
- Neg: `neg-e0315-missing-field` (E0315), `neg-e0314-unknown-field` (E0314)

## Checklist

- [x] Syntax pest `record_item` / `record_lit` / `field_access`
- [x] HIR typecheck E0314/E0315
- [x] Codegen struct + field access
- [x] Measure oracles wired
- [x] Lex measure bar **586/586** signed (Engineer / Lex SoT)

## Queue

Next Core 0.1 slice (ADR-232): enum → borrow surface → scenario → files/JSON/CLI — GO slice 2 ENUM-MATCH after this CLOSED.
