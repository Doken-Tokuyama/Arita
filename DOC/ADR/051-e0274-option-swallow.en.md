Translation of `051-e0274-option-swallow.md`; the original is normative. / Traducción de `051-e0274-option-swallow.md`; el original es el normativo.

# ADR-051 — E0274 Option swallow / ignore-None

- **Estado:** **aceptada** + **verified** Lex **105/105**
- **CUT-ID:** `TRAPS-OPTION-SWALLOW-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL)
- **Relacionados:** ADR-050 Option v0 (OUT swallow → este CUT); ADR-048 E0272 (espejo Result); ADR-022 safe-only
- **Gobernanza:** <person> sin parar; Mutex **PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** HIR reject antes de emit; measure neg → **E0274**; skip ≠ PASS.

## Context

ADR-050 landed Lex **102/102**. Surface `Option` + exhaustive match (`E0273`). Next: anti-theater **Option swallow / ignore-None** (mirror ADR-048). Do not invent `?`/`unwrap`/Mutex.

## Decision (GO pins)

### 1. Code

| Code | EN message (canonical) | When |
|--------|------------------------|--------|
| **E0274** | `option none swallowed` | brazo `None` de un `match` sobre `Option` es **success-theater** (ver §2) |

Family Option: E0273 exhaustiveness; E0274 swallow. Do not collide with E0270–E0272 (Result).

### 2. IN v0 — patterns (HIR)

Sobre `match <expr: Option<…>> { Some(…) => … None => … }`:

**Swallow** si el brazo `None` es **success-theater**:
- empty block `{ }`
- `print(<lit>)` / `assert` without absence evidence
- **lit/path default** (e.g. `{ 0 }`) as if there were a value

(`None` no trae binding; el criterion es success theater, no “dead binding”.)

### 3. OUT v0

- `?` / `unwrap` / `expect` / `ok_or`
- Result swallow (still E0272)
- Mutex
- `None` arm that **fails for real** (e.g. `print` of agreed absence message in pos, or surface divergence if it exists) — pos pin: **explicit** None path no-teatro
- Solo falta brazo None → **E0273**
- If Some+None same constant → prefer **E0274** when scrutinee is Option (analogous E0272 vs E0225)

### 4. Canonical fixture (neg)

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

Optional morph: `None => { }` (if it types in stmt context) or `None => { print("ok") }` → E0274.

### 5. Oracles measure

| Id | Expect |
|----|--------|
| `neg-e0274` | fixture §4 → **E0274** |
| `neg-e0274-print-ok` (opt.) | `None => { print("ok") }` → **E0274** |
| `option-swallow-ok` (pos) | `None => { print("none") }` (o message de ausencia no-teatro) → **accepted** |

Paths: Ingeniero fixes under `ejemplos/`.

### 6. Emit / anti-theater

- Gate HIR (as E0272); not rustc theater.
- Emit rechaza; Clippy intacto.

## Checklist

- [x] Pins E0274 + IN/OUT + fixture
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **105/105** (STABLE_VERIFY)

## Queue

Post measure: Mutex **PARK** (surface aparte). Do not invent.
