# ADR-184 — Option `flatten()` v0

- **Estado:** **aceptada** + **verified** Lex **478/478**
- **CUT-ID:** `STD-OPTION-FLATTEN-20260919`
- **Fecha:** 2026-09-19
- `o.flatten() → Option` where o: `Option<Option<T>>`; emit `.flatten()`; shared.
- Int / non-nested → E0206.
- Oracles: Some(Some), Some(None), None outer, neg-e0206. Baseline 474→478.
- Minimal pest: nested `Option` in `ty_option_arg` + Let `None::<inner>` turbofish (needed for CUT).
- OUT: Result.flatten PARK; Mutex/`[]` PARK.
- Mirrors `_mirror_184.tgz`.
- Closed: 2026-09-19T08:27:52Z (2026-09-19 Europe/Madrid)
