# ADR-200 — Option/Result `transpose()` v0 (milestone)

- **Estado:** **cerrada** Lex **521/521**
- **CUT-ID:** `STD-TRANSPOSE-20260919`
- IN: `Option&lt;Result&lt;T,E&gt;&gt;.transpose() → Result&lt;Option&lt;T&gt;,E&gt;`; `Result&lt;Option&lt;T&gt;,E&gt;.transpose() → Option&lt;Result&lt;T,E&gt;&gt;`; emit `.transpose()`.
- Minimal nest surface (parity flatten); Int → E0206.
- Oracles: option ok/err/none, result some/none, neg-e0206. Baseline 515 → ~521.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD except nest ty.
