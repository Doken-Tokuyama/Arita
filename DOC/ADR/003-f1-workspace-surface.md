# ADR-003 — Surface F1 (workspace spike)

- **Estado:** aceptada (spike en curso)
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (implementación) + ARITA Arquitecto (congelación DOC)
- **Relacionados:** ADR-001 (emit-Rust), ADR-002 (isla propia)

## Contexto

Fase 1 spike GO bajo `<repo>`. Hay que alinear la DOC congelada F0 con el surface real del repo.

## Decisión (surface canónico F1)

1. **Cargo workspace** en la raíz de ARITA (`resolver = "2"`).
2. Crates:
   - `crates/arita-syntax` — parse / AST (frontend)
   - `crates/arita-codegen` — emit-Rust (depende de syntax)
   - `crates/arita-cli` — binario `arita` (syntax + codegen)
3. Pipeline spike: **`.arita` → syntax → codegen (Rust) → `rustc`/`cargo` → binario**.
4. **Linux x86_64:** emit-Rust MVP verificado en el spike.
5. **macOS aarch64 hello:** pendiente — Lex Shell local roto (`zsh ENOENT`); no es fallo de arquitectura.
6. **Origin `new_repo`:** aplazado hasta que <person> provea namespace; trabajo local en `<repo>`.
7. Edition de crates del spike: **2021** (pin MSRV / posible subida a 2024 = decisión F1 posterior; no bloquea emit).

## Fuera de este ADR

- Isla lógica / TML: sigue ADR-002 (Fase 3).
- LLVM directo: sigue post-MVP (ADR-001).
- Forma del AST F1: **ADR-004** CUT-ID `F1-AST-RICH-20260913` (`Module`/`Function`/`Call`/`LitStr`; `main_prints` histórico; GO para cambios).

## Enlaces

- `Cargo.toml` (workspace members)
- `DOC/02-ARCHITECTURE.md`
- `ROADMAP.md` — Fase 1
