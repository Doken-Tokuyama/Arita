# ADR-030 — E0214 vacuous len / is_empty theater

- **Estado:** **aceptada** + **IMPL verified** (Lex measure **72/72** accepted)
- **CUT-ID:** `TRAPS-LEN-THEATER-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (draft/GO) ← brief Ingeniero (corpus trampas #3)
- **Relacionados:** ADR-010 (E0210–E0213), ADR-006 (assert / std), ADR-026 (`len`/`is_empty`), `THREAT_MODEL.md`
- **Barra:** neg oracles rejected + código estable; skip ≠ PASS.

## Contexto

Corpus trampas fit hoy: **#3 vacuous len/is_empty theater**.  
#1 Mutex×await y #2 Result swallow = **OUT** hasta que exista surface (candidatos ADR futuros — ver §Futuro).

E0211 cubre `assert true` / tautologías sin evidencia. Ampliarlo a `len() >= 0` **diluye** el mensaje. Mejor código dedicado.

## Decisión

### 1. Nuevo código

| Código | English (canonical) | Cuándo |
|--------|---------------------|--------|
| **E0214** | `vacuous length assert` | assert que “prueba” longitud/vacuidad de forma vacua / siempre-cierta |

**No** reutilizar E0211.

### 2. Patrones neg v0 (IN)

Detectar en `Stmt::Assert` (y equivalentes) cuando ambos lados hacen la desigualdad **trivialmente verdadera** sobre `len` / `is_empty`:

1. `assert <recv>.len() >= 0` (Int length siempre ≥ 0)
2. `assert <recv>.len() > -1` / `>= 0` variantes literales no negativas obvias (pin IMPL: al menos `>= 0` y `> -1` si el AST lo permite)
3. Opcional v0 si trivial: `assert !<recv>.is_empty() || <recv>.is_empty()` (tautología booleana sobre el mismo recv)

`<recv>` = Path tipado `Vec<_>` | `String` (ADR-026).

### 3. OUT de este CUT

- Asserts con evidencia real: `assert v.len() == 2`, `assert s.len() > 0` con estado conocido ≠ vacuous
- Ampliar a otros métodos std sin CUT
- #1 Mutex×await, #2 Result swallow (sin surface aún)

### 4. Oráculos

| Path | Expect |
|------|--------|
| `ejemplos/f2/neg/e0214-len-ge-zero.arita` | **E0214** (`v.len() >= 0` o `s.len() >= 0`) |
| opc. `ejemplos/f2/neg/e0214-is-empty-taut.arita` | **E0214** si se implementa patrón 3 |

Wire `arita measure` NEG_ORACLES. Control positivo: asserts de longitud no vacuos siguen OK.

### 5. Gobernanza

- Addendum a familia E021x (ADR-010); este ADR = CUT dedicado.
- IMPL: executor Ingeniero (o lane syntax); Parser/Codegen HOLD salvo brief.
- Arquitecto: DOC only.

## Futuro (candidatos trampas — no este CUT)

| # | Tema | Nota |
|---|------|------|
| 1 | Mutex × await | Requiere surface sync/async locks — ADR futuro post-surface |
| 2 | Result swallow | Requiere `Result`/`?` / ignore-err en surface — ADR futuro |

## Checklist GO — cerrado

- [x] E0214 (no diluir E0211) OK
- [x] Patrones `len() >= 0` (+ opcional tautología `is_empty`) OK
- [x] CUT `TRAPS-LEN-THEATER-20260914` → **aceptada** + **IMPL GO**
- [x] Oráculos neg + measure (Ingeniero) — `e0214-len-ge-zero` / `e0214-len-eq-len` / `e0214-is-empty-taut` → NEG_ORACLES
