# ADR-238 — Core 0.1 Files / JSON / CLI bindings v0

- **Estado:** **CLOSED** Lex **601/601** (2026-09-19) (CUT `CORE-0.1-FILES-JSON-CLI-20260919`)
- **CUT-ID:** `CORE-0.1-FILES-JSON-CLI-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 6)
- **Relacionados:** ADR-232; ADR-035 host-bridges; HOLD crates.io abierto en surface
- **Gobernanza:** skip ≠ PASS; emit `#![forbid(unsafe_code)]`; host crate safe

## Objetivo

Bindings curados **files / JSON / CLI** vía `host.<fn>` + crate `arita-host-core01` (serde_json interno).

## Surface IN

| Call | Tipo | Host |
|------|------|------|
| `host.read_text(path)` | `Result<Text, Int>` | `fs::read_to_string` |
| `host.write_text(path, body)` | `Result<Int, Int>` | `fs::write` → Ok(0) |
| `host.cli_arg(i)` | `Option<Text>` | `env::args().nth` |
| `host.json_get_int(doc, key)` | `Option<Int>` | `serde_json` object number |

## OUT

- net/async, arbitrary crates.io in `.arita`, raw `std::fs` in surface

## Oráculos

- `ejemplos/core01/io/08-read-text.arita` + fixture
- `ejemplos/core01/io/09-json-get-int.arita`
- `ejemplos/core01/io/10-cli-arg.arita` (custom argv measure)

## Checklist

- [x] host crate `arita-host-core01`
- [x] HIR host typing
- [x] pest host_call priority
- [x] Lex measure **601/601**
