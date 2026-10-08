# ADR-188 — Int `div_ceil` / `div_floor` v0

- **Estado:** **aceptada** + **verified** Lex **488/488**
- **CUT-ID:** `STD-DIV-CEIL-FLOOR-20260919`
- **Fecha:** 2026-09-19
- `a.div_ceil(b)` / `div_floor(b) → Int`; emit `__arita_div_*` stable helper (rustc 1.97 signed `int_roundings` still unstable).
- lit b==0 → E0216; MIN/-1 → E0217; wrong type → E0206.
- Oracles: ceil/floor signed + neg-e0216/e0206. Baseline 484→488.
- OUT: checked_*; Mutex/`[]` PARK. Parser HOLD.
- Mirrors `_mirror_188.tgz`.
- Closed: 2026-09-19T08:47:21Z (2026-09-19 Europe/Madrid)
