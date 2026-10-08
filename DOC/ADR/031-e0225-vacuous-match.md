# ADR-031 — E0225 vacuous match arms (anti-theater)

- **Estado:** **aceptada** + **IMPL verified** (Lex measure **75/75** accepted; **STABLE_VERIFY**)
- **CUT-ID:** `TRAPS-MATCH-VACUOUS-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (draft/GO) ← brief Ingeniero (hueco confirmado)
- **Relacionados:** ADR-015 (match stmt), ADR-025 (match-as-expr), ADR-030 (E0214 vacuous asserts), `THREAT_MODEL.md`
- **Barra:** neg rejected + código estable; skip ≠ PASS.

## Contexto

Hueco verificado: `match` Bool con brazos **idénticos** (`true => { 1 }`, `false => { 1 }`) **build OK** hoy — E0221–E0223 no lo pilla (exhaustivo pero theater).

Surface real (ADR-025): brazos con bloque `{ … }`, no bare lit obligatorio.

## Decisión

### 1. Código

| Código | English (canonical) | Cuándo |
|--------|---------------------|--------|
| **E0225** | `vacuous match arms` | todos los brazos del `match` producen el **mismo valor constante** (morph / theater) |

No diluir E0221–E0223 (exhaustividad / tipos).

### 2. Detección v0 (IN)

Aplicar a `Expr::Match` / stmt match (015+025):

1. **Bool:** brazos `true` y `false` (sin `_`, o con `_` irrelevante) cuyos bodies, tras strip de bloque `{ … }`, son la **misma constante** (`LitInt` / `LitBool` / `LitStr` v0).
2. **Int:** todos los brazos presentes (literales + `_` si hay) la **misma constante**.
3. Comparación: igualdad estructural del valor constante del **tail** del bloque (última expr) o body expr; side-effects (`print`) en el bloque = **OUT de detección v0** (no marcar E0225 si hay stmt no-tail distinto entre brazos — pin: v0 solo cuando cada brazo es `{ <lit> }` o lit puro).

### 3. Neg mínimo (Ingeniero)

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

+ Int morph: todos los brazos (incl `_`) misma constante → `ejemplos/f2.2/neg/e0225-match-int-same.arita` (nombre fijable en IMPL).

### 4. OUT

- Brazos con misma constante pero **stmts** distintos con efecto (`print`) — defer CUT
- Análisis de equivalencia semántica profunda / CSE
- Cambiar E0221–E0224

### 5. Oráculos

| Path | Expect |
|------|--------|
| `ejemplos/f2.2/neg/e0225-match-bool-same.arita` | **E0225** |
| `ejemplos/f2.2/neg/e0225-match-int-same.arita` | **E0225** |

Positivos match-as-expr (`f22-04`/`f22-05`) siguen verdes.

### 6. Gobernanza

- IMPL: executor Ingeniero (stack completo). Parser/Codegen HOLD review.
- Amplía familia E022x match; mensaje EN canónico arriba.

## Checklist GO — cerrado

- [x] E0225 `vacuous match arms` OK
- [x] Surface bloques `{ lit }` OK
- [x] Bool + Int morph OK
- [x] CUT `TRAPS-MATCH-VACUOUS-20260914` → **aceptada** + **IMPL GO**
- [x] Neg + measure (Ingeniero) — `e0225-match-bool-same` / `e0225-match-int-same` → NEG_ORACLES; Lex measure **75/75**
