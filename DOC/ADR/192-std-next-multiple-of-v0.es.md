Translation of `192-std-next-multiple-of-v0.md`; the original is normative. / Traducción de `192-std-next-multiple-of-v0.md`; el original es el normativo.

# ADR-192 — Int `next_multiple_of` / `checked_next_multiple_of` v0

- **Estado:** **cerrada** Lex **498/498** (helper estable; parity ADR-188)
- **CUT-ID:** `STD-NEXT-MULTIPLE-OF-20260919`
- `n.next_multiple_of(m) → Int`; `n.checked_next_multiple_of(m) → Option&lt;Int&gt;`.
- lit m==0 → E0216; tipo incorrecto → E0206; helper estable (int_roundings signed inestable).
- Oráculos: next, checked some/none, neg-e0216/e0206. Baseline 493 → ~498.
- OUT: previous_multiple_of; Mutex/`[]` PARK. Parser/Codegen HOLD.
