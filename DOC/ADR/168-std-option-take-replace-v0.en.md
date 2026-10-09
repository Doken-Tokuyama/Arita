Translation of `168-std-option-take-replace-v0.md`; the original is normative. / Traducción de `168-std-option-take-replace-v0.md`; el original es el normativo.

# ADR-168 — Option `take` / `replace` v0

- **Estado:** **aceptada** + **verified** Lex **440/440**
- **CUT-ID:** `STD-OPTION-TAKE-REPLACE-20260919`
- **Fecha:** 2026-09-19
- `mut o.take() → Option`; `mut o.replace(x) → Option`; emit `.take()` / `.replace(x)`.
- missing mut → E0202; wrong type → E0206. String.replace (2-arg) unchanged.
- Oracles: take-some/none, replace, neg-e0202/e0206. Baseline 435→440.
- OUT: Result.take; Mutex/`[]` PARK. Mirrors `_mirror_168.tgz`.
- Closed: 2026-09-19T06:58:56Z (2026-09-19 Europe/Madrid)
