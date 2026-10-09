Translation of `257-core-scenario-pkg-v0.md`; the original is normative. / Traducción de `257-core-scenario-pkg-v0.md`; el original es el normativo.

# ADR-257 — Core 0.4 slice 4: SCENARIO-PKG

- **Estado:** **CLOSED** Lex **677/677** (CUT `CORE-0.4-SCENARIO-PKG-20260920`)
- **Gate:** `../GATE-CORE04-SCENARIO-PKG-20260920.md` (not in the public export / no incluido en el export público)
- **Barra:** `arita measure` → **677/677 accepted**; scen-pkg-happy / lib-fn / workspace-build + neg E0331
- **CUT-ID:** `CORE-0.4-SCENARIO-PKG-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-254](254-core-0.4-pins.md) §0.1 slice 4
- **Prev:** slice 3 LIB-API (ADR-256) — IMPL/Measure en curso o CLOSED según Lex; este CUT no reabre 256
- **HOLD:** crates.io · `[]`/insert/Mutex · idle-kill · TLS/WS · repair · HTTP compose (fuera de scope 0.4)

## Objective

Scenarios/acceptance on a **lib+bin workspace** (255/256): the bin exercises the lib; measure signs without theater.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Harness** | `arita.toml` package + `pub` lib + CLI bin (or bin test harness) |
| **Scenarios** | language-level `scenario`/`acceptance` (reuse 0.1/247 style) |
| **Minimum** | ≥1 scenario calls lib API via bin **or** in-process scenario if emit allows — **preferred pin:** CLI bin + stdout oracle |
| **OUT** | net/HTTP scenarios · crates.io · parallel Mutex |

## 1. Surface

No new API. Reuses `scenario` + `pub` lib + bin `main` + print/assert acceptance.

Example:

```text
// lib: pub fn double(x: Int) -> Int
// bin: print(double(21))  → 42
scenario pkg_double {
  acceptance { /* expect stdout 42 via measure */ }
}
```

## 2. Oracles

| Id | Expect |
|----|--------|
| `core04-scen-pkg-happy` | bin+lib scenario PASS |
| `core04-scen-pkg-lib-fn` | output reflects `pub fn` |
| `neg-core04-scen-pkg-private` | scenario/bin cannot use private (E0331) |
| `core04-scen-pkg-workspace-build` | green measure workspace build |

## 3. Close (2026-09-20)

- Measure Lex **677/677** + Engineer GO (gate). Docs signs **CLOSED**.
- HOLDs intact. Veyra companion REJECTED (`bytes` RUSTSEC) non-blocking.
- Next: slice 5 REF-PKG (**GO**; closes Core 0.4).

## Checklist

- [x] lib+cli scenario pins + oracles
- [x] IMPL + Lex **677/677** CLOSED
- [x] GO slice 5 REF-PKG
