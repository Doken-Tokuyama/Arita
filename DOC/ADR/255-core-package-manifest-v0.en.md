Translation of `255-core-package-manifest-v0.md`; the original is normative. / Traducción de `255-core-package-manifest-v0.md`; el original es el normativo.

# ADR-255 — Core 0.4 slice 2: PACKAGE-MANIFEST

- **Estado:** **CLOSED** Lex **668/668** (2026-09-20) · siguiente [ADR-256](256-core-lib-api-v0.md)
- **CUT-ID:** `CORE-0.4-PACKAGE-MANIFEST-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-254](254-core-0.4-pins.md) §0.1 slice 2
- **Prev:** slice 1 MULTI-MODULE **CLOSED** Lex **662/662**
- **HOLD:** crates.io · `[]`/insert/Mutex · idle-kill · TLS/WS · repair · proc-macros

## Objective

**Package** manifest + path-local **Cargo workspace** `{lib, bin}` emit — reproducible, no crates.io.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Manifest file** | `arita.toml` (or `package.arita.toml`) at package root — **one** v0 format |
| **Campos IN** | `name`, `version`, `kind` ∈ {`lib`,`bin`,`workspace`}, `modules[]` / layout dirs |
| **Workspace** | emit `Cargo.toml` workspace members `crates/lib_*`, `crates/bin_*` (stable emit names DOC) |
| **Deps** | `path =` workspace/internal only — **forbid** crates.io keys |
| **OUT** | remote semver deps · cargo features theater · user build.rs |

### Minimal example

```toml
[package]
name = "demo"
version = "0.1.0"

[workspace]
members = ["lib", "bin"]
```

(Exact schema pinned in Codegen emit notes; fields above are semantic IN.)

## 1. Language surface

No mandatory new keywords if layout+toml suffice. If a `package` block in `.arita` is needed — OUT v0 (toml-only). Multi-module (254 slice1) stays.

## 2. Oracles

| Id | Expect |
|----|--------|
| `core04-pkg-manifest-parse` | valid toml → emit workspace |
| `core04-pkg-lib-bin-emit` | lib+bin members present in emit |
| `core04-pkg-build` | green `arita build` / cargo check |
| `neg-core04-pkg-cratesio` | crates.io dep → **E0xxx** reject (Codegen pin code; suggest E0330) |
| `neg-core04-pkg-bad-toml` | broken toml → stable diag |

## 3. CLOSED criterion

Lex §2 green; tick ADR-254 slice 2; HOLDs intact. Next: slice 3 LIB-API.

## Checklist

- [x] Manifest pins + workspace emit + oracles
- [x] IMPL + Lex **668/668**
- [x] Slice 3 GO → ADR-256
