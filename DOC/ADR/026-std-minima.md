# ADR-026 — Std mínima: `len` / `is_empty` (String + Vec)

- **Estado:** **aceptada** (DOC + IMPL CUT `STD-MIN-20260913`)
- **CUT-ID:** `STD-MIN-20260913`
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (draft/IMPL) · ARITA Arquitecto (**align** / pins)
- **Relacionados:** ADR-006 (whitelist F2), ADR-007 (MethodCall), ADR-009 (borrow), ADR-022 (safe-only)
- **Barra:** oráculos E2E compile+run; skip ≠ PASS; E0206 si método fuera whitelist.
- **Gobernanza:** Safe-only. Sin `unsafe`/FFI. Extiende whitelist ADR-006; no abre APIs nuevas fuera de esta tabla.

## Contexto

ADR-006 ya pinnea `Vec::new` / `push` / `len` y `String` lit, pero **no** documenta `String.len` ni `is_empty`. Hace falta std mínima medible para IAs sin theater ni APIs Rust libres.

## Pins GO (`STD-MIN-20260913`)

1. **`String.len() -> Int`** — longitud en **bytes** (emit Rust `String::len()` como `i64`). Documentar en PACK: no es conteo de graphemes.
2. **`Vec<T>.len() -> Int`** — ya en espíritu ADR-006; confirma retorno **`Int` (`i64`)**.
3. **`String.is_empty() -> Bool`** y **`Vec<T>.is_empty() -> Bool`** — emit `.is_empty()`.
4. **Receivers v0:** solo tipos **`String`** | **`Vec<_>`** (T ∈ {Int, Bool, String} como F2). Otro receptor + `.len`/`.is_empty` → **E0206**.
5. **Método no listado** (p.ej. `.clear`, `.get`, `.chars`, index `[]`) → **E0206** `method not in F2 std whitelist`.
6. **Borrow:** `.len` / `.is_empty` = shared borrow del receiver — **no move** (no E0201 por la llamada).
7. **Arity:** cero args; con args → **E0203** o E0206 (pin IMPL: preferir **E0203** type/arity mismatch si el nombre está en whitelist, **E0206** si el nombre no está).
8. **Safe-only (ADR-022):** emit sin `unsafe`; forbid en programa usuario.

## Whitelist F2 ampliada (canónica)

| Surface ARITA | Tipo retorno | Emit Rust (orientativo) |
|---------------|--------------|-------------------------|
| `print(expr)` | `Io<()>` (efecto) | `println!` |
| `Vec::new()` | `Vec<T>` | `Vec::new()` |
| `v.push(x)` | `()` | `Vec::push` |
| `v.len()` | `Int` | `Vec::len` as `i64` |
| `v.is_empty()` | `Bool` | `Vec::is_empty` |
| `s.len()` | `Int` | `String::len` as `i64` (**bytes**) |
| `s.is_empty()` | `Bool` | `String::is_empty` |

Todo lo demás = **OUT** / E0206.

## Forma canónica

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
- Métodos Rust no listados (theater vía std libre)

## Oráculos (IMPL)

| Id | Path (sugerido) | Expect |
|----|-----------------|--------|
| `f2-08-string-len` | `ejemplos/f2/08-string-len.arita` | stdout `2` para `"hi".len` / `s.len()` |
| `f2-09-is-empty` | `ejemplos/f2/09-is-empty.arita` | `true` / `false` / `false` |
| `neg-e0206-bad-method` | `ejemplos/f2/neg/e0206-bad-method.arita` | **E0206** |

Wire a `arita measure` solo con E2E real.

## Relación ADR-006

La tabla §5 de ADR-006 queda **extendida** por este ADR (String.len + is_empty). E0206 mensaje canónico **sin cambio**.

## Checklist Arquitecto

- [x] Safe-only
- [x] E0206 para fuera de whitelist
- [x] len String = bytes → Int; is_empty → Bool
- [x] No move en len/is_empty
- [x] Sin APIs extra
