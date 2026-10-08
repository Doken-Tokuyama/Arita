# ADR-222 — Vec `append` v0

- **Estado:** **cerrada** Lex **576/576** (verified)
- **CUT-ID:** `STD-VEC-APPEND-20260919`
- IN: `mut a.append(mut b) → ()`; emit `a.append(&mut b)`; both exclusive mut.
- missing mut → E0202; wrong type → E0206.
- Oracles: append, empty, neg-e0202/e0206. Baseline Lex **572/572** → **~576**.
- OUT: extend; Mutex/`[]` PARK. Parser/Codegen HOLD.
