# Arquitectura propuesta

```
  .arita  →  Parse (pest / lalrpop)
          →  AST + HIR
          →  Type + Ownership check (subset)
          →  Spec lowering → Logic IR (TML-shaped, motor propio)
          →  Codegen → Rust crate
          →  rustc / linker  →  binario nativo
```

LLVM / C-ABI directo: **post-MVP** (ADR-001). Sin bridge IDNI-parser (ADR-002 + `08-LICENSES-IDNI.md`).

## Capas

1. **Frontend:** lexer/parser (pest o lalrpop en F1), diagnósticos claros para IAs (códigos `E0xxx` estables).
2. **Semantic:** types, ownership simplificado (move, borrow compartido/exclusivo).
3. **Logic:** island TML-like (`spec`/`fact`/`rule`/`query`); verificador propio; no mezcla libre con efectos.
4. **Codegen:** **emitir Rust** en MVP (ADR-001).
5. **Runtime std:** I/O, strings, vec; sin `unwrap` en std pública.

## Targets (tabla única)

| OS | Arch | Fase |
|----|------|------|
| macOS | aarch64 | MVP (Fase 1) |
| Linux | x86_64 | MVP (Fase 1) |
| Linux | aarch64 | Fase 4 |
| Windows | x86_64 | Fase 4 |
| Windows | aarch64 | post-Fase 4 |
| macOS | x86_64 | post-Fase 4 |

## Host del toolchain

El compilador ARITA en sí se escribe en **Rust** (bootstrap en ARITA = Fase 5, opcional).

## Workspace F1 (surface real)

Raíz: `<repo>` (`Cargo.toml` workspace).

| Crate | Rol |
|-------|-----|
| `crates/arita-syntax` | Parse / AST |
| `crates/arita-hir` | HIR + ownership check v0 (ADR-011; HIR-TYPES + HIR-CHECK-V0 ACCEPTED) |
| `crates/arita-logic` | Isla lógica F3 v0 (ADR-012; LOGIC-ENGINE-V0 landed) |
| `crates/arita-codegen` | Emit-Rust |
| `crates/arita-cli` | CLI `arita` |

Ver `DOC/ADR/003-f1-workspace-surface.md`. Estado spike: Linux emit OK; Mac hello bloqueado por Lex Shell, no por el diseño.

## AST F1 (congelado, ADR-004)

**CUT-ID `F1-AST-RICH-20260913`** (manda; `main_prints` = histórico):

```text
Module { name, functions: [Function { name, body }] }
Stmt::Expr / Expr::Call | Expr::LitStr / Call { callee, args }
```

Emit-Rust camina ese árbol. Cambio de AST: GO + CUT-ID nuevo.

