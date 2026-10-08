[Español](02-ARCHITECTURE.md) | English

# Proposed architecture

```
  .arita  →  Parse (pest / lalrpop)
          →  AST + HIR
          →  Type + Ownership check (subset)
          →  Spec lowering → Logic IR (TML-shaped, motor propio)
          →  Codegen → Rust crate
          →  rustc / linker  →  binario nativo
```

LLVM / C-ABI direct: **post-MVP** (ADR-001). No IDNI-parser bridge (ADR-002 + `08-LICENSES-IDNI.md`).

## Layers

1. **Frontend:** lexer/parser (pest or lalrpop in F1), clear diagnostics for AIs (stable `E0xxx` codes).
2. **Semantic:** types, simplified ownership (move, shared/exclusive borrow).
3. **Logic:** TML-like island (`spec`/`fact`/`rule`/`query`); own checker; no free mix with effects.
4. **Codegen:** **emit Rust** in the MVP (ADR-001).
5. **Runtime std:** I/O, strings, vec; no `unwrap` in the public std.

## Targets (single table)

| OS | Arch | Fase |
|----|------|------|
| macOS | aarch64 | MVP (Fase 1) |
| Linux | x86_64 | MVP (Fase 1) |
| Linux | aarch64 | Fase 4 |
| Windows | x86_64 | Fase 4 |
| Windows | aarch64 | post-Fase 4 |
| macOS | x86_64 | post-Fase 4 |

## Toolchain host

The ARITA compiler itself is written in **Rust** (bootstrap in ARITA = Fase 5, optional).

## Workspace F1 (real surface)

Root: `<repo>` (`Cargo.toml` workspace).

| Crate | Rol |
|-------|-----|
| `crates/arita-syntax` | Parse / AST |
| `crates/arita-hir` | HIR + ownership check v0 (ADR-011; HIR-TYPES + HIR-CHECK-V0 ACCEPTED) |
| `crates/arita-logic` | Logic island F3 v0 (ADR-012; LOGIC-ENGINE-V0 landed) |
| `crates/arita-codegen` | Emit-Rust |
| `crates/arita-cli` | CLI `arita` |

See `DOC/ADR/003-f1-workspace-surface.md`. Spike status: Linux emit OK; Mac hello blocked by Lex Shell, not by the design.

## AST F1 (frozen, ADR-004)

**CUT-ID `F1-AST-RICH-20260913`** (authoritative; `main_prints` = historical):

```text
Module { name, functions: [Function { name, body }] }
Stmt::Expr / Expr::Call | Expr::LitStr / Call { callee, args }
```

Emit-Rust walks that tree. AST change: GO + new CUT-ID.
