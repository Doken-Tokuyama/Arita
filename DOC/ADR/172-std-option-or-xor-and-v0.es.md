Translation of `172-std-option-or-xor-and-v0.md`; the original is normative. / Traducción de `172-std-option-or-xor-and-v0.md`; el original es el normativo.

# ADR-172 — Option `or` / `and` / `xor` v0

- **Estado:** **aceptada** + **verified** Lex **450/450**
- **CUT-ID:** `STD-OPTION-OR-XOR-AND-20260919`
- **Fecha:** 2026-09-19
- `o.or(other)` / `and` / `xor` → Option; emit `.or`/`.and`/`.xor`; shared arity 1.
- Tipo incorrecto → E0206. Oráculos: or/and/xor + neg-e0206. Baseline 446→450.
- OUT: or_else/and_then; Result.*; Mutex/`[]` PARK. Mirrors `_mirror_172.tgz`.
- Closed: 2026-09-19T07:25:58Z (2026-09-19 Europe/Madrid)
