Translation of `038-e0215-vacuous-cmp.md`; the original is normative. / Traducción de `038-e0215-vacuous-cmp.md`; el original es el normativo.

# ADR-038 — E0215 vacuous comparison assert (trap portable)

- **Estado:** **aceptada** + **IMPL verified** (Lex measure **79 accepted** + **2 gated**; `STABLE_VERIFY`)
- **CUT-ID:** `TRAPS-CMP-THEATER-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-010, ADR-030 / E0214, ADR-031, ROADMAP Corpus trampas, `TRAPS-CATALOG.md`
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Safe-only; sin deps nuevas.
- **Barra:** E2E measure neg oracles; skip ≠ PASS.

## Context

Traps corpus Phase 4 still abierto. Candidatos ROADMAP **#1 Mutex×await** y **#2 Result swallow** **no caben** en surface current (no hay `Mutex` ni `Result`/`Err` en F2/async v0). Quedan **park** until surface+CUT.

Portable trap **now**: comparison asserts **reflexive / vacuous** on Int (or Bool identity), parallel to E0214 without diluting E0211.

## Decision (pins GO)

### 1. Code

| Code | Message EN canonical |
|--------|---------------------|
| **E0215** | `vacuous comparison assert` |

No reuse E0211 (`assert true`) ni E0214 (`vacuous length assert`).

### 2. Patterns v0 (detectar)

1. `assert <expr> == <expr>` with both lados **syntactically identical** (Path/LitInt/LitBool).
2. `assert <expr> != <expr>` same identical sides (always false — theater o bug; **reject** the same).
3. `assert <expr> <= <expr>` / `>=` with identical sides (reflexive always true).

OUT v0: algebra compleja, conmutatividad (`a+b == b+a`), side-effecting exprs.

### 3. Oracles

| Path (sugerido) | Expect |
|-----------------|--------|
| `ejemplos/f2/neg/e0215-eq-self.arita` | E0215 (`assert n == n` o `assert 1 == 1`) |
| `ejemplos/f2/neg/e0215-le-self.arita` | E0215 (`assert n <= n`) |

1–2 neg bastan; measure wire.

### 4. OUT / park

| Candidato | Estado |
|-----------|--------|
| Mutex×await theater | **PARK** — no Mutex en surface |
| Result swallow / ignore-Err | **→ ADR-048 / E0272** (`TRAPS-RESULT-SWALLOW-20260918`) — surface Result via ADR-047 |
| Self-host / unsafe | OUT |

## Checklist

- [x] E0215 + mensajes OK
- [x] #1/#2 park justificado
- [x] GO DOC+IMPL
- [x] Neg + measure (Ingeniero) — `e0215-eq-self` / `e0215-le-self` → NEG_ORACLES

HOLD crates → review-only post verify.
