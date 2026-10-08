Translation of `093-std-unwrap-or-v0.md`; the original is normative. / Traducción de `093-std-unwrap-or-v0.md`; el original es el normativo.

# ADR-093 — Option/Result `unwrap_or` v0

**Status:** Accepted (2026-09-18) — Lex measure **229/229**.

**CUT:** `STD-UNWRAP-OR-20260918`

## Decisión

- `Option<T>.unwrap_or(d) → T` (Some→x; None→d)
- `Result<T,E>.unwrap_or(d) → T` (Ok→x; Err→d)
- Aridad 1; `d:T`; emit Rust `.unwrap_or`.
- API de default explícito — no choca con el teatro de brazos match E0272/E0274.

## OUT / PARK

- unwrap / expect / ? / unwrap_or_else / default; Mutex PARK.

## Oráculos

- `std-unwrap-or-option-some` / `std-unwrap-or-option-none`
- `std-unwrap-or-result-ok` / `std-unwrap-or-result-err`
- `neg-e0206-unwrap-or-int`
