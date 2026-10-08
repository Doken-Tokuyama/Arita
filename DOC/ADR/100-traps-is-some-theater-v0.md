# ADR-100 — Traps `is_some`/`is_ok` theater (E0284) v0

**Status:** **VOID** (2026-09-18) — Lex probe **no HOLE**; bar unchanged **244/244**.

**CUT:** `TRAPS-IS-SOME-THEATER-20260918`

## Decision

Sondeo HOLE: forms `assert o.is_some() == true` / `assert r.is_ok() == true` are **not accepted** today.

- Direct: **E0006** (assert_side sin `method_call_other`).
- Helper + `== true`: **E0211**.
- Helper == helper: **E0215**.

No IMPL E0284. Evidence: `DOC/reviews/HOLE-E0284-is-some-theater-20260918.md`.

## OUT / PARK

B2 checked+unwrap_or; A wrapping happy-assert; Mutex; unwrap/?

## Re-open condition

If CUT amplía `assert_side` con `method_call_other`, re-probe before assigning E0284.
