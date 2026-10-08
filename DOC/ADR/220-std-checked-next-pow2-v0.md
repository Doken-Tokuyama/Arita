# ADR-220 — Int `checked_next_power_of_two` v0

- **Estado:** **cerrada** Lex **571/571**
- **CUT-ID:** `STD-CHECKED-NEXT-POW2-20260919`
- IN: `n.checked_next_power_of_two() → Option&lt;Int&gt;`; stable helper (parity ADR-218; i64 has no method).
- n≤0 or overflow → None; wrong type → E0206.
- Oracles: 5→Some(8), nonpos None, overflow None, neg-e0206. Baseline Lex **567/567** → **~571**.
- OUT: bare next_power_of_two; Mutex/`[]` PARK. Parser/Codegen HOLD.
