# ADR-001 — Backend MVP: emit-Rust

- **Estado:** aceptada
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust + ARITA Docs (congelación F0)
- **Relacionados:** `DOC/08-LICENSES-IDNI.md`, `DOC/ADR/002-isla-logica-propia.md`, `DOC/ADR/003-f1-workspace-surface.md`

## Contexto

Fase 0 pedía decidir backend: emitir Rust vs LLVM/C-ABI directo. Hace falta multi-target pronto (macOS aarch64 + Linux x86_64) sin escribir un backend nativo.

## Decisión

1. **MVP = emitir Rust** y compilar con `rustc` / `cargo`. El spike F1 usa crates **edition 2021**; pin MSRV / subida a 2024 = decisión F1 (no cambia emit).
2. Pipeline canónico: **ARITA → fuente Rust → binario**.
3. **LLVM / C-ABI directo = post-MVP** (fuera de v0). Opcional más tarde para FFI/staticlib.
4. Output del emit: `#![forbid(unsafe_code)]` **siempre** en programas de usuario. (La salvedad “módulos unsafe de dialecto / fase tardía” queda **superseded** por ADR-022 safe-only.)

## Consecuencias

- `arita build` y el spike F1 asumen rustup + toolchain pin (MSRV a documentar en F1).
- Targets MVP heredan los de rustc; ver tabla en `DOC/02-ARCHITECTURE.md` / `DOC/05-COMPILATION-TARGETS.md`.
- La isla lógica (ADR-002) no cambia este backend.
- Surface de crates del spike: ADR-003 (`arita-syntax` / `arita-codegen` / `arita-cli`).

## Alternativas rechazadas (v0)

| Opción | Por qué no |
|--------|------------|
| LLVM directo día 1 | Coste alto; emit-Rust cubre targets MVP |
| Solo C ABI | Pierde ergonomía del ecosistema Rust en el camino crítico |

## Enlaces

- `ROADMAP.md` — Fase 0 decisión backend; Fase 1 emit
- `DOC/05-COMPILATION-TARGETS.md`
- `DOC/ADR/003-f1-workspace-surface.md`
