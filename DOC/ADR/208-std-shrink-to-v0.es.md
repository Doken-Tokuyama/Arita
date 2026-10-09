Translation of `208-std-shrink-to-v0.md`; the original is normative. / Traducción de `208-std-shrink-to-v0.md`; el original es el normativo.

# ADR-208 — Vec/String `shrink_to` v0

- **Estado:** **cerrada** Lex **542/542**
- **CUT-ID:** `STD-SHRINK-TO-20260919`
- IN: `mut v/s.shrink_to(min_cap) → ()`; emit `.shrink_to(… as usize)`.
- lit min_cap < 0 → **E0298**; sin mut → E0202; tipo incorrecto → E0206.
- Oráculos: vec/string shrink_to, neg-e0298/e0202. Baseline Lex **538/538** → **~542**.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD.
