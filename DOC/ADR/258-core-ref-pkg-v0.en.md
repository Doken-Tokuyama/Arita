Translation of `258-core-ref-pkg-v0.md`; the original is normative. / Traducción de `258-core-ref-pkg-v0.md`; el original es el normativo.

# ADR-258 — Core 0.4 slice 5: REF-PKG

- **Estado:** **CLOSED** Lex **683/683** (2026-09-20) · Core **0.4 CLOSED** · siguiente: Core **0.5** [ADR-259](259-core-0.5-pins.md) (GO <person>)
- **CUT-ID:** `CORE-0.4-REF-PKG-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-254](254-core-0.4-pins.md) §0.1 slice 5 · §2 programa ref
- **Prev:** slice 4 SCENARIO-PKG **CLOSED** Lex **677/677** (ADR-257)
- **Cierra:** vertical Core **0.4** (packages / GP library)
- **HOLD:** crates.io · `[]`/insert/Mutex · idle-kill · TLS/WS · repair · HTTP compose (fuera de scope) · proc-macros · `pub use` theater

## Objective

Non-trivial ref `arita-ref-pkg-lib` + evidence + Lex bar that **closes Core 0.4**: lib+bin workspace, CLI consumes `pub` API, green scenarios, evidence hash.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Nombre** | `arita-ref-pkg-lib` (`ejemplos/core04/ref-pkg-lib/` or Lex equiv.) |
| **Layout** | `arita.toml` → emit Cargo workspace `{lib,bin}` (ADR-255) |
| **Lib** | ≥2 `pub` items (`pub fn` + `pub record` or useful equivalent) — **pure** domain (string/mini-JSON/pipeline); **no** mandatory HTTP |
| **CLI** | reads args and/or file → calls lib → deterministic stdout |
| **Scenarios** | ≥2 acceptance (happy CLI+lib + one error/edge path or private neg via suite 256/257) |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`) |
| **OUT** | crates.io · Mutex · `[]` · net daemon · advanced generics |

## 1. Surface

Only already-IN APIs (254–257). No new std. Reuses `pub`, multi-module, manifest, scenarios.

Orientative example (no theater):

```text
// lib: pub record Config { … }; pub fn process(input: String) -> Result<String, …>
// bin: parse args → process → print
scenario ref_pkg_happy { acceptance { /* stdout esperado */ } }
scenario ref_pkg_edge { acceptance { /* Result Err path o empty input */ } }
```

## 2. Oracles — Core 0.4 CLOSED gate

| Id | Expect |
|----|--------|
| `core04-ref-lib-build` | green lib+bin workspace build |
| `core04-ref-cli-happy` | CLI → expected stdout via lib |
| `core04-ref-cli-edge` | stable edge/error path (no panic) |
| `core04-ref-scenario-pkg` | ≥2 scenarios PASS (257) |
| `core04-ref-evidence` | evidence JSON + stable hash |
| `neg-core04-ref-private` | private unusable from bin (E0331) — may live in suite 256 |

skip ≠ PASS. Engineer fixes Lex N/N.

## 3. CLOSED criterion (slice 5 = Core 0.4 CLOSED)

Lex §2 green; ROADMAP/ADR-254 Core 0.4 **CLOSED**; HOLDs **not** unparked.

## 4. Post-0.4 (not this CUT)

Next vertical: Core **0.5** fallible collections ([ADR-259](259-core-0.5-pins.en.md)) — GO <person> 2026-09-20. Mutex/idle/TLS/WS/crates.io/repair stay HOLD.

## Checklist

- [x] Ref pins + oracles + evidence + CLOSE Core 0.4
- [x] IMPL + Lex Core 0.4 CLOSED **683/683**
