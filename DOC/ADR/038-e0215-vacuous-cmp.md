# ADR-038 — E0215 vacuous comparison assert (trampa portable)

- **Estado:** **aceptada** + **IMPL verified** (Lex measure **79 accepted** + **2 gated**; `STABLE_VERIFY`)
- **CUT-ID:** `TRAPS-CMP-THEATER-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-010, ADR-030 / E0214, ADR-031, ROADMAP Corpus trampas, `TRAPS-CATALOG.md`
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Safe-only; sin deps nuevas.
- **Barra:** E2E measure neg oracles; skip ≠ PASS.

## Contexto

Corpus trampas Fase 4 aún abierto. Candidatos ROADMAP **#1 Mutex×await** y **#2 Result swallow** **no caben** en surface actual (no hay `Mutex` ni `Result`/`Err` en F2/async v0). Quedan **park** hasta surface+CUT.

Trampa portable **ahora**: asserts de comparación **reflexiva / vacua** sobre Int (o Bool identidad), paralelo a E0214 sin diluir E0211.

## Decisión (pins GO)

### 1. Código

| Código | Mensaje EN canónico |
|--------|---------------------|
| **E0215** | `vacuous comparison assert` |

No reusar E0211 (`assert true`) ni E0214 (`vacuous length assert`).

### 2. Patrones v0 (detectar)

1. `assert <expr> == <expr>` con ambos lados **sintácticamente idénticos** (Path/LitInt/LitBool).
2. `assert <expr> != <expr>` mismos lados idénticos (siempre false — theater o bug; **reject** igual).
3. `assert <expr> <= <expr>` / `>=` con lados idénticos (reflexivo siempre true).

OUT v0: algebra compleja, conmutatividad (`a+b == b+a`), side-effecting exprs.

### 3. Oráculos

| Path (sugerido) | Expect |
|-----------------|--------|
| `ejemplos/f2/neg/e0215-eq-self.arita` | E0215 (`assert n == n` o `assert 1 == 1`) |
| `ejemplos/f2/neg/e0215-le-self.arita` | E0215 (`assert n <= n`) |

1–2 neg bastan; measure wire.

### 4. OUT / park

| Candidato | Estado |
|-----------|--------|
| Mutex×await theater | **PARK** — sin Mutex en surface |
| Result swallow / ignore-Err | **→ ADR-048 / E0272** (`TRAPS-RESULT-SWALLOW-20260918`) — surface Result vía ADR-047 |
| Self-host / unsafe | OUT |

## Checklist

- [x] E0215 + mensajes OK
- [x] #1/#2 park justificado
- [x] GO DOC+IMPL
- [x] Neg + measure (Ingeniero) — `e0215-eq-self` / `e0215-le-self` → NEG_ORACLES

HOLD crates → review-only post verify.
