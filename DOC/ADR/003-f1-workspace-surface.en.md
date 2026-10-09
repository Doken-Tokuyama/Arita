Translation of `003-f1-workspace-surface.md`; the original is normative. / Traducción de `003-f1-workspace-surface.md`; el original es el normativo.

# ADR-003 — Surface F1 (workspace spike)

- **Estado:** aceptada (spike en curso)
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (implementación) + ARITA Arquitecto (congelación DOC)
- **Relacionados:** ADR-001 (emit-Rust), ADR-002 (isla propia)

## Context

Phase 1 spike GO bajo `<repo>`. Hay que alinear la DOC congelada F0 with el surface real of the repo.

## Decision (canonical F1 surface)

1. **Cargo workspace** en la root de ARITA (`resolver = "2"`).
2. Crates:
   - `crates/arita-syntax` — parse / AST (frontend)
   - `crates/arita-codegen` — emit-Rust (depende de syntax)
   - `crates/arita-cli` — binario `arita` (syntax + codegen)
3. Pipeline spike: **`.arita` → syntax → codegen (Rust) → `rustc`/`cargo` → binario**.
4. **Linux x86_64:** emit-Rust MVP verificado in the spike.
5. **macOS aarch64 hello:** pending — Lex Shell local roto (`zsh ENOENT`); no es fallo de architecture.
6. **Origin `new_repo`:** aplazado until que <person> provea namespace; trabajo local en `<repo>`.
7. Edition de crates of the spike: **2021** (pin MSRV / possible bump a 2024 = later F1 decision; does not block emit).

## Out of this ADR

- Logic island / TML: remains ADR-002 (Phase 3).
- LLVM directo: remains post-MVP (ADR-001).
- AST shape F1: **ADR-004** CUT-ID `F1-AST-RICH-20260913` (`Module`/`Function`/`Call`/`LitStr`; `main_prints` historical; GO for changes).

## Enlaces

- `Cargo.toml` (workspace members)
- `DOC/02-ARCHITECTURE.md`
- `ROADMAP.md` — Phase 1
