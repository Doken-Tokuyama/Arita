Translation of `052-std-pop-v0.md`; the original is normative. / Traducción de `052-std-pop-v0.md`; el original es el normativo.

# ADR-052 — Vec.pop → Option v0 (unpark parcial ADR-036)

- **Estado:** **aceptada** + **verified** Lex **108/108**
- **CUT-ID:** `STD-POP-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-036 (park hot-path; `clear`→049; este CUT unpark **`pop`**); ADR-026 whitelist; ADR-049 clear; ADR-050 Option; ADR-022 safe-only
- **Gobernanza:** <person> sin idle; **Mutex PARK** (no inventar). Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS; sin timed thresholds.

## Context

Queue after E0274 **105/105**. Mutex/Origin PARK. `Option` already on surface → unlocks **`Vec.pop() -> Option<T>`** (candidate ADR-036, parked until Option). Does not invent Mutex.

## Decision (GO pins)

### 1. IN v0

| Surface | Regla | Emit |
|---------|--------|------|
| `v.pop()` | `v: mut Vec<T>`; return **`Option<T>`** | `Vec::pop` |

Pins:

1. **Arity 0** — args → E0203 (nombre whitelist) / E0206 si nombre fuera.
2. **Receiver mut** — gate `binding.mutable` → **E0202** (igual clear/push §1b ADR-049).
3. Semantics: empty → `None`; else → `Some(last)` + removes last.
4. **T** ∈ already-allowed Vec types F2 (Int/Bool/String like push).
5. Whitelist ADR-026/049 **+** `pop`. Safe-only emit.
6. Encadenar/match sobre el `Option` resultante = surface 050 (E0273/E0274 aplican).

### 2. OUT v0

- `insert` / `remove` / `truncate` / index `[]` / `get` / `swap_remove`
- `String.pop` (chars) — **OUT** v0 (solo Vec)
- Mutex / timed hot-path theater
- `pop` sobre no-Vec → E0206

### 3. Oracles measure

| Id | Expect |
|----|--------|
| `std-pop-some` | push ≥1; `pop` → `Some`; canonical print payload → **accepted** |
| `std-pop-none` | `Vec::new()`; `pop` → `None`; match/`print("none")` → **accepted** |
| `neg-e0202-pop-not-mut` (opt.) | `pop` without `mut` → **E0202** |
| reusar `neg-e0206` | `.insert`/index still E0206 |

Fixture minimal suggested:

```arita
module std_pop_none
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  let o: Option<Int> = v.pop()
  match o {
    Some(x) => { print(x) }
    None => { print("none") }
  }
}
```

### 4. Relation ADR-036 / 026

- ADR-036: unpark **`pop`** (this ADR); insert/index remain PARK.
- Whitelist: `pop` added alongside `clear`/`push`/`len`/`is_empty`.

## Checklist

- [x] Pins IN/OUT + oracles + CUT-ID
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **108/108** (STABLE_VERIFY)

## Queue

Post measure: Mutex **PARK**. insert/index remain PARK.
