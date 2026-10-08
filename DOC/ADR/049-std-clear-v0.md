# ADR-049 — Std clear v0 (unpark ADR-036)

- **Estado:** **aceptada** + **verified** Lex **99/99** (CUT `STD-CLEAR-20260918`)
- **CUT-ID:** `STD-CLEAR-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL post-048)
- **Relacionados:** ADR-036 (PARK hot-path → este CUT unpark `clear`); ADR-026 (whitelist `len`/`is_empty`); ADR-006; ADR-022 safe-only; ADR-048 (cola previa)
- **Gobernanza:** <person> «agentes NO idle» / avanzar sin parar. Parser/Codegen: prep/review OK; land crates solo con Ingeniero post-GO IMPL. **Mutex PARK** (no inventar).
- **Barra:** E2E measure compile+run; skip ≠ PASS; sin umbrales timed.

## Contexto

ADR-036 aparcó hot-path std por falta de API nueva segura medible. Candidato documentado: **`clear` + `len()==0`** (no wall-clock). Surface Result (047) + swallow (048) liberan la cola → este CUT.

## Decisión (pins)

### 1. IN v0

| Surface | Tipo | Semántica | Emit (orientativo) |
|---------|------|-----------|--------------------|
| `v.clear()` | `Vec<T>` **mut** | vacía el vec; postcondición `v.len()==0` / `v.is_empty()==true` | `Vec::clear` |
| `s.clear()` | `String` **mut** | vacía el string (bytes); post `s.len()==0` / `s.is_empty()==true` | `String::clear` |

Pins duros:

1. **Arity 0** — args → **E0203** (nombre en whitelist) o E0206 si se prefiere consistencia 026; pin: **E0203** arity, **E0206** nombre fuera.
2. **Receiver mut** — `clear` exige `&mut self` en emit; binding **`mut`**. Sin `mut` → error borrow existente (**E0202** / pin borrow checker) — no inventar código nuevo si E0202 ya cubre; si HIR no distingue, Ingeniero reporta HOLE y pedimos E0xxx aparte.
3. **Retorno** `()` — no encadenar valor.
4. **Whitelist ADR-026 ampliada:** añadir `clear` a String|Vec. Todo lo demás sigue E0206.
5. **Safe-only** — emit sin `unsafe`; forbid usuario.
6. **Borrow:** `clear` = mut borrow exclusivo del receiver; no move out.


### 1b. HOLE prep Parser (2026-09-18) — pin IMPL

SoT review `DOC/reviews/STD-CLEAR-PREP-NOTES.md` (Parser):

- Pest: `.clear()` ya parsea (`method_call_other`, arity 0) — **sin** cambio gramática.
- HIR hoy: whitelist solo `len|is_empty|push` → `.clear` = **E0206** (unit OK).
- **HOLE:** métodos exclusivos (`push`, y `clear` al unpark) **no** verifican aún `binding.mutable`. `push` puede colar sin `mut`.
- **Pin IMPL (este CUT):** al whitelistar `clear`, gate `binding.mutable` (y alinear `push` si el mismo path) → **E0202** si falta `mut`. No inventar E0xxx nuevo salvo que E0202 no encaje; reportar entonces.
- Codegen: emit `{recv}.clear()` sin move; oráculos measure = post `len==0` (no timed).

### 2. OUT v0

- `pop` / `insert` / `remove` / `retain` / `truncate` / `drain`
- Index `[]` / slice / `get` / `chars`
- `clear` sobre tipos ≠ String|Vec
- Mutex / locks / async clear
- Umbrales timed / “hot-path” perf theater (sigue ADR-028 profiles)

### 3. Diagnósticos

| Código | Uso |
|--------|-----|
| **E0206** | método fuera whitelist (p.ej. `.pop()` sigue E0206) |
| **E0203** | arity/`clear` mal tipado si aplica |
| E0202 (existente) | clear sin `mut` / borrow conflict — reutilizar si ya gatea |

Sin código E02xx nuevo salvo HOLE reportado post-sondeo.

### 4. Oráculos measure

| Id | Expect |
|----|--------|
| `std-clear-vec` | `mut v: Vec<Int>`; push; `v.clear()`; `assert v.len() == 0` (o print `0`) → **accepted** |
| `std-clear-string` | `mut s: String`; clear; `assert s.len() == 0` / `s.is_empty()` → **accepted** |
| `neg-e0206-pop` (o reusar neg E0206) | `.pop()` u otro OUT → **E0206** |

Paths: `ejemplos/f2/` (o `ejemplos/std/`) — Ingeniero fija al cablear. Fixture mínima sugerida:

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

### 5. Relación ADR-036 / 026

- ADR-036: **unpark parcial** solo para `clear` (este ADR). El resto del PARK (index/pop/…) sigue.
- ADR-026: tabla whitelist **+** filas `v.clear()` / `s.clear()` → `()`.

## Checklist

- [x] Pins IN/OUT + oráculos + CUT-ID
- [x] GO DOC (<person> paralelo a E0272)
- [x] IMPL GO — E0272 **96/96** cerrado; Ingeniero sole stack
- [x] Landed + measure Lex **99/99** (STABLE_VERIFY)
- [ ] Gate `binding.mutable` para `clear` (+ alinear `push`) → E0202 (HOLE Parser)
- [ ] Measure Lex post-IMPL

## Cola

1. ADR-048 E0272 measure verde  
2. **Este CUT IMPL** (`STD-CLEAR-20260918`)  
3. Mutex sigue **PARK** (surface aparte)
