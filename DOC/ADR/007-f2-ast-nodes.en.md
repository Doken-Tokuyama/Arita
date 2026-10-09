Translation of `007-f2-ast-nodes.md`; the original is normative. / Traducción de `007-f2-ast-nodes.md`; el original es el normativo.

# ADR-007 — AST F2 nodes

- **Estado:** **aceptada**
- **CUT-ID:** `F2-AST-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO)
- **Relacionados:** ADR-004 (`F1-AST-RICH-20260913` = **F1 subset vigente**), ADR-006 (`F2-OWNERSHIP-20260913`)
- **Alcance:** contrato AST. Impl crates only after **IMPL GO** explicit de Ingeniero (no this CUT only).
- **Barra:** E2E; skip ≠ PASS; sin F2 `.arita` en `ejemplos/` hasta toolchain E2E.

## Pins GO (`F2-AST-20260913`)

1. **Extender `Function`** with `params` + `ret_ty` — **no** invent un tipo `Fn` paralelo.
2. **`assert`:** `Stmt::Assert { lhs: Expr, rhs: Expr }` dentro de `Item::Test` (no `Binary` Eq suelto) — facilita **E0211**.
3. Resto of the set de nodes OK: `Let`, `LitInt`/`LitBool`, `Path`, `Binary`, `Borrow`, `MethodCall`, `Type`, `FnParam`, `Test`.
4. **ADR-004** remains as subset F1; **007 extends** (no lo replaces).

## Decision (nodes)

### F1 subset (ADR-004 — intact as core)

```text
Module { name, functions: [Function] }
Function { … }          # ver extensión F2 abajo
Stmt::Expr(Expr)
Expr::Call(Call) | Expr::LitStr(String)
Call { callee, args }
```

### Extension F2 (this ADR)

```text
# Function — EXTENDER (pin 1), no nuevo tipo
Function {
  name,
  params: [FnParam],     # F2; F1.1 ⇒ vacío
  ret_ty: Type,          # F2; F1.1 ⇒ IoUnit / equivalente main
  body: [Stmt],
}
FnParam { name, ty: Type }

Stmt::Let { mutable, name, ty, init: Expr }
Stmt::Expr(Expr)                    # F1
Stmt::Assert { lhs: Expr, rhs: Expr }  # solo dentro de Item::Test (pin 2)

Expr::Call | LitStr                 # F1
Expr::LitInt(i64)                   # Int = i64 (ADR-006)
Expr::LitBool(bool)
Expr::Path(Path)
Expr::Binary { op, lhs, rhs }       # Int arithmetic v0; Eq solo vía Assert
Expr::Borrow { mutable, inner }
Expr::MethodCall { receiver, method, args }

Type::Int | Bool | String | Vec(Box<Type>) | IoUnit

Item::Test { name, body: [Stmt] }   # body puede incluir Assert
Module { name, functions, tests: [Item::Test] }  # o items unificados — pin impl: preferir `tests: Vec<Test>` en Module sin romper parse F1.1
```

### Lowering (referencia)

| Surface | AST |
|---------|-----|
| `print("hi")` | `Call { print, [LitStr] }` |
| `let n: Int = 1` | `Let { mut:false, n, Int, LitInt(1) }` |
| `&x` / `&mut x` | `Borrow { mutable }` |
| `assert a == b` | `Stmt::Assert { lhs: a, rhs: b }` en `Test` |

### Fuera

- `if` / `while` (F2.1 **done**, ADR-014); `match` stmt (F2.2 **done**, ADR-015) — outside this AST F2 freeze
- lifetimes, traits, generics user-defined
- nodes logic island (F3)
- tipo `Fn` parallel to `Function`

## Orden de impl (Ingeniero — DOC)

**After** de flip Docs, y **only** after **IMPL GO** explicit:

1. Parser: **ADD types only** en `arita-syntax` (structs/enums).
2. F1.1 parse + tests deben seguir **verdes**.
3. **No** F2 pest still; **no** F2 ejemplos; **no** HIR until nuevo IMPL GO.

Arquitecto / Docs: no writes a crates.

## Consequences

- Code F1.1 no se rompe: campos F2 with defaults / `Vec::new()` / opciones compatibles.
- Measure anti-theater uses `Stmt::Assert` for E0211.
- Next: IMPL GO de Ingeniero (types-only), no pest F2.

## Checklist GO — cerrado

- [x] Extender `Function` (no `Fn` paralelo)
- [x] `Stmt::Assert` en `Test`
- [x] Node set OK
- [x] ADR-004 = F1 subset; 007 extends
- [x] GO `F2-AST-20260913` → accepted

## Verification (Ingeniero Rust, 2026-09-13)

- Types-only F2 AST en `arita-syntax`: **landed + verified**.
- F1.1 parse/tests: **green**.
- Still **no**: F2 pest, F2 ejemplos, HIR (wait for IMPL GO).

## Enlaces

- `DOC/ADR/004-f1-ast-freeze.md`, `DOC/ADR/006-f2-ownership-surface.md`
