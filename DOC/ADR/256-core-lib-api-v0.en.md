Translation of `256-core-lib-api-v0.md`; the original is normative. / Traducción de `256-core-lib-api-v0.md`; el original es el normativo.

# ADR-256 — Core 0.4 slice 3: LIB-API

- **Estado:** **CLOSED** Lex **673/673** (CUT `CORE-0.4-LIB-API-20260920`)
- **Gate:** `../GATE-CORE04-LIB-API-20260920.md` (not in the public export / no incluido en el export público)
- **Barra:** `arita measure` → **673/673 accepted**; lib-call-from-bin / pub-record / build + neg E0331 private
- **CUT-ID:** `CORE-0.4-LIB-API-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-254](254-core-0.4-pins.md) §0.1 slice 3
- **Prev:** slice 2 PACKAGE-MANIFEST **CLOSED** Lex **668/668** (ADR-255)
- **HOLD:** crates.io · `[]`/insert/Mutex · idle-kill · TLS/WS · repair · proc-macros · `pub use` reexport theater

## Objective

Typed exported **library** API (`lib`) consumable from the same workspace’s `bin` — public fns/records, `use pkg::…`.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Export** | `pub` items in lib root module (or `lib.arita` / `src/lib.arita` emit layout) |
| **IN** | `pub fn`, `pub record`, `use workspace_lib::item` from bin |
| **Visibilidad** | default private; only `pub` crosses the emit crate boundary |
| **OUT** | advanced generics · traits · glob `pub use` · dynamic load |

## 1. Surface

```text
// lib
pub record Point { x: Int, y: Int }
pub fn add(a: Int, b: Int) -> Int { a + b }

// bin
use demo_lib::add  // package/lib name per manifest 255
fn main() { print(add(1, 2)) }
```

No new keyword beyond `pub` (if `pub` already IN — reuse; if not, pin minimum `pub` in this CUT).

## 2. Oracles

| Id | Expect |
|----|--------|
| `core04-lib-call-from-bin` | bin calls lib `pub fn` → expected output |
| `core04-lib-pub-record` | bin builds/uses `pub record` |
| `neg-core04-lib-private` | bin uses non-`pub` item → stable diag (suggested E0331) |
| `core04-lib-build-workspace` | green lib+bin workspace build |

## 3. Close (2026-09-20)

- Measure Lex **673/673** + Engineer GO (gate). Docs signs **CLOSED**.
- HOLDs intact. Veyra companion REJECTED (`bytes` RUSTSEC) non-blocking.
- Next: slice 4 SCENARIO-PKG (**GO**).

## Checklist

- [x] pub lib→bin pins + oracles
- [x] IMPL Lex smoke (Codegen)
- [x] Measure Lex **673/673** + Engineer gate → **CLOSED**
- [x] GO slice 4 SCENARIO-PKG
