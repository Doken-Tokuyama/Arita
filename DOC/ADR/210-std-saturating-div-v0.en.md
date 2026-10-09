Translation of `210-std-saturating-div-v0.md`; the original is normative. / Traducción de `210-std-saturating-div-v0.md`; el original es el normativo.

# ADR-210 — Int `saturating_div` v0

- **Estado:** **cerrada** Lex **547/547**
- **CUT-ID:** `STD-SATURATING-DIV-20260919`
- IN: `a.saturating_div(b) → Int`; emit `.saturating_div(b)`.
- lit b==0 → **E0216**; MIN/-1 → MAX (no panic); wrong type → E0206.
- Oracles: normal, min/-1, neg-e0216/e0206. Baseline Lex **543/543** → **~547**.
- OUT: wrapping_div; Mutex/`[]` PARK. Parser/Codegen HOLD.
