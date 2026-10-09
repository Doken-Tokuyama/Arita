Translation of `202-std-ok-or-ok-err-v0.md`; the original is normative. / Traducción de `202-std-ok-or-ok-err-v0.md`; el original es el normativo.

# ADR-202 — Option `ok_or` / Result `ok` / `err` v0

- **Estado:** **cerrada** Lex **527/527**
- **CUT-ID:** `STD-OK-OR-OK-ERR-20260919`
- IN: `o.ok_or(e) → Result`; `r.ok() → Option`; `r.err() → Option`; emit Rust.
- Wrong type → E0206.
- Oracles: ok_or some/none, result ok/err, neg-e0206. Baseline 522 → ~527.
- OUT: ok_or_else; Mutex/`[]` PARK. Parser/Codegen HOLD.
