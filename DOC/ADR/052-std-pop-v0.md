# ADR-052 — Vec.pop → Option v0 (unpark parcial ADR-036)

- **Estado:** **aceptada** + **verified** Lex **108/108**
- **CUT-ID:** `STD-POP-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-036 (park hot-path; `clear`→049; este CUT unpark **`pop`**); ADR-026 whitelist; ADR-049 clear; ADR-050 Option; ADR-022 safe-only
- **Gobernanza:** <person> sin idle; **Mutex PARK** (no inventar). Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS; sin timed thresholds.

## Contexto

Cola post E0274 **105/105**. Mutex/Origin PARK. `Option` ya en surface → desbloquea **`Vec.pop() -> Option<T>`** (candidato ADR-036, aparcado hasta Option). No inventa Mutex.

## Decisión (pins GO)

### 1. IN v0

| Surface | Regla | Emit |
|---------|--------|------|
| `v.pop()` | `v: mut Vec<T>`; retorno **`Option<T>`** | `Vec::pop` |

Pins:

1. **Arity 0** — args → E0203 (nombre whitelist) / E0206 si nombre fuera.
2. **Receiver mut** — gate `binding.mutable` → **E0202** (igual clear/push §1b ADR-049).
3. Semántica: vacío → `None`; si no → `Some(último)` + remueve último.
4. **T** ∈ tipos Vec ya permitidos F2 (Int/Bool/String como push).
5. Whitelist ADR-026/049 **+** `pop`. Safe-only emit.
6. Encadenar/match sobre el `Option` resultante = surface 050 (E0273/E0274 aplican).

### 2. OUT v0

- `insert` / `remove` / `truncate` / index `[]` / `get` / `swap_remove`
- `String.pop` (chars) — **OUT** v0 (solo Vec)
- Mutex / timed hot-path theater
- `pop` sobre no-Vec → E0206

### 3. Oráculos measure

| Id | Expect |
|----|--------|
| `std-pop-some` | push ≥1; `pop` → `Some`; print payload canónico → **accepted** |
| `std-pop-none` | `Vec::new()`; `pop` → `None`; match/`print("none")` → **accepted** |
| `neg-e0202-pop-not-mut` (opc.) | `pop` sin `mut` → **E0202** |
| reusar `neg-e0206` | `.insert`/index sigue E0206 |

Fixture mínima sugerida:

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

### 4. Relación ADR-036 / 026

- ADR-036: unpark **`pop`** (este ADR); insert/index siguen PARK.
- Whitelist: `pop` añadido junto a `clear`/`push`/`len`/`is_empty`.

## Checklist

- [x] Pins IN/OUT + oráculos + CUT-ID
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **108/108** (STABLE_VERIFY)

## Cola

Post measure: Mutex **PARK**. insert/index siguen PARK.
