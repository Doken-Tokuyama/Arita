Translation of `236-core-text-bytes-v0.md`; the original is normative. / Traducción de `236-core-text-bytes-v0.md`; el original es el normativo.

# ADR-236 — Core 0.1 Text / Bytes v0

- **Estado:** **CLOSED** Lex **595/595** (2026-09-19) (CUT `CORE-0.1-TEXT-BYTES-20260919`)
- **CUT-ID:** `CORE-0.1-TEXT-BYTES-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 4)
- **Relacionados:** ADR-232; String F2 sigue válido como alias de Text
- **Gobernanza:** skip ≠ PASS; emit `#![forbid(unsafe_code)]`

## Objective

Real AI-native base types **`Text`** and **`Bytes`** (no theater): Text ≡ UTF-8 string; Bytes ≡ byte sequence (`Vec<u8>`).

## Surface IN

| Form | Semantics | Emit Rust |
|------|-----------|-----------|
| `Text` | text type | `String` |
| `String` | v0 alias of Text | `String` |
| `Bytes` | bytes type | `Vec<u8>` |
| lit `"…"` | `Text`/`String` | `String` |
| `t.as_bytes()` | `Text`→`Bytes` (shared copy) | `t.as_bytes().to_vec()` |
| `b.len()` | bytes length → Int | `b.len() as i64` |

## OUT

- raw byte lit `b"..."`, `[]` indexing, in-place byte mutate without API

## Oracles

- Pos Text: `ejemplos/core01/04-text-lit.arita` → `hi`
- Pos Bytes: `ejemplos/core01/05-bytes-as-bytes.arita` → `2`
- Neg: `neg-e0206-as-bytes-on-int` (E0206)

## Checklist

- [x] pest `ty_text` / `ty_bytes`
- [x] HIR Bytes + `as_bytes`
- [x] Codegen
- [x] Lex measure **595/595**
