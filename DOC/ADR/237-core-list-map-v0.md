# ADR-237 — Core 0.1 List / Map names v0

- **Estado:** **CLOSED** Lex **598/598** (2026-09-19) (CUT `CORE-0.1-LIST-MAP-NAMES-20260919`)
- **CUT-ID:** `CORE-0.1-LIST-MAP-NAMES-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 5)
- **Relacionados:** ADR-232; Vec F2 sigue válido; HOLD insert panic / `[]` (E0310)
- **Gobernanza:** skip ≠ PASS; emit `#![forbid(unsafe_code)]`

## Objetivo

Nombres AI-native **`List`** / **`Map`** + APIs fallible (`get`→Option). Sin sugar `[]`, sin `insert` en surface.

## Surface IN

| Forma | Semántica | Emit Rust |
|-------|-----------|-----------|
| `List<Int>` | alias de `Vec<Int>` | `Vec<i64>` |
| `List::new()` | lista vacía | `Vec::new()` |
| `Map<Text, Int>` / `Map<String, Int>` | mapa | `HashMap<String, i64>` |
| `Map::new()` | mapa vacío | `HashMap::new()` |
| `m.put(k, v)` | write no-panic (reemplaza) | `{ let _ = m.insert(k, v); }` |
| `m.get(k)` | fallible | `m.get(&k).cloned()` → `Option` |
| `list.get(i)` | fallible (F2) | usize get + cloned |

## OUT

- `insert` nombre surface, `[]`, Map keys ≠ Text/String, valores ≠ Int (v0)

## Oráculos

- `ejemplos/core01/06-list-alias.arita` → `2`
- `ejemplos/core01/07-map-put-get.arita` → `7`
- `neg-e0206-map-insert-banned` (E0206 si `insert`)

## Checklist

- [x] pest List/Map
- [x] HIR Map + put/get
- [x] Codegen HashMap
- [x] Lex measure **598/598**
