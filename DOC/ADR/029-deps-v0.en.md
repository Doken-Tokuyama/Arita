Translation of `029-deps-v0.md`; the original is normative. / Traducción de `029-deps-v0.md`; el original es el normativo.

# ADR-029 — Deps v0 (whitelist + bridges, Phase 4c)

- **Estado:** **aceptada** + **IMPL verified** (Lex measure **69/69** accepted; CUT `DEPS-V0-20260914`)
- **CUT-ID:** `DEPS-V0-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (draft/align) + Ingeniero Rust (**GO DOC+IMPL**)
- **Relacionados:** ADR-001 (emit-Rust), ADR-022 / E0231 (safe-only), ADR-027 (tokio emit pin), ADR-028 (perf verified 66/66), ROADMAP Fase 4c
- **Gobernanza:** **aceptada**. CUT IMPL autorizado (`DEPS-V0-20260914`). Executor Ingeniero (stack completo); Parser/Codegen HOLD salvo review. Sin demo keys / theater. Nombre = **ARITA**.
- **Barra:** parse → emit → rustc/cargo → execute + `clippy -D warnings`; skip ≠ PASS; inconclusive ≠ accepted.

## Context

Product (<person>): **yes** to crates.io deps; **not** as free `use` nor `unsafe` in `.arita`. The model is **whitelist + bridges** in the generated crate (aligned with Async 027: runtime only in emit).

Perf 028 / Async 027 / std 026 ya verdes en Lex.

## Decision (MVP)

### 1. Principios

1. Surface ARITA **never** nombra paths de crates.io (`tokio::`, `serde::`, …).
2. Solo crates en **whitelist v0** pueden aparecer in the manifiesto ARITA / Cargo emitido.
3. **Bridge:** emit exposes APIs ARITA (fns/tipos of the dialect); the generated crate uses the dep behind the scenes.
4. Safe-only: deps que exijan `unsafe`/FFI **in the surface** = OUT / reject (ADR-022 / E0231).
5. Versiones **pinneadas** in emitted Cargo (as `tokio = "=1.53.1"` en 027).

### 2. Manifiesto (pin cerrado)

Project file: **`arita.toml`** next to the entry `.arita` (or package root).  
`deps { }` embebido en `.arita` = **OUT de v0** / future.

```toml
[package]
name = "demo"

[deps]
# only names on the v0 whitelist; version is fixed by emit (SoT pin)
tokio = { bridge = "async-runtime" }   # formaliza ADR-027; no nuevas APIs surface
```

### 3. Whitelist v0

| Crate | Bridge id | Surface expuesto v0 | Cargo pin (emit) |
|-------|-----------|---------------------|------------------|
| `tokio` | `async-runtime` | **Ninguno nuevo** — only habilita emit async ya definido en ADR-027 (`async fn` / `await` / `#[tokio::main(…)]`) | `=1.53.1`, features `rt`+`macros`, current_thread |

**Whitelist v0 = tokio-only** (pin cerrado). +1 crate safe = **CUT later**.

Lista whitelist vive en DOC (this ADR) + tabla en emit; ampliar = CUT nuevo.

### 4. Emit

1. Lee **`arita.toml` `[deps]`** only (v0).
2. Valida cada clave ∈ whitelist → si no, **E0260**.
3. Genera `Cargo.toml` of the crate usuario with deps pinneadas + `#![forbid(unsafe_code)]`.
4. Si el programa usa `async` without declarar bridge tokio when sea requerido = pin IMPL (error claro; puede reuse E0241 o E0260 — abierta menor).
5. Path `tokio::` / `use` de crate en `.arita` → **E0261**.

### 5. Diagnostics draft

| Code | Message EN canonical | When |
|--------|---------------------|--------|
| **E0260** | `dependency not in ARITA whitelist` | crate fuera de whitelist / mal declarado |
| **E0261** | `external crate path not allowed in surface` | `tokio::…` / `use` crate / reexport crudo en `.arita` |

(E0250 = perf profile; E024x = async; E0231 = unsafe.)

### 6. Oracles (after IMPL GO)

| Id | Path | Expect |
|----|------|--------|
| `deps-01-tokio-bridge` | `ejemplos/deps/01-tokio-bridge.arita` (+ `arita.toml`) | E2E async minimum ya cubierto / build with `[deps] tokio` explicit → stdout fijado |
| `deps-neg-e0260` | `ejemplos/deps/neg/e0260-unknown-crate.arita` | **E0260** |
| `deps-neg-e0261` (opc.) | surface with path ilegal | **E0261** |

### 7. OUT

- deps que exijan unsafe/FFI **expuesto** al surface
- `use` / paths crates.io libres en `.arita`
- “pegar” crates.io without whitelist
- demo keys / secrets en manifiesto
- reexport crudo of the crate al dialecto
- Host crate with deps libres **behind** de boundary (pattern border ROADMAP) = **future DOC**, not MVP surface

### 8. Pattern borde (nota, no IMPL v0)

Crate host Rust (deps libres) behind de bridge ARITA → **ADR-033** (`HOST-BORDER-20260914`, proposal DOC). ADR-029 v0 = whitelist+emit.

## Decisiones cerradas (2026-09-14 — Ingeniero GO)

1. **Solo `arita.toml` `[deps]`** v0. `deps { }` en `.arita` = OUT v0 / future.
2. Whitelist v0 = **tokio-only** (formaliza ADR-027; without APIs surface nuevas). +1 crate = CUT later.
3. **E0260 / E0261 OK.**

## Consequences

- CUT IMPL autorizado: executor Ingeniero (full stack, pattern ASYNC/PERF).
- Parser/Codegen: HOLD crates salvo review-only.
- ROADMAP 4c: this ADR **accepted**.
- Ampliar whitelist = CUT DOC+IMPL, no drift silencioso.

## Checklist GO

- [x] Bridges / no path crate en surface OK
- [x] Whitelist v0 tokio-only OK
- [x] E0260/E0261 OK
- [x] Abiertas 1–3 **cerradas**
- [x] GO DOC → **accepted** + IMPL autorizado
- [x] Oracles `ejemplos/deps/` E2E + measure Lex **69/69** accepted

## IMPL evidence (`DEPS-V0-20260914`)

| Id | Path | Expect | Verdict |
|----|------|--------|---------|
| deps-01-tokio-bridge | `ejemplos/deps/01-tokio-bridge.arita` + `arita.toml` | E2E stdout `hi` / `deps-ok`; Cargo.toml tokio `=1.53.1` | accepted (Lex measure) |
| deps-neg-e0260 | `ejemplos/deps/neg/e0260-unknown-crate/` | **E0260** | accepted |
| deps-neg-e0261 | `ejemplos/deps/neg/e0261-crate-path.arita` | **E0261** | accepted |

Pins: surface never names `tokio::`; emit pin from whitelist when `[deps]` declared; async path without manifest still works (ADR-027).
