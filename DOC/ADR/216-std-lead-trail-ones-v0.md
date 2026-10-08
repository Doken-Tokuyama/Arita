# ADR-216 — Int `leading_ones` / `trailing_ones` v0

- **Estado:** **cerrada** Lex **562/562**
- **CUT-ID:** `STD-LEAD-TRAIL-ONES-20260919`
- IN: `n.leading_ones()` / `trailing_ones() → Int`; emit `(….leading_ones() as i64)` / `(….trailing_ones() as i64)`.
- Wrong type → E0206. (Also: 0.leading_ones→0; 8.trailing_ones→0.)
- Oracles: leading -1→64, trailing 7→3, neg-e0206. Baseline Lex **559/559** → **~562**.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD.
