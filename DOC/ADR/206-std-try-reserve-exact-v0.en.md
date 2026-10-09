Translation of `206-std-try-reserve-exact-v0.md`; the original is normative. / Traducción de `206-std-try-reserve-exact-v0.md`; el original es el normativo.

# ADR-206 — Vec/String `try_reserve_exact` v0

- **Estado:** **cerrada** Lex **537/537**
- **CUT-ID:** `STD-TRY-RESERVE-EXACT-20260919`
- IN: `mut v/s.try_reserve_exact(n) → Result&lt;(), Int&gt;`; emit `.try_reserve_exact(…).map_err(|_| 0)`.
- lit n&lt;0 → E0294; missing mut → E0202.
- Oracles: vec/string ok, neg-e0294/e0202. Baseline 533 → ~537.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD.
