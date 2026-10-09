Translation of `048-e0272-result-swallow.md`; the original is normative. / Traducción de `048-e0272-result-swallow.md`; el original es el normativo.

# ADR-048 — E0272 Result swallow / ignore-Err

- **Estado:** **aceptada** + **verified** Lex **96/96** (pins cerrados post sondeo Lex)
- **CUT-ID:** `TRAPS-RESULT-SWALLOW-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (sondeo + IMPL)
- **Relacionados:** ADR-047 Result v0 (OUT swallow → este CUT); ADR-038 (PARK histórico); ADR-010 familia anti-theater; ADR-031 E0225 (vacuous match — distinto)
- **Gobernanza:** <person> «sigue» post-PAUSE; cola **1 swallow → STD-CLEAR → Mutex**. Parser/Codegen HOLD review-only; Ingeniero sole IMPL stack.
- **Barra:** HIR reject **antes** de emit; measure neg must fail with **E0272**; skip ≠ PASS; safe-only ADR-022.
- **Probe Lex:** HOLE **confirmed** — fixture below build+run **accepted**, stdout `0`, no E0xxx today. Morph `Err(_) => { 0 }` equally green.

## Context

ADR-047 landed Lex **92/92**. Surface already has `Result<T,E>` + exhaustive `match` (`E0270`). The anti-theater CUT remained **Result swallow / ignore-Err**. Do not invent surface (`?`/`unwrap`/`fn → Result` siguen OUT v0).

## Decision (pins GO) — cerrados

### 1. Code

| Code | EN message (canonical) | When |
|--------|------------------------|--------|
| **E0272** | `result error swallowed` | brazo `Err` de un `match` sobre `Result` **traga** el error (ver §2) |

- **E0272** (no E0228): family Result E027x junto a E0270.
- Do not reuse E0270 (exhaustiveness) nor E0203 (type mismatch) nor E0225 (all arms same constant).

### 2. IN v0 — patterns (HIR, syntactic)

Sobre `match <expr: Result<…>> { Ok(…) => … Err(…) => … }`:

**Swallow** si el brazo `Err` cumple **todas**:

1. Payload de error **no se usa**: `Err(_)` **o** `Err(e)` / `Err(name)` con binding **muerto** (no aparece en el cuerpo).
2. Cuerpo = **success-theater**: empty block, `print(<lit>)`, `assert` without reading Err, **o lit/path default** (e.g. `{ 0 }`) independent of the error.

### 3. OUT v0

- `?` / `unwrap` / `expect` / `map_err`
- Mutex / Option swallow
- `fn … -> Result<…>` (still OUT ADR-047; oracles usan `let r: Result<…> = …`)
- Oracles en `test { … }` si surface test limita (`E0007` en sondeo) — **usar `fn main() -> Io<()>`**
- Brazo `Err(e)` que **usa** `e` (e.g. `print(e)`)
- Solo falta brazo Err → **E0270**
- If Ok+Err same constant **and** Err unused → prefer **E0272** (scrutinee Result)

### 4. Canonical fixture (neg) — probe SoT

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

Expect: **E0272** `result error swallowed` (today: accepted + stdout `0` = theater).

Morph optional cableado: `Err(_) => { 0 }` → same E0272.

### 5. Oracles measure

| Id | Expect |
|----|--------|
| `neg-e0272` | canonical fixture (§4) → **E0272** |
| `neg-e0272-underscore` (opt.) | morph `Err(_) => { 0 }` → **E0272** |
| `result-swallow-ok` (pos) | `Err(e)` + uso real de `e` (e.g. `print(e)` o valor derivado de `e`) → **accepted** |

Paths: Ingeniero fixes under `ejemplos/` al cablear; ids = contrato.

### 6. Emit / anti-theater

- Gate **HIR check** (as E0216/E0217); not rustc/panic theater.
- Emit **rechaza**; no “arreglar” swallow.
- Clippy workspace intacto.

## Checklist

- [x] HOLE Lex confirmed (Ingeniero)
- [x] Pins E0272 + message + IN/OUT + fixture
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **96/96** — baseline **92/92**; expect **~94–95** (+ neg ± morph ± pos)

## Queue

Post measure verde: **STD-CLEAR** → **Mutex** PARK until surface.
