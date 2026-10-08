Translation of `001-emit-rust-mvp.md`; the original is normative. / Traducción de `001-emit-rust-mvp.md`; el original es el normativo.

# ADR-001 — MVP backend: emit-Rust

- **Estado:** aceptada
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust + ARITA Docs (congelación F0)
- **Relacionados:** `DOC/08-LICENSES-IDNI.md`, `DOC/ADR/002-isla-logica-propia.md`, `DOC/ADR/003-f1-workspace-surface.md`

## Context

Phase 0 required a backend decision: emit Rust vs direct LLVM/C-ABI. Multi-target is needed soon (macOS aarch64 + Linux x86_64) without writing a native backend.

## Decision

1. **MVP = emit Rust** and compile with `rustc` / `cargo`. The F1 spike uses **edition 2021** crates; MSRV pin / bump to 2024 = F1 decision (does not change emit).
2. Canonical pipeline: **ARITA → Rust source → binary**.
3. **Direct LLVM / C-ABI = post-MVP** (outside v0). Optional later for FFI/staticlib.
4. Emit output: `#![forbid(unsafe_code)]` **always** on user programs. (The “dialect unsafe modules / late phase” caveat is **superseded** by ADR-022 safe-only.)

## Consequences

- `arita build` and the F1 spike assume rustup + a pinned toolchain (MSRV to document in F1).
- MVP targets inherit rustc’s; see the table in `DOC/02-ARCHITECTURE.md` / `DOC/05-COMPILATION-TARGETS.md`.
- The logic island (ADR-002) does not change this backend.
- Spike crate surface: ADR-003 (`arita-syntax` / `arita-codegen` / `arita-cli`).

## Rejected alternatives (v0)

| Option | Why not |
|--------|------------|
| Direct LLVM on day 1 | High cost; emit-Rust covers MVP targets |
| C ABI only | Loses Rust-ecosystem ergonomics on the critical path |

## Links

- `ROADMAP.md` — Phase 0 backend decision; Phase 1 emit
- `DOC/05-COMPILATION-TARGETS.md`
- `DOC/ADR/003-f1-workspace-surface.md`
