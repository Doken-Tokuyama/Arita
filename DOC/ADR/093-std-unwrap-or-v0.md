# ADR-093 — Option/Result `unwrap_or` v0

**Status:** Accepted (2026-09-18) — Lex measure **229/229**.

**CUT:** `STD-UNWRAP-OR-20260918`

## Decision

- `Option<T>.unwrap_or(d) → T` (Some→x; None→d)
- `Result<T,E>.unwrap_or(d) → T` (Ok→x; Err→d)
- Arity 1; `d:T`; emit Rust `.unwrap_or`.
- Explicit default API — does not collide with E0272/E0274 match-arm theater.

## OUT / PARK

- unwrap / expect / ? / unwrap_or_else / default; Mutex PARK.

## Oracles

- `std-unwrap-or-option-some` / `std-unwrap-or-option-none`
- `std-unwrap-or-result-ok` / `std-unwrap-or-result-err`
- `neg-e0206-unwrap-or-int`
