# ADR-192 — Int `next_multiple_of` / `checked_next_multiple_of` v0

- **Estado:** **cerrada** Lex **498/498** (helper estable; parity ADR-188)
- **CUT-ID:** `STD-NEXT-MULTIPLE-OF-20260919`
- `n.next_multiple_of(m) → Int`; `n.checked_next_multiple_of(m) → Option&lt;Int&gt;`.
- lit m==0 → E0216; wrong type → E0206; stable helper (signed int_roundings unstable).
- Oracles: next, checked some/none, neg-e0216/e0206. Baseline 493 → ~498.
- OUT: previous_multiple_of; Mutex/`[]` PARK. Parser/Codegen HOLD.
