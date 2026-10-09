Translation of `100-traps-is-some-theater-v0.md`; the original is normative. / Traducción de `100-traps-is-some-theater-v0.md`; el original es el normativo.

# ADR-100 — Traps teatro `is_some`/`is_ok` (E0284) v0

**Status:** **VOID** (2026-09-18) — Lex probe **no HOLE**; bar unchanged **244/244**.

**CUT:** `TRAPS-IS-SOME-THEATER-20260918`

## Decisión

Sondeo HOLE: las formas `assert o.is_some() == true` / `assert r.is_ok() == true` **no se aceptan** hoy.

- Directo: **E0006** (assert_side sin `method_call_other`).
- Helper + `== true`: **E0211**.
- Helper == helper: **E0215**.

Sin IMPL E0284. Evidencia: `DOC/reviews/HOLE-E0284-is-some-theater-20260918.md`.

## OUT / PARK

B2 checked+unwrap_or; A wrapping happy-assert; Mutex; unwrap/?

## Condición de reapertura

Si el CUT amplía `assert_side` con `method_call_other`, volver a sondear antes de asignar E0284.
