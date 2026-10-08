# ADR-235 — Core 0.1 borrow surface v0

- **Estado:** **aceptada** + **IMPL** (CUT `CORE-0.1-BORROW-SURFACE-20260919`)
- **CUT-ID:** `CORE-0.1-BORROW-SURFACE-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 3)
- **Relacionados:** ADR-232; ADR-009; ADR-233/234 CLOSED; HOLD `[]`/insert/Mutex
- **Gobernanza:** skip ≠ PASS; emit `#![forbid(unsafe_code)]`

## Objetivo

Surface mínima AI-native **`borrow` / `borrow mut`** (además de `&` / `&mut`), sin refs escapando en return/struct.

## Surface IN (v0)

| Forma | Semántica |
|-------|-----------|
| `borrow x` | Shared loan (≡ `&x`) |
| `borrow mut x` | Exclusive loan (≡ `&mut x`); exige binding `mut` |
| `&x` / `&mut x` | Alias v0 (sigue válido) |

Para `Int`/`Bool` (Copy): el valor leído vía borrow se **copia** en emit (`*&` / `*&mut`); el checker ARITA sigue registrando el préstamo.

## OUT

- refs en campos de `record` / return de fn (E0318)
- lifetimes en surface, reborrow chains, `share`/actor

## Diagnósticos

| Código | Mensaje EN | Caso |
|--------|------------|------|
| **E0202** | `borrow conflict` | préstamos solapados (sin cambio) |
| **E0205** | `invalid let / mut binding` | `borrow mut` sobre non-mut |
| **E0318** | `borrow cannot escape` | borrow como return de fn o field de record lit |

## Oráculos

- Pos: `ejemplos/core01/03-borrow-shared.arita` → `7`
- Neg: `neg-e0202-borrow-double-mut` (E0202 keyword), `neg-e0318-borrow-escape-return` (E0318)

## Checklist

- [x] Keyword `borrow` / `borrow mut` in pest
- [x] Emit Copy-safe `*&` / `*&mut`
- [x] HIR E0318 no-escape
- [x] Oráculos + Lex barra **592/592**
