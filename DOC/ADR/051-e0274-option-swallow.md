# ADR-051 — E0274 Option swallow / ignore-None

- **Estado:** **aceptada** + **verified** Lex **105/105**
- **CUT-ID:** `TRAPS-OPTION-SWALLOW-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-050 Option v0 (OUT swallow → este CUT); ADR-048 E0272 (espejo Result); ADR-022 safe-only
- **Gobernanza:** <person> sin parar; Mutex **PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** HIR reject antes de emit; measure neg → **E0274**; skip ≠ PASS.

## Contexto

ADR-050 landed Lex **102/102**. Surface `Option` + match exhaustivo (`E0273`). Siguiente: anti-theater **Option swallow / ignore-None** (espejo ADR-048). Sin inventar `?`/`unwrap`/Mutex.

## Decisión (pins GO)

### 1. Código

| Código | Mensaje EN (canónico) | Cuándo |
|--------|------------------------|--------|
| **E0274** | `option none swallowed` | brazo `None` de un `match` sobre `Option` es **éxito-teatro** (ver §2) |

Familia Option: E0273 exhaustividad; E0274 swallow. No chocar E0270–E0272 (Result).

### 2. IN v0 — patrones (HIR)

Sobre `match <expr: Option<…>> { Some(…) => … None => … }`:

**Swallow** si el brazo `None` es **éxito-teatro**:
- bloque vacío `{ }`
- `print(<lit>)` / `assert` sin evidencia de ausencia
- **lit/path default** (p.ej. `{ 0 }`) como si hubiera valor

(`None` no trae binding; el criterio es teatro de éxito, no “binding muerto”.)

### 3. OUT v0

- `?` / `unwrap` / `expect` / `ok_or`
- Result swallow (sigue E0272)
- Mutex
- Brazo `None` que **falla de forma real** (p.ej. `print` de mensaje de ausencia acordado en pos, o divergencia surface si existe) — pin pos: camino None **explícito** no-teatro
- Solo falta brazo None → **E0273**
- Si Some+None misma constante → preferir **E0274** cuando scrutinee es Option (análogo E0272 vs E0225)

### 4. Fixture canónica (neg)

```arita
module option_none_swallow_neg
fn main() -> Io<()> {
  let o: Option<Int> = None
  let v: Int = match o {
    Some(x) => { x }
    None => { 0 }
  }
  print(v)
}
```

Expect: **E0274** `option none swallowed`.

Morph opc.: `None => { }` (si tipa en contexto stmt) o `None => { print("ok") }` → E0274.

### 5. Oráculos measure

| Id | Expect |
|----|--------|
| `neg-e0274` | fixture §4 → **E0274** |
| `neg-e0274-print-ok` (opc.) | `None => { print("ok") }` → **E0274** |
| `option-swallow-ok` (pos) | `None => { print("none") }` (o mensaje de ausencia no-teatro) → **accepted** |

Paths: Ingeniero fija bajo `ejemplos/`.

### 6. Emit / anti-theater

- Gate HIR (como E0272); no rustc theater.
- Emit rechaza; Clippy intacto.

## Checklist

- [x] Pins E0274 + IN/OUT + fixture
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **105/105** (STABLE_VERIFY)

## Cola

Post measure: Mutex **PARK** (surface aparte). Sin inventar.
