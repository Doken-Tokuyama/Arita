Translation of `033-host-border-v0.md`; the original is normative. / Traducción de `033-host-border-v0.md`; el original es el normativo.

# ADR-033 — Host-border / bridge pattern v0 (Phase 4c)

- **Estado:** **aceptada** (DOC-only; sin IMPL en este CUT)
- **CUT-ID:** `HOST-BORDER-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (draft/align) + Ingeniero Rust (**GO DOC-only**)
- **Relacionados:** ADR-029 (deps whitelist + bridges), ADR-022 / E0231 (safe-only), ADR-027 (tokio bridge), E0260/E0261, `DOC/PERF-HOST-BORDER.md` (borde **perf**, distinto), ROADMAP 4c “Patrón borde”
- **Gobernanza:** **aceptada DOC-only**. Hello bridge E2E = **CUT IMPL aparte**. Sin theater. Nombre = **ARITA**.
- **Bar (when there is IMPL):** parse→emit→execute + clippy; skip ≠ PASS. This ADR **does not** yet require a measure oracle.

## Context

ADR-029 formalized deps **whitelist** in `arita.toml` + bridges in the **generated** crate (tokio-only). ROADMAP 4c still asks to document the **border pattern**: a host Rust crate with free deps **behind** a boundary; ARITA only sees bridge APIs.

Distinct from `PERF-HOST-BORDER.md` (tuning CPU/LTO/tools). Here = **integration architecture** host↔ARITA.

Baseline: measure **75/75**; LSP landed; deps 029 verified.

## Decision (DOC MVP)

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

1. **Surface ARITA:** only dialect + declared bridges (029). No paths `crate::`, without `unsafe`, without FFI en `.arita` (E0261 / E0231).
2. **Emit crate:** Cargo pinneado; `#![forbid(unsafe_code)]` en code de **usuario** generado; may depender de host via path/workspace **si** el bridge lo expone de forma safe.
3. **Host crate:** Rust free (deps, incluso `unsafe` interno) **always behind** de safe-callable bridge API from the emit. It is not re-exported to the surface.

### 2. Relation to codes existentes

| Code / ADR | Rol en host-border |
|--------------|--------------------|
| **E0260** | dep no whitelist en `arita.toml` |
| **E0261** | path / `use` de crate externo en `.arita` |
| **E0231** / ADR-022 | unsafe/FFI en surface o emit usuario |
| ADR-029 | whitelist + bridge ids en manifiesto |

Host-border **no** inventa E0xxx nuevos en v0 DOC.

### 3. IN (documentation)

- Diagrama de capas + reglas “surface never nombra host/crate”.
- Checklist for add un bridge: (a) API ARITA tipada, (b) impl en emit/host, (c) entrada whitelist 029 si hay dep, (d) oracle E2E **future**.
- Nota: tokio async-runtime ya es bridge **in-emit** (027/029), no requiere host crate.

### 4. OUT (v0)

- FFI / `extern "C"` **expuesto** al dialecto `.arita`
- Reexport crudo de tipos of the host al surface
- Permitir `unsafe` en `.arita` “because el host lo necesita”
- Demo keys / secrets en manifiestos
- Oracle measure mandatory en this CUT DOC (evita theater); IMPL ejemplo = CUT aparte
- Meter host crates en `[deps]` (usar `[host-bridges]`)

### 5. Future measurement (no v0)

| Id (reservado) | Expect |
|----------------|--------|
| `host-01-bridge-call` | `.arita` llama fn bridge → stdout fijado; host puede usar dep no-whitelist **only** en su crate |
| neg | E0261 si `.arita` intenta path of the host/crate |

## Decisiones cerradas (2026-09-14 — Ingeniero GO DOC-only)

1. **Layout:** workspace canonical `crates/arita-host-<name>`. Path-dep externo = variante documentada, **no** default v0.
2. **Declaration:** section **`[host-bridges]`** en `arita.toml` (do not mix with `[deps]` whitelist crates.io).
3. **DOC-only** this CUT — hello bridge E2E = CUT IMPL aparte (anti-theater).

### Forma manifiesto (sketch)

```toml
[deps]
tokio = { bridge = "async-runtime" }   # ADR-029 crates.io whitelist

[host-bridges]
demo = { crate = "arita-host-demo", path = "crates/arita-host-demo" }
```

## Consequences

- ROADMAP 4c “Border pattern” → **accepted DOC** (this ADR).
- No crates / no measure delta en this CUT.
- Ampliar surface bridge o hello E2E = CUT IMPL nuevo.

## Checklist GO

- [x] Capas host/emit/surface OK
- [x] E0260/E0261/E0231 roles OK
- [x] DOC-only v0 (without oracle theater) OK
- [x] Abiertas 1–3 **cerradas**
- [x] GO → **accepted** DOC-only
- [x] CUT IMPL hello → **ADR-035** (`HOST-BORDER-HELLO-20260914`)
