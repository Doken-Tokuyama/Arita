# ADR-178 — Int `abs_diff` v0

- **Estado:** **aceptada** + **verified** Lex **463/463**
- **CUT-ID:** `STD-ABS-DIFF-20260919`
- **Fecha:** 2026-09-19
- `a.abs_diff(b) → Int`; emit `(a.abs_diff(b) as i64)`; shared arity 1.
- Wrong type → E0206; no overflow panic on MIN/MAX.
- Oracles: abs-diff, min-max safe, neg-e0206. Baseline 460→463.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD.
- Mirrors `_mirror_178.tgz`.
- Closed: 2026-09-19T07:56:44Z (2026-09-19 Europe/Madrid)
