# ADR-258 — Core 0.4 slice 5: REF-PKG

- **Estado:** **CLOSED** Lex **683/683** (2026-09-20) · Core **0.4 CLOSED** · siguiente: Core **0.5** [ADR-259](259-core-0.5-pins.md) (GO <person>)
- **CUT-ID:** `CORE-0.4-REF-PKG-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-254](254-core-0.4-pins.md) §0.1 slice 5 · §2 programa ref
- **Prev:** slice 4 SCENARIO-PKG **CLOSED** Lex **677/677** (ADR-257)
- **Cierra:** vertical Core **0.4** (packages / GP library)
- **HOLD:** crates.io · `[]`/insert/Mutex · idle-kill · TLS/WS · repair · HTTP compose (fuera de scope) · proc-macros · `pub use` theater

## Objetivo

Ref **no trivial** `arita-ref-pkg-lib` + evidence + Lex barra que **cierra Core 0.4**: workspace lib+bin, CLI consume `pub` API, scenarios verdes, evidence hash.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Nombre** | `arita-ref-pkg-lib` (`ejemplos/core04/ref-pkg-lib/` o Lex equiv.) |
| **Layout** | `arita.toml` → emit Cargo workspace `{lib,bin}` (ADR-255) |
| **Lib** | ≥2 `pub` ítems (`pub fn` + `pub record` o equivalente útil) — dominio **puro** (string/JSON-mini/pipeline); **sin** HTTP obligatorio |
| **CLI** | lee args y/o file → llama lib → stdout determinista |
| **Scenarios** | ≥2 acceptance (happy CLI+lib + un path error/edge o neg private vía suite 256/257) |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`) |
| **OUT** | crates.io · Mutex · `[]` · net daemon · generics avanzados |

## 1. Surface

Solo APIs ya IN (254–257). Sin std nueva. Reusa `pub`, multi-module, manifest, scenarios.

Ejemplo orientativo (no theater):

```text
// lib: pub record Config { … }; pub fn process(input: String) -> Result<String, …>
// bin: parse args → process → print
scenario ref_pkg_happy { acceptance { /* stdout esperado */ } }
scenario ref_pkg_edge { acceptance { /* Result Err path o empty input */ } }
```

## 2. Oracles — gate CLOSED Core 0.4

| Id | Expect |
|----|--------|
| `core04-ref-lib-build` | workspace lib+bin build verde |
| `core04-ref-cli-happy` | CLI → stdout esperado vía lib |
| `core04-ref-cli-edge` | edge/error path estable (no panic) |
| `core04-ref-scenario-pkg` | ≥2 scenarios PASS (257) |
| `core04-ref-evidence` | evidence JSON + hash estable |
| `neg-core04-ref-private` | private no usable desde bin (E0331) — puede vivir en suite 256 |

skip ≠ PASS. Ingeniero fija N/N Lex.

## 3. Criterio CLOSED (slice 5 = Core 0.4 CLOSED)

Lex §2 verde; ROADMAP/ADR-254 Core 0.4 **CLOSED**; HOLDs **no** unpark.

## 4. Post-0.4 (no este CUT)

Siguiente vertical: Core **0.5** fallible collections ([ADR-259](259-core-0.5-pins.md)) — GO <person> 2026-09-20. Mutex/idle/TLS/WS/crates.io/repair siguen HOLD.

## Checklist

- [x] Pins ref + oracles + evidence + CLOSE Core 0.4
- [x] IMPL + Lex Core 0.4 CLOSED **683/683**
