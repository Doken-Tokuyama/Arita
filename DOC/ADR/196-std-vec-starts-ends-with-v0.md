# ADR-196 — Vec `starts_with` / `ends_with` v0

- **Estado:** **cerrada** Lex **509/509**
- **CUT-ID:** `STD-VEC-STARTS-ENDS-WITH-20260919`
- IN: `v.starts_with(prefix)` / `ends_with(suffix)` → Bool; emit `.starts_with(&…)` / `.ends_with(&…)`; Vec&lt;Int&gt; arg.
- Wrong type → E0206; String methods unchanged.
- Oracles: starts true/false, ends true, neg-e0206. Baseline 505 → ~509.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD.

- Migración: `neg-e0206-starts/ends-with-vec` → E0203 (elem arg); recv Vec ahora whitelist.
