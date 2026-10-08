# ADR-039 — E0242 borrow held across await

- **Estado:** **aceptada** + **verified** (Lex measure **80 accepted** + **2 gated**; CUT `TRAPS-BORROW-AWAIT-20260915`)
- **CUT-ID:** `TRAPS-BORROW-AWAIT-20260915`
- **Fecha:** 2026-09-15
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL; sondeo HOLE)
- **Relacionados:** ADR-009 (borrow), ADR-027 (async), ADR-022 safe-only, ADR-038 verified
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. No sobrecargar E0202.
- **Barra:** E2E measure neg; skip ≠ PASS; rechazo **antes** de rustc (no depender de E0100).

## Contexto (sondeo)

`borrow_across_await` = **CLOSED** (was HOLE; now HIR E0242):

- HIR `parse_lower_check` OK con loan vivo en `await`
- Emit tipa mal (`&String` vs `i64`) → rustc E0100, **sin** diagnóstico ARITA
- Move across await OK; E0202 solo en push-vs-live-loan ordinario

## Decisión (pins GO)

### 1. Código

| Código | Mensaje EN canónico |
|--------|---------------------|
| **E0242** | `borrow held across await` |

**No** E0202 (ownership ordinario). **No** E0207. Familia async = E024x (junto a E0240/E0241).

### 2. Regla

Si existe un **préstamo vivo** (`&` / `&mut`) cuyo lifetime cruza un punto `await` en la misma fn → **E0242** en check (HIR), antes de emit/rustc.

OUT v0: análisis interprocedural profundo; `while false` tautology → **ADR-040** / E0226 (no este CUT).

### 3. Oráculo

Path: `ejemplos/async/neg/e0242-borrow-across-await.arita`

Fixture canónica (hole):

```arita
module probe_borrow_str_await
async fn tick() -> Io<()> { print("tick") }
async fn main() -> Io<()> {
  let s: String = "hi"
  let r: Int = &s
  await tick()
  print(r)
}
```

Expect: **E0242** (measure wire). Nota: el `Int = &s` es parte del theater/hole actual; el pin de este CUT es **borrow across await**, no redefinir E0203 en este ADR.

### 4. OUT

- Sobrecargar E0202
- Afirmar PASS vía rustc E0100
- `while false` → **ADR-040** / E0226
- Mutex×await / Result swallow (siguen PARK)

## Checklist

- [x] E0242 + EN + CUT OK
- [x] No E0202 overload OK
- [x] GO DOC+IMPL
- [x] Neg + measure (Ingeniero) — Lex **80** accepted + **2** gated; `neg-e0242-borrow-across-await`; reject in `parse_lower_check` (**E0242**, not rustc E0100)
