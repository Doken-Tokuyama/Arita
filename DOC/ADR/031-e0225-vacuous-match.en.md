Translation of `031-e0225-vacuous-match.md`; the original is normative. / Traducción de `031-e0225-vacuous-match.md`; el original es el normativo.

# ADR-031 — E0225 vacuous match arms (anti-theater)

- **Estado:** **aceptada** + **IMPL verified** (Lex measure **75/75** accepted; **STABLE_VERIFY**)
- **CUT-ID:** `TRAPS-MATCH-VACUOUS-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (draft/GO) ← brief Ingeniero (hueco confirmado)
- **Relacionados:** ADR-015 (match stmt), ADR-025 (match-as-expr), ADR-030 (E0214 vacuous asserts), `THREAT_MODEL.md`
- **Barra:** neg rejected + código estable; skip ≠ PASS.

## Context

Verified gap: `match` Bool with arms **identical** (`true => { 1 }`, `false => { 1 }`) **build OK** hoy — E0221–E0223 does not catch it (exhaustive pero theater).

Surface real (ADR-025): arms with bloque `{ … }`, no bare lit mandatory.

## Decision

### 1. Code

| Code | English (canonical) | When |
|--------|---------------------|--------|
| **E0225** | `vacuous match arms` | todos los arms of the `match` producen el **same valor constante** (morph / theater) |

No diluir E0221–E0223 (exhaustiveness / tipos).

### 2. Detection v0 (IN)

Aplicar a `Expr::Match` / stmt match (015+025):

1. **Bool:** arms `true` y `false` (without `_`, o with `_` irrelevant) whose bodies, after strip of block `{ … }`, son la **same constante** (`LitInt` / `LitBool` / `LitStr` v0).
2. **Int:** todos los arms presentes (literales + `_` si hay) la **same constante**.
3. Comparison: structural equality of the constant value of the block **tail** (last expr) or body expr; side-effects (`print`) in the block = **OUT of v0 detection** (do not mark E0225 if there is a non-tail stmt differing across arms — pin: v0 only when each arm is `{ <lit> }` or pure lit).

### 3. Neg minimum (Ingeniero)

```arita
module e0225_match_theater

fn main() -> Io<()> {
  let flag: Bool = true
  let x: Int = match flag {
    true => { 1 }
    false => { 1 }
  }
  print(x)
}
```

+ Int morph: todos los arms (incl `_`) same constante → `ejemplos/f2.2/neg/e0225-match-int-same.arita` (nombre fijable en IMPL).

### 4. OUT

- Brazos with same constante pero **stmts** distintos with efecto (`print`) — defer CUT
- Deep semantic equivalence analysis / CSE
- Cambiar E0221–E0224

### 5. Oracles

| Path | Expect |
|------|--------|
| `ejemplos/f2.2/neg/e0225-match-bool-same.arita` | **E0225** |
| `ejemplos/f2.2/neg/e0225-match-int-same.arita` | **E0225** |

Positivos match-as-expr (`f22-04`/`f22-05`) siguen verdes.

### 6. Governance

- IMPL: executor Ingeniero (full stack). Parser/Codegen HOLD review.
- Extends family E022x match; canonical EN message above.

## Checklist GO — cerrado

- [x] E0225 `vacuous match arms` OK
- [x] Surface bloques `{ lit }` OK
- [x] Bool + Int morph OK
- [x] CUT `TRAPS-MATCH-VACUOUS-20260914` → **accepted** + **IMPL GO**
- [x] Neg + measure (Ingeniero) — `e0225-match-bool-same` / `e0225-match-int-same` → NEG_ORACLES; Lex measure **75/75**
