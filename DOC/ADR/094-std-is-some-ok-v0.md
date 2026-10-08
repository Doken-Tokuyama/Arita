# ADR-094 — Option/Result `is_some`/`is_none`/`is_ok`/`is_err` v0

**Status:** Accepted (2026-09-18) — Lex measure **234/234**.

**CUT:** `STD-IS-SOME-OK-20260918`

## Decision

- Arity 0 → Bool.
- `Option.is_some()` / `is_none()`; `Result.is_ok()` / `is_err()`.
- Emit Rust homonyms; whitelist Option|Result only.

## OUT / PARK

- is_some_and / is_ok_and; Mutex PARK.

## Oracles

- `std-is-some-true` / `std-is-none-true` / `std-is-ok-true` / `std-is-err-true`
- `neg-e0206-is-some-int`
