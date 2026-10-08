# ADR-029 — Deps v0 (whitelist + bridges, Fase 4c)

- **Estado:** **aceptada** + **IMPL verified** (Lex measure **69/69** accepted; CUT `DEPS-V0-20260914`)
- **CUT-ID:** `DEPS-V0-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (draft/align) + Ingeniero Rust (**GO DOC+IMPL**)
- **Relacionados:** ADR-001 (emit-Rust), ADR-022 / E0231 (safe-only), ADR-027 (tokio emit pin), ADR-028 (perf verified 66/66), ROADMAP Fase 4c
- **Gobernanza:** **aceptada**. CUT IMPL autorizado (`DEPS-V0-20260914`). Executor Ingeniero (stack completo); Parser/Codegen HOLD salvo review. Sin demo keys / theater. Nombre = **ARITA**.
- **Barra:** parse → emit → rustc/cargo → execute + `clippy -D warnings`; skip ≠ PASS; inconclusive ≠ accepted.

## Contexto

Producto (<person>): **sí** a deps crates.io; **no** como `use` libre ni `unsafe` en `.arita`. El modelo es **whitelist + bridges** en el crate generado (alineado Async 027: runtime solo en emit).

Perf 028 / Async 027 / std 026 ya verdes en Lex.

## Decisión (MVP)

### 1. Principios

1. Surface ARITA **nunca** nombra paths de crates.io (`tokio::`, `serde::`, …).
2. Solo crates en **whitelist v0** pueden aparecer en el manifiesto ARITA / Cargo emitido.
3. **Bridge:** el emit expone APIs ARITA (fns/tipos del dialecto); el crate generado usa la dep por detrás.
4. Safe-only: deps que exijan `unsafe`/FFI **en el surface** = OUT / reject (ADR-022 / E0231).
5. Versiones **pinneadas** en Cargo emitido (como `tokio = "=1.53.1"` en 027).

### 2. Manifiesto (pin cerrado)

Archivo de proyecto: **`arita.toml`** junto al entry `.arita` (o raíz de paquete).  
`deps { }` embebido en `.arita` = **OUT de v0** / futuro.

```toml
[package]
name = "demo"

[deps]
# solo nombres en whitelist v0; versión la fija el emit (pin SoT)
tokio = { bridge = "async-runtime" }   # formaliza ADR-027; no nuevas APIs surface
```

### 3. Whitelist v0

| Crate | Bridge id | Surface expuesto v0 | Cargo pin (emit) |
|-------|-----------|---------------------|------------------|
| `tokio` | `async-runtime` | **Ninguno nuevo** — solo habilita emit async ya definido en ADR-027 (`async fn` / `await` / `#[tokio::main(…)]`) | `=1.53.1`, features `rt`+`macros`, current_thread |

**Whitelist v0 = tokio-only** (pin cerrado). +1 crate safe = **CUT posterior**.

Lista whitelist vive en DOC (este ADR) + tabla en emit; ampliar = CUT nuevo.

### 4. Emit

1. Lee **`arita.toml` `[deps]`** únicamente (v0).
2. Valida cada clave ∈ whitelist → si no, **E0260**.
3. Genera `Cargo.toml` del crate usuario con deps pinneadas + `#![forbid(unsafe_code)]`.
4. Si el programa usa `async` sin declarar bridge tokio cuando sea requerido = pin IMPL (error claro; puede reusar E0241 o E0260 — abierta menor).
5. Path `tokio::` / `use` de crate en `.arita` → **E0261**.

### 5. Diagnósticos draft

| Código | Mensaje EN canónico | Cuándo |
|--------|---------------------|--------|
| **E0260** | `dependency not in ARITA whitelist` | crate fuera de whitelist / mal declarado |
| **E0261** | `external crate path not allowed in surface` | `tokio::…` / `use` crate / reexport crudo en `.arita` |

(E0250 = perf profile; E024x = async; E0231 = unsafe.)

### 6. Oráculos (tras IMPL GO)

| Id | Path | Expect |
|----|------|--------|
| `deps-01-tokio-bridge` | `ejemplos/deps/01-tokio-bridge.arita` (+ `arita.toml`) | E2E async mínimo ya cubierto / build con `[deps] tokio` explícito → stdout fijado |
| `deps-neg-e0260` | `ejemplos/deps/neg/e0260-unknown-crate.arita` | **E0260** |
| `deps-neg-e0261` (opc.) | surface con path ilegal | **E0261** |

### 7. OUT

- deps que exijan unsafe/FFI **expuesto** al surface
- `use` / paths crates.io libres en `.arita`
- “pegar” crates.io sin whitelist
- demo keys / secrets en manifiesto
- reexport crudo del crate al dialecto
- Host crate con deps libres **detrás** de frontera (patrón borde ROADMAP) = **doc futuro**, no MVP surface

### 8. Patrón borde (nota, no IMPL v0)

Crate host Rust (deps libres) detrás de bridge ARITA → **ADR-033** (`HOST-BORDER-20260914`, propuesta DOC). ADR-029 v0 = whitelist+emit.

## Decisiones cerradas (2026-09-14 — Ingeniero GO)

1. **Solo `arita.toml` `[deps]`** v0. `deps { }` en `.arita` = OUT v0 / futuro.
2. Whitelist v0 = **tokio-only** (formaliza ADR-027; sin APIs surface nuevas). +1 crate = CUT posterior.
3. **E0260 / E0261 OK.**

## Consecuencias

- CUT IMPL autorizado: executor Ingeniero (stack completo, patrón ASYNC/PERF).
- Parser/Codegen: HOLD crates salvo review-only.
- ROADMAP 4c: este ADR **aceptada**.
- Ampliar whitelist = CUT DOC+IMPL, no drift silencioso.

## Checklist GO

- [x] Bridges / no path crate en surface OK
- [x] Whitelist v0 tokio-only OK
- [x] E0260/E0261 OK
- [x] Abiertas 1–3 **cerradas**
- [x] GO DOC → **aceptada** + IMPL autorizado
- [x] Oráculos `ejemplos/deps/` E2E + measure Lex **69/69** accepted

## IMPL evidence (`DEPS-V0-20260914`)

| Id | Path | Expect | Verdict |
|----|------|--------|---------|
| deps-01-tokio-bridge | `ejemplos/deps/01-tokio-bridge.arita` + `arita.toml` | E2E stdout `hi` / `deps-ok`; Cargo.toml tokio `=1.53.1` | accepted (Lex measure) |
| deps-neg-e0260 | `ejemplos/deps/neg/e0260-unknown-crate/` | **E0260** | accepted |
| deps-neg-e0261 | `ejemplos/deps/neg/e0261-crate-path.arita` | **E0261** | accepted |

Pins: surface never names `tokio::`; emit pin from whitelist when `[deps]` declared; async path without manifest still works (ADR-027).
