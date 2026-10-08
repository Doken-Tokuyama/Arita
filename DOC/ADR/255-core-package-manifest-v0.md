# ADR-255 — Core 0.4 slice 2: PACKAGE-MANIFEST

- **Estado:** **CLOSED** Lex **668/668** (2026-09-20) · siguiente [ADR-256](256-core-lib-api-v0.md)
- **CUT-ID:** `CORE-0.4-PACKAGE-MANIFEST-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-254](254-core-0.4-pins.md) §0.1 slice 2
- **Prev:** slice 1 MULTI-MODULE **CLOSED** Lex **662/662**
- **HOLD:** crates.io · `[]`/insert/Mutex · idle-kill · TLS/WS · repair · proc-macros

## Objetivo

Manifiesto de **package** + emit **Cargo workspace** `{lib, bin}` path-local — reproducible, sin crates.io.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Manifest file** | `arita.toml` (o `package.arita.toml`) en root package — **un** formato v0 |
| **Campos IN** | `name`, `version`, `kind` ∈ {`lib`,`bin`,`workspace`}, `modules[]` / layout dirs |
| **Workspace** | emit `Cargo.toml` workspace members `crates/lib_*`, `crates/bin_*` (nombres emit estables DOC) |
| **Deps** | solo `path =` workspace/internal — **forbid** crates.io keys |
| **OUT** | semver deps remote · features cargo theater · build.rs user |

### Ejemplo mínimo

```toml
[package]
name = "demo"
version = "0.1.0"

[workspace]
members = ["lib", "bin"]
```

(Exact schema pin en emit notes Codegen; campos arriba son IN semánticos.)

## 1. Surface lenguaje

Sin keywords nuevas obligatorias si layout+toml bastan. Si hace falta `package` block en `.arita` — OUT v0 (toml-only). Multi-module (254 slice1) sigue.

## 2. Oracles

| Id | Expect |
|----|--------|
| `core04-pkg-manifest-parse` | toml válido → emit workspace |
| `core04-pkg-lib-bin-emit` | members lib+bin presentes en emit |
| `core04-pkg-build` | `arita build` / cargo check verde |
| `neg-core04-pkg-cratesio` | dep crates.io → **E0xxx** reject (código pin Codegen; sugerir E0330) |
| `neg-core04-pkg-bad-toml` | toml roto → diag estable |

## 3. Criterio CLOSED

Lex §2 verde; tick ADR-254 slice 2; HOLDs intactos. Siguiente: slice 3 LIB-API.

## Checklist

- [x] Pins manifest + emit workspace + oracles
- [x] IMPL + Lex **668/668**
- [x] Slice 3 GO → ADR-256
