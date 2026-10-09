Translation of `204-std-try-reserve-v0.md`; the original is normative. / Traducción de `204-std-try-reserve-v0.md`; el original es el normativo.

# ADR-204 — Vec/String `try_reserve` v0

- **Estado:** **cerrada** Lex **532/532**
- **CUT-ID:** `STD-TRY-RESERVE-20260919`
- IN: `mut v/s.try_reserve(n) → Result&lt;(), Int&gt;`; emit try_reserve; map TryReserveError → Err(0).
- lit n&lt;0 → E0294; missing mut → E0202; wrong type → E0206.
- Oracles: vec/string ok, neg-e0294/e0202. Baseline 528 → ~532.
- OUT: try_reserve_exact; Mutex/`[]` PARK. Parser/Codegen HOLD.
