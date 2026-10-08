# ADR-034 — Targets v0 (Windows x86_64 + Linux aarch64)

- **Estado:** **aceptada** + **verified** (Lex measure **89/89 accepted**, **0 gated**; CUT toolchain `TARGETS-CROSS-LEX-20260915`)
- **CUT-ID:** `TARGETS-V0-20260914`
- **CUT-ID (cross Lex):** `TARGETS-CROSS-LEX-20260915`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (DOC) + Ingeniero Rust (IMPL posterior)
- **Relacionados:** ADR-001 (emit-Rust), ROADMAP Fase 4 targets, measure Lex **89/89** (`TARGETS-CROSS-LEX-20260915`)
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Cross oráculos **gated**.
- **Barra:** skip ≠ PASS; inconclusive ≠ accepted; never fake reject por falta de toolchain.

## Contexto

Fase 4: Windows x86_64 + Linux aarch64. Emit-Rust hereda targets de rustc/cargo.

## Decisión (pins GO 2026-09-14)

### 1. IN

| Pieza | Regla v0 |
|-------|----------|
| Windows | **`x86_64-pc-windows-gnu`** (default). `x86_64-pc-windows-msvc` = **OUT v0** / doc futuro |
| Linux ARM | `aarch64-unknown-linux-gnu` |
| CLI | `arita build --target <triple>` → cargo/rustc del crate emitido |
| Default | host triple (comportamiento actual) |
| Toolchain | DOC: `rustup target add <triple>` requerido antes de oráculo cross |
| Measure | oráculos **gated**: sin toolchain/linker → **inconclusive** para ese oracle; **no** tumba la suite con fake reject |

### 2. Oráculos (IMPL)

| Id | Expect |
|----|--------|
| `target-win-gnu` | build `--target x86_64-pc-windows-gnu` → artifact/rustc OK **iff** toolchain; else **inconclusive** |
| `target-linux-arm64` | igual para `aarch64-unknown-linux-gnu` |

Positivo control: build host sin `--target` sigue en measure baseline.

### 3. OUT

- Inventar PASS o reject sin linker/toolchain
- `windows-msvc` en v0
- Targets adicionales sin CUT
- Cambiar emit backend (sigue ADR-001)

## Decisiones abiertas

**Cerradas** (GO Ingeniero):

1. ~~msvc vs gnu~~ → **gnu** v0; msvc OUT
2. ~~rustup doc~~ → requerido; missing → inconclusive
3. ~~measure present vs gated~~ → **gated**; inconclusive no tumba suite

## Checklist GO

- [x] `--target` + triples OK
- [x] inconclusive-not-PASS / not fake-reject OK
- [x] Abiertas 1–3 cerradas
- [x] ADR **aceptada**
- [x] IMPL (`TARGETS-V0-20260914`); gated `target-win-gnu` / `target-linux-arm64`
- [x] Lex cross linkers (`TARGETS-CROSS-LEX-20260915`); `.cargo/config.toml`; measure **89/89**; HOLD → review-only; sin cambio surface crates

## Addendum — Lex cross linkers (**verified**)

- **CUT:** `TARGETS-CROSS-LEX-20260915` (ADR-034; **no** nuevo ADR)
- **IN:** `.cargo/config.toml` host linkers `aarch64-linux-gnu-gcc` + `x86_64-w64-mingw32-gcc`
- **Lex:** measure **89/89** (gated targets accepted; no crate/surface change)
- **OUT:** CI remote / Origin; fake PASS sin linker
