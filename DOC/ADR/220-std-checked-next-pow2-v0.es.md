Translation of `220-std-checked-next-pow2-v0.md`; the original is normative. / Traducción de `220-std-checked-next-pow2-v0.md`; el original es el normativo.

# ADR-220 — Int `checked_next_power_of_two` v0

- **Estado:** **cerrada** Lex **571/571**
- **CUT-ID:** `STD-CHECKED-NEXT-POW2-20260919`
- IN: `n.checked_next_power_of_two() → Option&lt;Int&gt;`; helper estable (paridad ADR-218; i64 no tiene método).
- n≤0 u overflow → None; tipo incorrecto → E0206.
- Oráculos: 5→Some(8), nonpos None, overflow None, neg-e0206. Baseline Lex **567/567** → **~571**.
- OUT: `next_power_of_two` sin checked; Mutex/`[]` PARK. Parser/Codegen HOLD.
