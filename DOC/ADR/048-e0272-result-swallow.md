# ADR-048 — E0272 Result swallow / ignore-Err

- **Estado:** **aceptada** + **verified** Lex **96/96** (pins cerrados post sondeo Lex)
- **CUT-ID:** `TRAPS-RESULT-SWALLOW-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (sondeo + IMPL)
- **Relacionados:** ADR-047 Result v0 (OUT swallow → este CUT); ADR-038 (PARK histórico); ADR-010 familia anti-theater; ADR-031 E0225 (vacuous match — distinto)
- **Gobernanza:** <person> «sigue» post-PAUSE; cola **1 swallow → STD-CLEAR → Mutex**. Parser/Codegen HOLD review-only; Ingeniero sole IMPL stack.
- **Barra:** HIR reject **antes** de emit; measure neg must fail with **E0272**; skip ≠ PASS; safe-only ADR-022.
- **Sondeo Lex:** HOLE **confirmado** — fixture abajo build+run **accepted**, stdout `0`, sin E0xxx hoy. Morph `Err(_) => { 0 }` igual verde.

## Contexto

ADR-047 landed Lex **92/92**. Surface ya tiene `Result<T,E>` + `match` exhaustivo (`E0270`). Quedaba el CUT anti-theater **Result swallow / ignore-Err**. Sin inventar surface (`?`/`unwrap`/`fn → Result` siguen OUT v0).

## Decisión (pins GO) — cerrados

### 1. Código

| Código | Mensaje EN (canónico) | Cuándo |
|--------|------------------------|--------|
| **E0272** | `result error swallowed` | brazo `Err` de un `match` sobre `Result` **traga** el error (ver §2) |

- **E0272** (no E0228): familia Result E027x junto a E0270.
- No reutilizar E0270 (exhaustividad) ni E0203 (type mismatch) ni E0225 (todos brazos misma constante).

### 2. IN v0 — patrones (HIR, sintáctico)

Sobre `match <expr: Result<…>> { Ok(…) => … Err(…) => … }`:

**Swallow** si el brazo `Err` cumple **todas**:

1. Payload de error **no se usa**: `Err(_)` **o** `Err(e)` / `Err(name)` con binding **muerto** (no aparece en el cuerpo).
2. Cuerpo = **éxito-teatro**: bloque vacío, `print(<lit>)`, `assert` sin leer Err, **o lit/path default** (p.ej. `{ 0 }`) independiente del error.

### 3. OUT v0

- `?` / `unwrap` / `expect` / `map_err`
- Mutex / Option swallow
- `fn … -> Result<…>` (sigue OUT ADR-047; oráculos usan `let r: Result<…> = …`)
- Oráculos en `test { … }` si surface test limita (`E0007` en sondeo) — **usar `fn main() -> Io<()>`**
- Brazo `Err(e)` que **usa** `e` (p.ej. `print(e)`)
- Solo falta brazo Err → **E0270**
- Si Ok+Err misma constante **y** Err unused → preferir **E0272** (scrutinee Result)

### 4. Fixture canónica (neg) — SoT sondeo

```arita
module result_err_swallow_neg
fn main() -> Io<()> {
  let r: Result<Int, Int> = Err(1)
  let v: Int = match r {
    Ok(x) => { x }
    Err(e) => { 0 }
  }
  print(v)
}
```

Expect: **E0272** `result error swallowed` (hoy: accepted + stdout `0` = theater).

Morph opcional cableado: `Err(_) => { 0 }` → mismo E0272.

### 5. Oráculos measure

| Id | Expect |
|----|--------|
| `neg-e0272` | fixture canónica (§4) → **E0272** |
| `neg-e0272-underscore` (opc.) | morph `Err(_) => { 0 }` → **E0272** |
| `result-swallow-ok` (pos) | `Err(e)` + uso real de `e` (p.ej. `print(e)` o valor derivado de `e`) → **accepted** |

Paths: Ingeniero fija bajo `ejemplos/` al cablear; ids = contrato.

### 6. Emit / anti-theater

- Gate **HIR check** (como E0216/E0217); no rustc/panic theater.
- Emit **rechaza**; no “arreglar” swallow.
- Clippy workspace intacto.

## Checklist

- [x] HOLE Lex confirmado (Ingeniero)
- [x] Pins E0272 + mensaje + IN/OUT + fixture
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **96/96** — baseline **92/92**; expect **~94–95** (+ neg ± morph ± pos)

## Cola

Post measure verde: **STD-CLEAR** → **Mutex** PARK hasta surface.
