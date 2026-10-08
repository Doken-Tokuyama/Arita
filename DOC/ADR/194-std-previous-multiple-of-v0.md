# ADR-194 — Int `previous_multiple_of` / `checked_previous_multiple_of` v0

- **Estado:** **cerrada** Lex **504/504**
- **CUT-ID:** `STD-PREV-MULTIPLE-OF-20260919`
- `n.previous_multiple_of(m) → Int`; `n.checked_previous_multiple_of(m) → Option&lt;Int&gt;`.
- lit m==0 → E0216; wrong type → E0206; helpers OK (parity ADR-192).
- Oracles: previous, checked some/none, neg-e0216/e0206. Baseline 499 → ~504.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD.
