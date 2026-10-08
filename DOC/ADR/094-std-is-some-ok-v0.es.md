Translation of `094-std-is-some-ok-v0.md`; the original is normative. / Traducción de `094-std-is-some-ok-v0.md`; el original es el normativo.

# ADR-094 — Option/Result `is_some`/`is_none`/`is_ok`/`is_err` v0

**Status:** Accepted (2026-09-18) — Lex measure **234/234**.

**CUT:** `STD-IS-SOME-OK-20260918`

## Decisión

- Aridad 0 → Bool.
- `Option.is_some()` / `is_none()`; `Result.is_ok()` / `is_err()`.
- Emit Rust homónimos; whitelist solo Option|Result.

## OUT / PARK

- is_some_and / is_ok_and; Mutex PARK.

## Oráculos

- `std-is-some-true` / `std-is-none-true` / `std-is-ok-true` / `std-is-err-true`
- `neg-e0206-is-some-int`
