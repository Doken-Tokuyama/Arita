# ADR-033 — Host-border / bridge pattern v0 (Fase 4c)

- **Estado:** **aceptada** (DOC-only; sin IMPL en este CUT)
- **CUT-ID:** `HOST-BORDER-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (draft/align) + Ingeniero Rust (**GO DOC-only**)
- **Relacionados:** ADR-029 (deps whitelist + bridges), ADR-022 / E0231 (safe-only), ADR-027 (tokio bridge), E0260/E0261, `DOC/PERF-HOST-BORDER.md` (borde **perf**, distinto), ROADMAP 4c “Patrón borde”
- **Gobernanza:** **aceptada DOC-only**. Hello bridge E2E = **CUT IMPL aparte**. Sin theater. Nombre = **ARITA**.
- **Barra (cuando haya IMPL):** parse→emit→execute + clippy; skip ≠ PASS. Este ADR **no** exige oráculo measure aún.

## Contexto

ADR-029 formalizó deps **whitelist** en `arita.toml` + bridges en el crate **generado** (tokio-only). ROADMAP 4c aún pide documentar el **patrón borde**: un crate host Rust con deps libres **detrás** de una frontera; ARITA solo ve APIs bridge.

Distinto de `PERF-HOST-BORDER.md` (tuning CPU/LTO/tools). Aquí = **arquitectura de integración** host↔ARITA.

Baseline: measure **75/75**; LSP landed; deps 029 verified.

## Decisión (DOC MVP)

### 1. Capas

```text
.arita  ──(surface: tipos/fns bridge whitelist)──►  emit crate ARITA
                                                         │
                                                         │ llama solo APIs bridge
                                                         ▼
                                              host crate Rust (opcional)
                                              deps crates.io libres / unsafe
                                              acotado al host — NUNCA en .arita
```

1. **Surface ARITA:** solo dialecto + bridges declarados (029). Sin paths `crate::`, sin `unsafe`, sin FFI en `.arita` (E0261 / E0231).
2. **Emit crate:** Cargo pinneado; `#![forbid(unsafe_code)]` en código de **usuario** generado; puede depender de host vía path/workspace **si** el bridge lo expone de forma safe.
3. **Host crate:** Rust libre (deps, incluso `unsafe` interno) **siempre detrás** de API bridge safe-callable desde el emit. No se reexporta al surface.

### 2. Relación con códigos existentes

| Código / ADR | Rol en host-border |
|--------------|--------------------|
| **E0260** | dep no whitelist en `arita.toml` |
| **E0261** | path / `use` de crate externo en `.arita` |
| **E0231** / ADR-022 | unsafe/FFI en surface o emit usuario |
| ADR-029 | whitelist + bridge ids en manifiesto |

Host-border **no** inventa E0xxx nuevos en v0 DOC.

### 3. IN (documentación)

- Diagrama de capas + reglas “surface nunca nombra host/crate”.
- Checklist para añadir un bridge: (a) API ARITA tipada, (b) impl en emit/host, (c) entrada whitelist 029 si hay dep, (d) oráculo E2E **futuro**.
- Nota: tokio async-runtime ya es bridge **in-emit** (027/029), no requiere host crate.

### 4. OUT (v0)

- FFI / `extern "C"` **expuesto** al dialecto `.arita`
- Reexport crudo de tipos del host al surface
- Permitir `unsafe` en `.arita` “porque el host lo necesita”
- Demo keys / secrets en manifiestos
- Oráculo measure obligatorio en este CUT DOC (evita theater); IMPL ejemplo = CUT aparte
- Meter host crates en `[deps]` (usar `[host-bridges]`)

### 5. Medición futura (no v0)

| Id (reservado) | Expect |
|----------------|--------|
| `host-01-bridge-call` | `.arita` llama fn bridge → stdout fijado; host puede usar dep no-whitelist **solo** en su crate |
| neg | E0261 si `.arita` intenta path del host/crate |

## Decisiones cerradas (2026-09-14 — Ingeniero GO DOC-only)

1. **Layout:** workspace canónico `crates/arita-host-<name>`. Path-dep externo = variante documentada, **no** default v0.
2. **Declaración:** sección **`[host-bridges]`** en `arita.toml` (no mezclar con `[deps]` whitelist crates.io).
3. **DOC-only** este CUT — hello bridge E2E = CUT IMPL aparte (anti-theater).

### Forma manifiesto (sketch)

```toml
[deps]
tokio = { bridge = "async-runtime" }   # ADR-029 crates.io whitelist

[host-bridges]
demo = { crate = "arita-host-demo", path = "crates/arita-host-demo" }
```

## Consecuencias

- ROADMAP 4c “Patrón borde” → **aceptada DOC** (este ADR).
- No crates / no measure delta en este CUT.
- Ampliar surface bridge o hello E2E = CUT IMPL nuevo.

## Checklist GO

- [x] Capas host/emit/surface OK
- [x] E0260/E0261/E0231 roles OK
- [x] DOC-only v0 (sin oráculo theater) OK
- [x] Abiertas 1–3 **cerradas**
- [x] GO → **aceptada** DOC-only
- [x] CUT IMPL hello → **ADR-035** (`HOST-BORDER-HELLO-20260914`)
