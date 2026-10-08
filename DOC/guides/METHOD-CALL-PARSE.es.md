Español | [English](METHOD-CALL-PARSE.md)

# Cómo ARITA analiza las llamadas a métodos (guía para IAs)

**Audiencia:** agentes que escriben `.arita` · **Status:** DOC only (Parser inventory 2026-09-18)  
**SoT grammar:** `crates/arita-syntax/src/arita.pest`

## Formas

| Forma | Regla pest | Uso típico |
|------|-----------|-------------|
| `v.push(x)` | `method_call_push` | solo **stmt** |
| `v.len()` / `s.is_empty()` | `method_call_len` / `method_call_is_empty` | init de let, arg de print, rhs de assign, lado de assert |
| `recv.name(args…)` | `method_call_other` | stmt, init de let, print, assign, scrutinee de `while let` |

`method_call_other` cubre métodos F2 que HIR lista en blanco después: `clear`, `pop`, `unwrap_or`, `is_some` / `is_none` / `is_ok` / `is_err`, `wrapping_*`, `saturating_*`, `checked_*`, ayudantes de string, etc. Los nombres desconocidos **sí parsean**; HIR los rechaza con **E0206**.

El receptor v0 es un **`ident`** desnudo (no `a.b.c`, no `(expr).m()`).

Args de `method_call_other` / `push`: solo `string_lit | int_lit | ident` (sin llamadas anidadas, sin `Some(x)` como arg).

## Dónde funciona

✅ `let x: T = recv.method(…)`  
✅ `print(recv.method(…))`  
✅ `x = recv.method(…)` (assign)  
✅ stmt: `recv.clear()` / `recv.push(1)` / `recv.wrapping_add(1)` as expr-stmt via `method_call_other`  
✅ `while let Some(x) = v.pop() { … }` (`while_let_scrutinee` includes `method_call_other`)

## Dónde falla (trampas de sintaxis)

| Quieres | **No** escribas | Prefiere |
|------|------------------|--------|
| Ramificar sobre método Bool | `if v.is_empty() { … }` | `let e: Bool = v.is_empty()` then `if e { … }` **or** `print(v.is_empty())` in tests |
| Bucle sobre método Bool | `while r.is_ok() { … }` | bind to Bool first / use `while let` when Option/Result |
| Match sobre resultado de método | `match v.pop() { … }` | `let o: Option<Int> = v.pop()` then `match o { … }` |
| Método en lado de assert (no-len) | `assert x.is_some() == true` | bind Bool, or assert via print/oracle |
| Arg de método anidado | `a.min(b.max(c))` as single call | temps: `let t: Int = b.max(c)` then `a.min(t)` |
| Método en primary binary/cmp | `a + b.wrapping_add(1)` | `let t: Int = b.wrapping_add(1)` then `a + t` |

**Causa raíz:** `bool_expr = cmp_expr | bool_lit | ident` — **no** `method_call_*`.  
`primary = int_lit | ident` — los métodos no van en operandos aritméticos/cmp.  
`match_scrutinee = bool_lit | int_lit | ident` — sin método.

Trampa histórica: las IAs emiten `if recv.method()` → fallo de parse / E0006. Prefiere **`print(recv.method())`** para oráculos measure, o **`let` + bind y luego `if`**.

## Diagnósticos (tras el parse)

- Bad arity / type on whitelisted name → **E0203**
- Name not in F2 whitelist → **E0206**
- Exclusive methods (`push`/`clear`/`pop`/…) on non-`mut` → **E0202** (post STD-CLEAR mut gate)

## Checklist para IAs

1. ¿Necesitas Bool de un método en `if`/`while`? → **bind primero**.  
2. ¿Necesitas match sobre `pop`/`get`? → **bind Option/Result primero**.  
3. Prefiere `print(recv.method())` para oráculos de stdout.  
4. No inventes encadenamiento estilo Rust; ARITA v0 es denso en stmt/`let`.
