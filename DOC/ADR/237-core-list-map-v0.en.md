Translation of `237-core-list-map-v0.md`; the original is normative. / Traducción de `237-core-list-map-v0.md`; el original es el normativo.

# ADR-237 — Core 0.1 List / Map names v0

- **Estado:** **CLOSED** Lex **598/598** (2026-09-19) (CUT `CORE-0.1-LIST-MAP-NAMES-20260919`)
- **CUT-ID:** `CORE-0.1-LIST-MAP-NAMES-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 5)
- **Relacionados:** ADR-232; Vec F2 sigue válido; HOLD insert panic / `[]` (E0310)
- **Gobernanza:** skip ≠ PASS; emit `#![forbid(unsafe_code)]`

## Objective

AI-native **`List`** / **`Map`** names + fallible APIs (`get`→Option). No `[]` sugar, no `insert` on surface.

## Surface IN

| Form | Semantics | Emit Rust |
|------|-----------|-----------|
| `List<Int>` | alias of `Vec<Int>` | `Vec<i64>` |
| `List::new()` | empty list | `Vec::new()` |
| `Map<Text, Int>` / `Map<String, Int>` | map | `HashMap<String, i64>` |
| `Map::new()` | empty map | `HashMap::new()` |
| `m.put(k, v)` | no-panic write (replace) | `{ let _ = m.insert(k, v); }` |
| `m.get(k)` | fallible | `m.get(&k).cloned()` → `Option` |
| `list.get(i)` | fallible (F2) | usize get + cloned |

## OUT

- surface name `insert`, `[]`, Map keys ≠ Text/String, values ≠ Int (v0)

## Oracles

- `ejemplos/core01/06-list-alias.arita` → `2`
- `ejemplos/core01/07-map-put-get.arita` → `7`
- `neg-e0206-map-insert-banned` (E0206 if `insert`)

## Checklist

- [x] pest List/Map
- [x] HIR Map + put/get
- [x] Codegen HashMap
- [x] Lex measure **598/598**
