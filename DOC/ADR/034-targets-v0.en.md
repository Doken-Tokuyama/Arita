Translation of `034-targets-v0.md`; the original is normative. / Traducción de `034-targets-v0.md`; el original es el normativo.

# ADR-034 — Targets v0 (Windows x86_64 + Linux aarch64)

- **Estado:** **aceptada** + **verified** (Lex measure **89/89 accepted**, **0 gated**; CUT toolchain `TARGETS-CROSS-LEX-20260915`)
- **CUT-ID:** `TARGETS-V0-20260914`
- **CUT-ID (cross Lex):** `TARGETS-CROSS-LEX-20260915`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (DOC) + Ingeniero Rust (IMPL posterior)
- **Relacionados:** ADR-001 (emit-Rust), ROADMAP Fase 4 targets, measure Lex **89/89** (`TARGETS-CROSS-LEX-20260915`)
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Cross oráculos **gated**.
- **Barra:** skip ≠ PASS; inconclusive ≠ accepted; never fake reject por falta de toolchain.

## Context

Phase 4: Windows x86_64 + Linux aarch64. Emit-Rust hereda targets de rustc/cargo.

## Decision (pins GO 2026-09-14)

### 1. IN

| Pieza | Regla v0 |
|-------|----------|
| Windows | **`x86_64-pc-windows-gnu`** (default). `x86_64-pc-windows-msvc` = **OUT v0** / doc future |
| Linux ARM | `aarch64-unknown-linux-gnu` |
| CLI | `arita build --target <triple>` → cargo/rustc of the crate emitido |
| Default | host triple (comportamiento current) |
| Toolchain | DOC: `rustup target add <triple>` requerido before de oracle cross |
| Measure | oracles **gated**: without toolchain/linker → **inconclusive** for that oracle; **no** tumba la suite with fake reject |

### 2. Oracles (IMPL)

| Id | Expect |
|----|--------|
| `target-win-gnu` | build `--target x86_64-pc-windows-gnu` → artifact/rustc OK **iff** toolchain; else **inconclusive** |
| `target-linux-arm64` | same for `aarch64-unknown-linux-gnu` |

Positive control: build host without `--target` stays in measure baseline.

### 3. OUT

- Invent PASS or reject without linker/toolchain
- `windows-msvc` en v0
- Additional targets without CUT
- Cambiar emit backend (remains ADR-001)

## Decisiones abiertas

**Cerradas** (GO Ingeniero):

1. ~~msvc vs gnu~~ → **gnu** v0; msvc OUT
2. ~~rustup doc~~ → requerido; missing → inconclusive
3. ~~measure present vs gated~~ → **gated**; inconclusive no tumba suite

## Checklist GO

- [x] `--target` + triples OK
- [x] inconclusive-not-PASS / not fake-reject OK
- [x] Abiertas 1–3 cerradas
- [x] ADR **accepted**
- [x] IMPL (`TARGETS-V0-20260914`); gated `target-win-gnu` / `target-linux-arm64`
- [x] Lex cross linkers (`TARGETS-CROSS-LEX-20260915`); `.cargo/config.toml`; measure **89/89**; HOLD → review-only; no change surface crates

## Addendum — Lex cross linkers (**verified**)

- **CUT:** `TARGETS-CROSS-LEX-20260915` (ADR-034; **no** nuevo ADR)
- **IN:** `.cargo/config.toml` host linkers `aarch64-linux-gnu-gcc` + `x86_64-w64-mingw32-gcc`
- **Lex:** measure **89/89** (gated targets accepted; no crate/surface change)
- **OUT:** CI remote / Origin; fake PASS without linker
