# ADR-218 — Int `is_power_of_two` v0

- **Estado:** **cerrada** Lex **566/566**
- **CUT-ID:** `STD-IS-POWER-OF-TWO-20260919`
- IN: `n.is_power_of_two() → Bool`; emit via stable helper `__arita_is_power_of_two` (i64 has no method; semantics `n > 0 && (n as u64).is_power_of_two()`).
- Wrong type → E0206.
- Oracles: 8→true; 7→false (0/-1 also false); neg-e0206. Baseline Lex **563/563** → **~566**.
- OUT: next_power_of_two; Mutex/`[]` PARK. Parser/Codegen HOLD.
