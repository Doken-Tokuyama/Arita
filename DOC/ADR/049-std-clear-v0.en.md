Translation of `049-std-clear-v0.md`; the original is normative. / Traducción de `049-std-clear-v0.md`; el original es el normativo.

# ADR-049 — Std clear v0 (unpark ADR-036)

- **Estado:** **aceptada** + **verified** Lex **99/99** (CUT `STD-CLEAR-20260918`)
- **CUT-ID:** `STD-CLEAR-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL post-048)
- **Relacionados:** ADR-036 (PARK hot-path → este CUT unpark `clear`); ADR-026 (whitelist `len`/`is_empty`); ADR-006; ADR-022 safe-only; ADR-048 (cola previa)
- **Gobernanza:** <person> «agentes NO idle» / avanzar sin parar. Parser/Codegen: prep/review OK; land crates solo con Ingeniero post-GO IMPL. **Mutex PARK** (no inventar).
- **Barra:** E2E measure compile+run; skip ≠ PASS; sin umbrales timed.

## Context

ADR-036 parked hot-path std por falta de API nueva segura measurable. Candidato documentado: **`clear` + `len()==0`** (no wall-clock). Surface Result (047) + swallow (048) liberan la cola → this CUT.

## Decision (pins)

### 1. IN v0

| Surface | Tipo | Semantics | Emit (orientativo) |
|---------|------|-----------|--------------------|
| `v.clear()` | `Vec<T>` **mut** | empties the vec; postcondition `v.len()==0` / `v.is_empty()==true` | `Vec::clear` |
| `s.clear()` | `String` **mut** | empties the string (bytes); post `s.len()==0` / `s.is_empty()==true` | `String::clear` |

Pins duros:

1. **Arity 0** — args → **E0203** (nombre en whitelist) o E0206 si se prefiere consistencia 026; pin: **E0203** arity, **E0206** nombre fuera.
2. **Receiver mut** — `clear` requires `&mut self` in emit; **`mut`** binding. Without `mut` → existing borrow error (**E0202** / borrow-checker pin) — do not invent a new code if E0202 already covers it; if HIR does not distinguish, Ingeniero reports HOLE and we ask for a separate E0xxx.
3. **Return** `()` — do not chain a value.
4. **Whitelist ADR-026 expanded:** add `clear` to String|Vec. Everything else stays E0206.
5. **Safe-only** — emit without `unsafe`; forbid on user.
6. **Borrow:** `clear` = mut borrow exclusivo del receiver; no move out.


### 1b. HOLE prep Parser (2026-09-18) — pin IMPL

SoT review `DOC/reviews/STD-CLEAR-PREP-NOTES.md` (Parser):

- Pest: `.clear()` already parses (`method_call_other`, arity 0) — **no** grammar change.
- HIR today: whitelist solo `len|is_empty|push` → `.clear` = **E0206** (unit OK).
- **HOLE:** exclusive methods (`push`, and `clear` on unpark) **do not** yet check `binding.mutable`. `push` can slip through without `mut`.
- **Pin IMPL (this CUT):** when whitelisting `clear`, gate `binding.mutable` (and align `push` if same path) → **E0202** if `mut` missing. Do not invent a new E0xxx unless E0202 does not fit; report then.
- Codegen: emit `{recv}.clear()` without move; oracles measure = post `len==0` (no timed).

### 2. OUT v0

- `pop` / `insert` / `remove` / `retain` / `truncate` / `drain`
- Index `[]` / slice / `get` / `chars`
- `clear` sobre tipos ≠ String|Vec
- Mutex / locks / async clear
- Umbrales timed / “hot-path” perf theater (still ADR-028 profiles)

### 3. Diagnostics

| Code | Use |
|--------|-----|
| **E0206** | method off whitelist (e.g. `.pop()` still E0206) |
| **E0203** | arity/`clear` mal tipado si aplica |
| E0202 (existente) | clear without `mut` / borrow conflict — reuse if already gated |

No new E02xx code unless HOLE reported after probe.

### 4. Oracles measure

| Id | Expect |
|----|--------|
| `std-clear-vec` | `mut v: Vec<Int>`; push; `v.clear()`; `assert v.len() == 0` (o print `0`) → **accepted** |
| `std-clear-string` | `mut s: String`; clear; `assert s.len() == 0` / `s.is_empty()` → **accepted** |
| `neg-e0206-pop` (o reusar neg E0206) | `.pop()` u otro OUT → **E0206** |

Paths: `ejemplos/f2/` (o `ejemplos/std/`) — Ingeniero fija al cablear. Fixture minimal suggested:

```arita
module std_clear_vec
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  v.clear()
  print(v.len())
}
```
Expect stdout `0`.

### 5. Relation ADR-036 / 026

- ADR-036: **unpark parcial** only for `clear` (this ADR). El resto del PARK (index/pop/…) still.
- ADR-026: tabla whitelist **+** filas `v.clear()` / `s.clear()` → `()`.

## Checklist

- [x] Pins IN/OUT + oracles + CUT-ID
- [x] GO DOC (<person> paralelo a E0272)
- [x] IMPL GO — E0272 **96/96** cerrado; Ingeniero sole stack
- [x] Landed + measure Lex **99/99** (STABLE_VERIFY)
- [ ] Gate `binding.mutable` for `clear` (+ align `push`) → E0202 (HOLE Parser)
- [ ] Measure Lex post-IMPL

## Queue

1. ADR-048 E0272 measure verde  
2. **This CUT IMPL** (`STD-CLEAR-20260918`)  
3. Mutex still **PARK** (surface aparte)
