Translation of `218-std-is-power-of-two-v0.md`; the original is normative. / Traducción de `218-std-is-power-of-two-v0.md`; el original es el normativo.

# ADR-218 — Int `is_power_of_two` v0

- **Estado:** **cerrada** Lex **566/566**
- **CUT-ID:** `STD-IS-POWER-OF-TWO-20260919`
- IN: `n.is_power_of_two() → Bool`; emit vía helper estable `__arita_is_power_of_two` (i64 no tiene método; semántica `n > 0 && (n as u64).is_power_of_two()`).
- Tipo incorrecto → E0206.
- Oráculos: 8→true; 7→false (0/-1 también false); neg-e0206. Baseline Lex **563/563** → **~566**.
- OUT: next_power_of_two; Mutex/`[]` PARK. Parser/Codegen HOLD.
