Translation of `010-e021x-negative-oracles.md`; the original is normative. / Traducción de `010-e021x-negative-oracles.md`; el original es el normativo.

# ADR-010 — Oracles negativos anti-theater E021x (F2)

- **Estado:** **aceptada**
- **CUT-ID:** `E021X-NEG-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO)
- **Relacionados:** ADR-006 §3–§4, ADR-007 (`Item::Test` / `Stmt::Assert`), ADR-030 (E0214), `THREAT_MODEL.md`, `ejemplos/f2/07-assert.arita`
- **Barra:** skip ≠ PASS; no inventar PASS; negativos deben **rejected** de forma estable

## Context

Positive Test+Assert verified (`arita test` PASS in `07-assert`). Anti-theater reverse: programs that **must fail** with canonical **E021x** codes (EN text), measured by oracle (CLI / tests), never silent.

## Decision (proposal CUT)

### 1. Codes IN (ADR-006, immovable without a new CUT)

| Code | English (canonical) | Caso negativo minimum |
|--------|---------------------|----------------------|
| **E0210** | `todo/unimplemented not allowed` | body / stmt with `todo!` / `unimplemented!` / keyword `todo` |
| **E0211** | `assert requires evidence` | `assert true` / `assert(true)` / assert tautological without evident sides |
| **E0212** | `empty test not allowed` | `test name { }` o only comentarios |
| **E0213** | `stub function body not allowed` | `fn` que exige valor with body empty |
| **E0214** | `vacuous length assert` | `len() >= 0` / `len()==len()` / `is_empty()||!is_empty()` — **ADR-030** / CUT `TRAPS-LEN-THEATER-20260914` |
| **E0215** | `vacuous comparison assert` | `assert n == n` / `n <= n` / lit==lit — **ADR-038** / CUT `TRAPS-CMP-THEATER-20260915` |
| **E0216** | `integer division by zero` | `/0` `%0` lit o call with divisor 0 — **ADR-044** / CUT `TRAPS-INT-DIV0-20260915` |
| **E0217** | `integer overflow` | `+/-/*` overflow i64 — **ADR-045** / CUT `TRAPS-INT-OVERFLOW-20260915` |

### 2. Oracle shape

- Corpus bajo `ejemplos/f2/neg/` (o suite dedicated) — **only** after E2E toolchain que emita E021x de verdad.
- Each case: parse/check/measure → **rejected** + stable code on stderr; **no** `accepted`; skip/timeout → `inconclusive` (never promote).
- Positivo de control: `07-assert` remains PASS via `arita test`.
- `arita measure` (o `arita test` harness) must list these negatives when IMPL GO exists.

### 3. Fuera de this CUT

- Ownership E0201/E0202 (ADR-009; necesita HIR + IMPL GO aparte).
- Ampliar surface F2.
- Declara “anti-theater done” only with lints Clippy of the workspace host.

### 4. Governance

- DOC GO closed; **IMPL authorized** (Ingeniero): diagnostics + `neg/` + measure.
- Arquitecto: DOC; **no** ownership E020x en this CUT.
- Nombre lenguaje = **ARITA**.

## Checklist GO (Ingeniero) — cerrado

- [x] Tabla E0210–E0213 + mensajes EN OK
- [x] E0214 addendum — ADR-030 (`TRAPS-LEN-THEATER-20260914`)
- [x] E0215 addendum — ADR-038 (`TRAPS-CMP-THEATER-20260915`)
- [x] E0216 addendum — ADR-044 (`TRAPS-INT-DIV0-20260915`)
- [x] E0217 addendum — ADR-045 (`TRAPS-INT-OVERFLOW-20260915`)
- [x] Negative corpus location OK (`ejemplos/f2/neg/`)
- [x] Regla skip ≠ PASS / inconclusive OK
- [x] GO `E021X-NEG-20260913` → **accepted**

## IMPL — cerrado (verificado)

- [x] Emitir **E0210–E0213** en parse/check (texto EN canonical) — `arita-syntax`.
- [x] Corpus `ejemplos/f2/neg/` with real **rejected** + code on stderr (`arita parse`).
- [x] Unit tests en `arita-syntax` cubren E0210–E0213.
- [x] `arita measure` neg-oracles wired: accepted iff parse fails with matching E021x; accidental PASS / wrong code → rejected; missing → inconclusive.
- **No** ownership E020x (remains ADR-009 / HIR).

## Verification (Ingeniero Rust, 2026-09-13)

- **E021x neg verified real** (CUT `E021X-NEG-20260913`).
- Corpus:
  - `ejemplos/f2/neg/e0210-todo.arita` → **E0210**
  - `ejemplos/f2/neg/e0211-assert-true.arita` → **E0211**
  - `ejemplos/f2/neg/e0212-empty-test.arita` → **E0212**
  - `ejemplos/f2/neg/e0213-empty-fn.arita` → **E0213**
- Control positivo: `ejemplos/f2/07-assert.arita` (`arita test` PASS).
- Ver `ejemplos/f2/neg/README.md`.

## Enlaces

- `DOC/ADR/030-e0214-vacuous-len.md` (E0214)
- `DOC/ADR/006-f2-ownership-surface.md`
- `ejemplos/f2/07-assert.arita`, `ejemplos/f2/neg/`
- `DOC/THREAT_MODEL.md`
- `ROADMAP.md`
