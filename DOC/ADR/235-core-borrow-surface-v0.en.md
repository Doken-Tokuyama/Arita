Translation of `235-core-borrow-surface-v0.md`; the original is normative. / Traducción de `235-core-borrow-surface-v0.md`; el original es el normativo.

# ADR-235 — Core 0.1 borrow surface v0

- **Estado:** **aceptada** + **IMPL** (CUT `CORE-0.1-BORROW-SURFACE-20260919`)
- **CUT-ID:** `CORE-0.1-BORROW-SURFACE-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 3)
- **Relacionados:** ADR-232; ADR-009; ADR-233/234 CLOSED; HOLD `[]`/insert/Mutex
- **Gobernanza:** skip ≠ PASS; emit `#![forbid(unsafe_code)]`

## Objective

Minimal AI-native **`borrow` / `borrow mut`** surface (besides `&` / `&mut`), with no refs escaping via return/struct.

## Surface IN (v0)

| Form | Semantics |
|------|-----------|
| `borrow x` | Shared loan (≡ `&x`) |
| `borrow mut x` | Exclusive loan (≡ `&mut x`); requires `mut` binding |
| `&x` / `&mut x` | v0 alias (still valid) |

For `Int`/`Bool` (Copy): the value read via borrow is **copied** in emit (`*&` / `*&mut`); the ARITA checker still records the loan.

## OUT

- refs in `record` fields / fn return (E0318)
- lifetimes on surface, reborrow chains, `share`/actor

## Diagnostics

| Code | EN message | Case |
|------|------------|------|
| **E0202** | `borrow conflict` | overlapping loans (unchanged) |
| **E0205** | `invalid let / mut binding` | `borrow mut` on non-mut |
| **E0318** | `borrow cannot escape` | borrow as fn return or record-lit field |

## Oracles

- Pos: `ejemplos/core01/03-borrow-shared.arita` → `7`
- Neg: `neg-e0202-borrow-double-mut` (E0202 keyword), `neg-e0318-borrow-escape-return` (E0318)

## Checklist

- [x] Keyword `borrow` / `borrow mut` in pest
- [x] Emit Copy-safe `*&` / `*&mut`
- [x] HIR E0318 no-escape
- [x] Oracles + Lex bar **592/592**
