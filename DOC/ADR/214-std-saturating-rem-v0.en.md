Translation of `214-std-saturating-rem-v0.md`; the original is normative. / Traducción de `214-std-saturating-rem-v0.md`; el original es el normativo.

# ADR-214 — Int `saturating_rem` v0

- **Estado:** **cerrada** Lex **558/558**
- **CUT-ID:** `STD-SATURATING-REM-20260919`
- IN: `a.saturating_rem(b) → Int`; emit via stable helper `__arita_saturating_rem` (Rust i64 has no `saturating_rem` on 1.97; semantics = rem, MIN/-1 → 0).
- lit b==0 → **E0216**; MIN/-1 → 0; wrong type → E0206.
- Oracles: normal, min/-1, neg-e0216/e0206. Baseline Lex **554/554** → **~558**.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD.
