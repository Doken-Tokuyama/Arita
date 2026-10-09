Translation of `030-e0214-vacuous-len.md`; the original is normative. / Traducción de `030-e0214-vacuous-len.md`; el original es el normativo.

# ADR-030 — E0214 vacuous len / is_empty theater

- **Estado:** **aceptada** + **IMPL verified** (Lex measure **72/72** accepted)
- **CUT-ID:** `TRAPS-LEN-THEATER-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (draft/GO) ← brief Ingeniero (corpus trampas #3)
- **Relacionados:** ADR-010 (E0210–E0213), ADR-006 (assert / std), ADR-026 (`len`/`is_empty`), `THREAT_MODEL.md`
- **Barra:** neg oracles rejected + código estable; skip ≠ PASS.

## Context

Traps corpus fit hoy: **#3 vacuous len/is_empty theater**.  
#1 Mutex×await y #2 Result swallow = **OUT** until que exista surface (candidatos ADR futuros — ver §Future).

E0211 covers `assert true` / tautologies without evidence. Extending it to `len() >= 0` **diluye** el mensaje. Better a dedicated code.

## Decision

### 1. New code

| Code | English (canonical) | When |
|--------|---------------------|--------|
| **E0214** | `vacuous length assert` | assert que “prueba” longitud/vacuidad de forma vacua / always-cierta |

**No** reuse E0211.

### 2. Patterns neg v0 (IN)

Detectar en `Stmt::Assert` (y equivalentes) when both lados hacen la desigualdad **trivialmente verdadera** sobre `len` / `is_empty`:

1. `assert <recv>.len() >= 0` (Int length always ≥ 0)
2. `assert <recv>.len() > -1` / `>= 0` variantes literales no negativas obvias (pin IMPL: al menos `>= 0` y `> -1` si el AST lo permite)
3. Optional v0 if trivial: `assert !<recv>.is_empty() || <recv>.is_empty()` (tautology booleana sobre el same recv)

`<recv>` = Path tipado `Vec<_>` | `String` (ADR-026).

### 3. OUT de this CUT

- Asserts with evidence real: `assert v.len() == 2`, `assert s.len() > 0` with estado conocido ≠ vacuous
- Widen to other std methods without CUT
- #1 Mutex×await, #2 Result swallow (still without surface)

### 4. Oracles

| Path | Expect |
|------|--------|
| `ejemplos/f2/neg/e0214-len-ge-zero.arita` | **E0214** (`v.len() >= 0` o `s.len() >= 0`) |
| opc. `ejemplos/f2/neg/e0214-is-empty-taut.arita` | **E0214** si se implementa pattern 3 |

Wire `arita measure` NEG_ORACLES. Control positivo: asserts de longitud no vacuos siguen OK.

### 5. Governance

- Addendum to family E021x (ADR-010); this ADR = dedicated CUT.
- IMPL: executor Ingeniero (o lane syntax); Parser/Codegen HOLD salvo brief.
- Arquitecto: DOC only.

## Future (candidatos traps — no this CUT)

| # | Tema | Nota |
|---|------|------|
| 1 | Mutex × await | Requiere surface sync/async locks — ADR future post-surface |
| 2 | Result swallow | Requiere `Result`/`?` / ignore-err en surface — ADR future |

## Checklist GO — cerrado

- [x] E0214 (no diluir E0211) OK
- [x] Patterns `len() >= 0` (+ optional tautology `is_empty`) OK
- [x] CUT `TRAPS-LEN-THEATER-20260914` → **accepted** + **IMPL GO**
- [x] Oracles neg + measure (Ingeniero) — `e0214-len-ge-zero` / `e0214-len-eq-len` / `e0214-is-empty-taut` → NEG_ORACLES
