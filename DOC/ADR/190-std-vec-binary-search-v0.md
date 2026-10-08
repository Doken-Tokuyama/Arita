# ADR-190 — Vec `binary_search` v0

- **Estado:** **aceptada** + **verified** Lex **492/492**
- **CUT-ID:** `STD-VEC-BINARY-SEARCH-20260919`
- **Fecha:** 2026-09-19
- `v.binary_search(x) → Result<Int,Int>`; emit `.binary_search(&x).map(|i| i as i64).map_err(|i| i as i64)`.
- Ok=index, Err=insert point; `Vec<Int>` F1.1; wrong type → E0206.
- Oracles: ok/err + neg-e0206. Baseline 489→492.
- OUT: binary_search_by*; Mutex/`[]` PARK. Parser/Codegen HOLD.
- Mirrors `_mirror_190.tgz`.
- Closed: 2026-09-19T08:57:46Z (2026-09-19 Europe/Madrid)
