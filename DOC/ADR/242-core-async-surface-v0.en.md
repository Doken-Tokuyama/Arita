Translation of `242-core-async-surface-v0.md`; the original is normative. / Traducción de `242-core-async-surface-v0.md`; el original es el normativo.

# ADR-242 — Core 0.2 Async surface (slice 1)

- **Estado:** **CLOSED** Lex **605/605** (2026-09-20) (CUT `CORE-0.2-ASYNC-SURFACE-20260920`)
- **CUT-ID:** `CORE-0.2-ASYNC-SURFACE-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins ADR-233-core-0.2) · Ingeniero (IMPL)
- **Prereq:** Core 0.1 CLOSED; ADR-027 async v0 (E0240/41/42)
- **Gobernanza:** skip ≠ PASS; emit `#![forbid(unsafe_code)]` + tokio bridge curado

## Objective

Sign the **`async fn` / `await`** surface as Core 0.2 slice 1 (service profile), reusing ADR-027 + a new oracle that combines async with Text (0.1).

## IN

| Form | Rule |
|------|------|
| `async fn … -> Io<()>` | entry + helpers |
| `await foo()` | only in async; call to async fn in the same module |
| `print` / Text / Core 0.1 in async body | OK |

## OUT (this slice)

- spawn/join (slice 2)
- timeout/cancel (slice 3)
- HTTP (slice 4+)

## Oracles

- Existing: `async-01`, `async-02`, `neg-e0240/41/42`
- New: `core02-async-text` → `hi`

## Checklist

- [x] ejemplos/core02
- [x] measure
- [x] Lex **605/605**
