# ADR-224 — Vec `extend` v0

- **Estado:** **cerrada** Lex **581/581** (verified)
- **CUT-ID:** `STD-VEC-EXTEND-20260919`
- IN: `mut a.extend(b) → ()`; emit `a.extend_from_slice(&b)`; recv mut, arg shared (b intact).
- ≠ append (drain); sin mut → E0202; wrong type → E0206.
- Oracles: extend, empty, neg-e0202/e0206. Baseline Lex **577/577** → **~581**.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD.
