# ADR-243 — Core 0.2 Task spawn/join v0

- **Estado:** **CLOSED** Lex **607/607** (CUT `CORE-0.2-TASK-SPAWN-20260920`)
- **Gate:** [`../GATE-CORE02-SPAWN-20260920.md`](../GATE-CORE02-SPAWN-20260920.md) · Measure TASK-SPAWN 2026-09-20
- **Barra:** `arita measure` → **607/607 accepted** (exit 0); oracles `core02-spawn-join` + `neg-e0240-spawn-outside`
- **CUT-ID:** `CORE-0.2-TASK-SPAWN-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins 0.2) · Ingeniero (IMPL)
- **Gobernanza:** skip ≠ PASS; sin Mutex; sin unwrap en emit; current_thread

## Surface IN

| Forma | Tipo | Emit |
|-------|------|------|
| `spawn(foo())` | `Task` | `tokio::spawn(foo())` |
| `await join(t)` | efecto | `{ let _ = t.await; }` |

- Solo dentro de `async fn`
- `foo` async fn mismo módulo, 0 params, `Io<()>`
- `Task` opaco (no fields)

## OUT

- spawn libre / JoinHandle en surface / multi-thread / Mutex

## Diagnósticos

- spawn/join fuera de async → **E0240**
- spawn de no-async / arity → **E0203** / **E0241**
- `tokio::spawn` en surface → **E0261** (ya)

## Oráculos

- pos: `ejemplos/core02/02-spawn-join.arita` → `w` then `done`
- neg: `neg-e0240-spawn-outside`

## Cierre (2026-09-20)

- Ingeniero **GO** CLOSED slice 2 (gate + Veyra companion OK; rustfmt drift non-blocking).
- Docs firma este ADR **CLOSED** Lex **607/607**. HOLDs intactos.
- Siguiente: ADR-233 **GO** slice 3 TIMEOUT-CANCEL (ADR-244 DRAFT PREP; land solo tras nuevo measure+gate).
