Translation of `243-core-task-spawn-v0.md`; the original is normative. / Traducción de `243-core-task-spawn-v0.md`; el original es el normativo.

# ADR-243 — Core 0.2 Task spawn/join v0

- **Estado:** **CLOSED** Lex **607/607** (CUT `CORE-0.2-TASK-SPAWN-20260920`)
- **Gate:** `../GATE-CORE02-SPAWN-20260920.md` (not in the public export / no incluido en el export público) · Measure TASK-SPAWN 2026-09-20
- **Barra:** `arita measure` → **607/607 accepted** (exit 0); oracles `core02-spawn-join` + `neg-e0240-spawn-outside`
- **CUT-ID:** `CORE-0.2-TASK-SPAWN-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins 0.2) · Ingeniero (IMPL)
- **Gobernanza:** skip ≠ PASS; sin Mutex; sin unwrap en emit; current_thread

## Surface IN

| Form | Type | Emit |
|------|------|------|
| `spawn(foo())` | `Task` | `tokio::spawn(foo())` |
| `await join(t)` | effect | `{ let _ = t.await; }` |

- Only inside `async fn`
- `foo` async fn same module, 0 params, `Io<()>`
- Opaque `Task` (no fields)

## OUT

- free spawn / JoinHandle on surface / multi-thread / Mutex

## Diagnostics

- spawn/join outside async → **E0240**
- spawn of non-async / arity → **E0203** / **E0241**
- `tokio::spawn` on surface → **E0261** (already)

## Oracles

- pos: `ejemplos/core02/02-spawn-join.arita` → `w` then `done`
- neg: `neg-e0240-spawn-outside`

## Close (2026-09-20)

- Engineer **GO** CLOSED slice 2 (gate + Veyra companion OK; rustfmt drift non-blocking).
- Docs signs this ADR **CLOSED** Lex **607/607**. HOLDs intact.
- Next: ADR-233 **GO** slice 3 TIMEOUT-CANCEL (ADR-244 DRAFT PREP; land only after new measure+gate).
