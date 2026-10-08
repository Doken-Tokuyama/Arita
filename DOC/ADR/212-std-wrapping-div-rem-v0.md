# ADR-212 — Int `wrapping_div` / `wrapping_rem` v0

- **Estado:** **cerrada** Lex **553/553**
- **CUT-ID:** `STD-WRAPPING-DIV-REM-20260919`
- IN: `a.wrapping_div(b)` / `wrapping_rem(b) → Int`; emit Rust.
- lit b==0 → **E0216**; MIN/-1 → MIN / 0 (no panic); wrong type → E0206.
- Oracles: div, min/-1, rem, neg-e0216/e0206. Baseline Lex **548/548** → **~553**.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD.
