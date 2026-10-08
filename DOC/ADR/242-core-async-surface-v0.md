# ADR-242 — Core 0.2 Async surface (slice 1)

- **Estado:** **CLOSED** Lex **605/605** (2026-09-20) (CUT `CORE-0.2-ASYNC-SURFACE-20260920`)
- **CUT-ID:** `CORE-0.2-ASYNC-SURFACE-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins ADR-233-core-0.2) · Ingeniero (IMPL)
- **Prereq:** Core 0.1 CLOSED; ADR-027 async v0 (E0240/41/42)
- **Gobernanza:** skip ≠ PASS; emit `#![forbid(unsafe_code)]` + tokio bridge curado

## Objetivo

Firmar surface **`async fn` / `await`** como slice 1 de Core 0.2 (perfil service), reusando ADR-027 + oráculo nuevo que combina async con Text (0.1).

## IN

| Forma | Regla |
|-------|--------|
| `async fn … -> Io<()>` | entry + helpers |
| `await foo()` | solo en async; call a async fn mismo módulo |
| `print` / Text / Core 0.1 en async body | OK |

## OUT (este slice)

- spawn/join (slice 2)
- timeout/cancel (slice 3)
- HTTP (slice 4+)

## Oráculos

- Existentes: `async-01`, `async-02`, `neg-e0240/41/42`
- Nuevo: `core02-async-text` → `hi`

## Checklist

- [x] ejemplos/core02
- [x] measure
- [x] Lex **605/605**
