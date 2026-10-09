Translation of `026-std-minima.md`; the original is normative. / Traducción de `026-std-minima.md`; el original es el normativo.

# ADR-026 — Minimal std: `len` / `is_empty` (String + Vec)

- **Estado:** **aceptada** (DOC + IMPL CUT `STD-MIN-20260913`)
- **CUT-ID:** `STD-MIN-20260913`
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (draft/IMPL) · ARITA Arquitecto (**align** / pins)
- **Relacionados:** ADR-006 (whitelist F2), ADR-007 (MethodCall), ADR-009 (borrow), ADR-022 (safe-only)
- **Barra:** oráculos E2E compile+run; skip ≠ PASS; E0206 si método fuera whitelist.
- **Gobernanza:** Safe-only. Sin `unsafe`/FFI. Extiende whitelist ADR-006; no abre APIs nuevas fuera de esta tabla.

## Context

ADR-006 already pins `Vec::new` / `push` / `len` y `String` lit, but **does not** document `String.len` ni `is_empty`. Need minimal measurable std for AIs without theater ni APIs Rust libres.

## Pins GO (`STD-MIN-20260913`)

1. **`String.len() -> Int`** — longitud en **bytes** (emit Rust `String::len()` as `i64`). Document in PACK: not a grapheme count.
2. **`Vec<T>.len() -> Int`** — already in spirit ADR-006; confirm return **`Int` (`i64`)**.
3. **`String.is_empty() -> Bool`** y **`Vec<T>.is_empty() -> Bool`** — emit `.is_empty()`.
4. **Receivers v0:** only types **`String`** | **`Vec<_>`** (T ∈ {Int, Bool, String} as F2). Other receiver + `.len`/`.is_empty` → **E0206**.
5. **Unlisted method** (e.g. `.clear`, `.get`, `.chars`, index `[]`) → **E0206** `method not in F2 std whitelist`.
6. **Borrow:** `.len` / `.is_empty` = shared borrow of the receiver — **no move** (no E0201 por la llamada).
7. **Arity:** zero args; with args → **E0203** or E0206 (IMPL pin: prefer **E0203** type/arity mismatch if the name is on the whitelist, **E0206** if the name is not).
8. **Safe-only (ADR-022):** emit without `unsafe`; forbid en programa usuario.

## Whitelist F2 ampliada (canonical)

| Surface ARITA | Tipo return | Emit Rust (orientativo) |
|---------------|--------------|-------------------------|
| `print(expr)` | `Io<()>` (efecto) | `println!` |
| `Vec::new()` | `Vec<T>` | `Vec::new()` |
| `v.push(x)` | `()` | `Vec::push` |
| `v.len()` | `Int` | `Vec::len` as `i64` |
| `v.is_empty()` | `Bool` | `Vec::is_empty` |
| `s.len()` | `Int` | `String::len` as `i64` (**bytes**) |
| `s.is_empty()` | `Bool` | `String::is_empty` |

Everything else = **OUT** / E0206.

## Canonical form

```arita
module demo

fn main() -> Io<()> {
  let s: String = "hi"
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  v.push(2)
  print(s.len())
  print(v.len())
  print(s.is_empty())
  print(v.is_empty())
}
```

## OUT

- Indexing `s[i]` / `v[i]`, slicing, `chars`/`bytes` iterators
- `clear`, `pop`, `insert`, `remove`, `capacity`, `with_capacity`
- `len`/`is_empty` sobre `Int`/`Bool`/`&T` distinto de path tipado String|Vec
- Rust methods not listed (theater via free std)

## Oracles (IMPL)

| Id | Path (sugerido) | Expect |
|----|-----------------|--------|
| `f2-08-string-len` | `ejemplos/f2/08-string-len.arita` | stdout `2` for `"hi".len` / `s.len()` |
| `f2-09-is-empty` | `ejemplos/f2/09-is-empty.arita` | `true` / `false` / `false` |
| `neg-e0206-bad-method` | `ejemplos/f2/neg/e0206-bad-method.arita` | **E0206** |

Wire a `arita measure` only with E2E real.

## Relation ADR-006

La tabla §5 de ADR-006 queda **extended** por this ADR (String.len + is_empty). E0206 canonical message **no change**.

## Checklist Arquitecto

- [x] Safe-only
- [x] E0206 for fuera de whitelist
- [x] len String = bytes → Int; is_empty → Bool
- [x] No move en len/is_empty
- [x] No extra APIs
