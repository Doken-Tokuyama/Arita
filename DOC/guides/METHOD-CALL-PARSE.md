# How ARITA parses method calls (IA guide)

**Audience:** coding agents writing `.arita` · **Status:** DOC only (Parser inventory 2026-09-18)  
**SoT grammar:** `crates/arita-syntax/src/arita.pest`

## Forms

| Form | Pest rule | Typical use |
|------|-----------|-------------|
| `v.push(x)` | `method_call_push` | **stmt** only |
| `v.len()` / `s.is_empty()` | `method_call_len` / `method_call_is_empty` | let init, print arg, assign rhs, assert side |
| `recv.name(args…)` | `method_call_other` | stmt, let init, print, assign, `while let` scrutinee |

`method_call_other` covers F2 methods that HIR whitelists later: `clear`, `pop`, `unwrap_or`, `is_some` / `is_none` / `is_ok` / `is_err`, `wrapping_*`, `saturating_*`, `checked_*`, string helpers, etc. Unknown names still **parse**; HIR rejects with **E0206**.

Receiver v0 is a bare **`ident`** (no `a.b.c`, no `(expr).m()`).

Args for `method_call_other` / `push`: `string_lit | int_lit | ident` only (no nested calls, no `Some(x)` as arg).

## Where it works

✅ `let x: T = recv.method(…)`  
✅ `print(recv.method(…))`  
✅ `x = recv.method(…)` (assign)  
✅ stmt: `recv.clear()` / `recv.push(1)` / `recv.wrapping_add(1)` as expr-stmt via `method_call_other`  
✅ `while let Some(x) = v.pop() { … }` (`while_let_scrutinee` includes `method_call_other`)

## Where it fails (syntax traps)

| Want | Do **not** write | Prefer |
|------|------------------|--------|
| Branch on Bool method | `if v.is_empty() { … }` | `let e: Bool = v.is_empty()` then `if e { … }` **or** `print(v.is_empty())` in tests |
| Loop on Bool method | `while r.is_ok() { … }` | bind to Bool first / use `while let` when Option/Result |
| Match on method result | `match v.pop() { … }` | `let o: Option<Int> = v.pop()` then `match o { … }` |
| Method in assert side (non-len) | `assert x.is_some() == true` | bind Bool, or assert via print/oracle |
| Nested method arg | `a.min(b.max(c))` as single call | temps: `let t: Int = b.max(c)` then `a.min(t)` |
| Method in binary/cmp primary | `a + b.wrapping_add(1)` | `let t: Int = b.wrapping_add(1)` then `a + t` |

**Root cause:** `bool_expr = cmp_expr | bool_lit | ident` — **no** `method_call_*`.  
`primary = int_lit | ident` — methods not in arithmetic/cmp operands.  
`match_scrutinee = bool_lit | int_lit | ident` — no method.

Historic trap: IAs emit `if recv.method()` → parse fail / E0006. Prefer **`print(recv.method())`** for measure oracles, or **`let` bind then `if`**.

## Diagnostics (after parse)

- Bad arity / type on whitelisted name → **E0203**
- Name not in F2 whitelist → **E0206**
- Exclusive methods (`push`/`clear`/`pop`/…) on non-`mut` → **E0202** (post STD-CLEAR mut gate)

## Checklist for IAs

1. Need Bool from method in `if`/`while`? → **bind first**.  
2. Need match on `pop`/`get`? → **bind Option/Result first**.  
3. Prefer `print(recv.method())` for stdout oracles.  
4. Don’t invent Rust chaining; ARITA v0 is flat stmt/`let` heavy.
