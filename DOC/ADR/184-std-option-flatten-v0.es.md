Translation of `184-std-option-flatten-v0.md`; the original is normative. / Traducción de `184-std-option-flatten-v0.md`; el original es el normativo.

# ADR-184 — Option `flatten()` v0

- **Estado:** **aceptada** + **verified** Lex **478/478**
- **CUT-ID:** `STD-OPTION-FLATTEN-20260919`
- **Fecha:** 2026-09-19
- `o.flatten() → Option` donde o: `Option<Option<T>>`; emit `.flatten()`; shared.
- Int / no anidado → E0206.
- Oráculos: Some(Some), Some(None), None outer, neg-e0206. Baseline 474→478.
- Pest mínimo: `Option` anidado en `ty_option_arg` + Let `None::<inner>` turbofish (necesario para el CUT).
- OUT: Result.flatten PARK; Mutex/`[]` PARK.
- Mirrors `_mirror_184.tgz`.
- Closed: 2026-09-19T08:27:52Z (2026-09-19 Europe/Madrid)
