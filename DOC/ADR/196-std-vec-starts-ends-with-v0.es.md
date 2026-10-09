Translation of `196-std-vec-starts-ends-with-v0.md`; the original is normative. / Traducción de `196-std-vec-starts-ends-with-v0.md`; el original es el normativo.

# ADR-196 — Vec `starts_with` / `ends_with` v0

- **Estado:** **cerrada** Lex **509/509**
- **CUT-ID:** `STD-VEC-STARTS-ENDS-WITH-20260919`
- IN: `v.starts_with(prefix)` / `ends_with(suffix)` → Bool; emit `.starts_with(&…)` / `.ends_with(&…)`; arg Vec&lt;Int&gt;.
- Tipo incorrecto → E0206; métodos String sin cambios.
- Oráculos: starts true/false, ends true, neg-e0206. Baseline 505 → ~509.
- OUT: Mutex/`[]` PARK. Parser/Codegen HOLD.

- Migración: `neg-e0206-starts/ends-with-vec` → E0203 (arg elem); recv Vec ahora en whitelist.
