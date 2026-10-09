Translation of `174-std-result-or-and-v0.md`; the original is normative. / Traducción de `174-std-result-or-and-v0.md`; el original es el normativo.

# ADR-174 — Result `or` / `and` v0

- **Estado:** **aceptada** + **verified** Lex **454/454**
- **CUT-ID:** `STD-RESULT-OR-AND-20260919`
- **Fecha:** 2026-09-19
- `r.or(other)` / `r.and(other)` → Result; emit `.or`/`.and`; shared arity 1.
- **xor VOID** → E0206. Oráculos: or/and + neg-e0206. Baseline 451→454.
- OUT: or_else/and_then; Mutex/`[]` PARK. Mirrors `_mirror_174.tgz`.
- Closed: 2026-09-19T07:36:41Z (2026-09-19 Europe/Madrid)
