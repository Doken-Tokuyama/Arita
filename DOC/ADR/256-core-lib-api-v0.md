# ADR-256 — Core 0.4 slice 3: LIB-API

- **Estado:** **CLOSED** Lex **673/673** (CUT `CORE-0.4-LIB-API-20260920`)
- **Gate:** [`../GATE-CORE04-LIB-API-20260920.md`](../GATE-CORE04-LIB-API-20260920.md)
- **Barra:** `arita measure` → **673/673 accepted**; lib-call-from-bin / pub-record / build + neg E0331 private
- **CUT-ID:** `CORE-0.4-LIB-API-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-254](254-core-0.4-pins.md) §0.1 slice 3
- **Prev:** slice 2 PACKAGE-MANIFEST **CLOSED** Lex **668/668** (ADR-255)
- **HOLD:** crates.io · `[]`/insert/Mutex · idle-kill · TLS/WS · repair · proc-macros · `pub use` reexport theater

## Objetivo

API de **librería** tipada exportada (`lib`) consumible desde `bin` del mismo workspace — fns/records públicos, `use pkg::…`.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Export** | ítems `pub` en módulo lib root (o `lib.arita` / `src/lib.arita` layout emit) |
| **IN** | `pub fn`, `pub record`, `use workspace_lib::item` desde bin |
| **Visibilidad** | default private; solo `pub` cruza crate boundary emit |
| **OUT** | generics avanzados · traits · `pub use` glob · dynamic load |

## 1. Surface

```text
// lib
pub record Point { x: Int, y: Int }
pub fn add(a: Int, b: Int) -> Int { a + b }

// bin
use demo_lib::add  // nombre package/lib según manifest 255
fn main() { print(add(1, 2)) }
```

Sin keyword nueva más allá de `pub` (si `pub` ya IN — reusar; si no, pin `pub` mínimo en este CUT).

## 2. Oracles

| Id | Expect |
|----|--------|
| `core04-lib-call-from-bin` | bin llama `pub fn` lib → output esperado |
| `core04-lib-pub-record` | bin construye/usa `pub record` |
| `neg-core04-lib-private` | bin usa ítem no-`pub` → diag estable (E0331 sugerido) |
| `core04-lib-build-workspace` | workspace lib+bin build verde |

## 3. Cierre (2026-09-20)

- Measure Lex **673/673** + Ingeniero GO (gate). Docs firma **CLOSED**.
- HOLDs intactos. Veyra companion REJECTED (`bytes` RUSTSEC) non-blocking.
- Siguiente: slice 4 SCENARIO-PKG (**GO**).

## Checklist

- [x] Pins pub lib→bin + oracles
- [x] IMPL Lex smoke (Codegen)
- [x] Measure Lex **673/673** + Ingeniero gate → **CLOSED**
- [x] GO slice 4 SCENARIO-PKG
