# ADR-233 — Core 0.1 record v0 (declare + construct + field get)

- **Estado:** **aceptada** + **IMPL** (CUT `CORE-0.1-RECORD-20260919`)
- **CUT-ID:** `CORE-0.1-RECORD-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 1)
- **Relacionados:** ADR-232 Core 0.1 vertical; HOLD `[]`/insert/Mutex (ADR-227–229)
- **Gobernanza:** skip ≠ PASS; emit `#![forbid(unsafe_code)]`; no field-mut / enum / scenario en este CUT.

## Objetivo

Surface mínima de **user `record` types** para el vertical Core 0.1 (CLI JSON): declarar, construir, leer campos (shared get).

## Surface IN (v0)

| Forma | Semántica |
|-------|-----------|
| `record Name { field: Type, ... }` | Decl módulo; `Type` ∈ {`Int`,`Bool`,`String`} MVP |
| `Name { field: expr, ... }` | Construct (todos los campos requeridos) |
| `x.field` | Shared field get (no Index / no `[]`) |

## OUT (este CUT)

- field-mut / assign a campo
- sugar `[]`, `insert`, `Mutex`
- `enum`, nested record fields, `List`/`Map` rename
- `scenario` / files / JSON

## Diagnósticos

| Código | Mensaje EN (estable) | Caso |
|--------|----------------------|------|
| **E0314** | `unknown field` | campo inexistente en construct o access; campo duplicado en lit |
| **E0315** | `missing field` | falta campo requerido en construct |

(`E0310` permanece reservado Index/`[]` — ADR-227; no reutilizar.)

## Emit

- `record` → Rust `struct` + `#[derive(Copy, Clone, Debug)]` si todos los campos Copy; si no `Clone, Debug`
- Field get → `base.field`
- Crates usuario: `#![forbid(unsafe_code)]`

## Oráculos measure

- Pos: `ejemplos/core01/01-record-construct-access.arita` → stdout `3` / `7`
- Neg: `neg-e0315-missing-field` (E0315), `neg-e0314-unknown-field` (E0314)

## Checklist

- [x] Syntax pest `record_item` / `record_lit` / `field_access`
- [x] HIR typecheck E0314/E0315
- [x] Codegen struct + field access
- [x] Oráculos measure wired
- [x] Lex measure barra **586/586** firmada (Ingeniero / SoT Lex)

## Cola

Siguiente slice Core 0.1 (ADR-232): enum → borrow surface → scenario → files/JSON/CLI — GO slice 2 ENUM-MATCH tras este CLOSED.
