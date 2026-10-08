# ADR-257 — Core 0.4 slice 4: SCENARIO-PKG

- **Estado:** **CLOSED** Lex **677/677** (CUT `CORE-0.4-SCENARIO-PKG-20260920`)
- **Gate:** [`../GATE-CORE04-SCENARIO-PKG-20260920.md`](../GATE-CORE04-SCENARIO-PKG-20260920.md)
- **Barra:** `arita measure` → **677/677 accepted**; scen-pkg-happy / lib-fn / workspace-build + neg E0331
- **CUT-ID:** `CORE-0.4-SCENARIO-PKG-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-254](254-core-0.4-pins.md) §0.1 slice 4
- **Prev:** slice 3 LIB-API (ADR-256) — IMPL/Measure en curso o CLOSED según Lex; este CUT no reabre 256
- **HOLD:** crates.io · `[]`/insert/Mutex · idle-kill · TLS/WS · repair · HTTP compose (fuera de scope 0.4)

## Objetivo

Scenarios/acceptance sobre **workspace lib+bin** (255/256): el bin ejercita la lib; measure firma sin theater.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Harness** | package `arita.toml` + lib `pub` + bin CLI (o bin test harness) |
| **Scenarios** | language-level `scenario`/`acceptance` (reusar 0.1/247 style) |
| **Mínimo** | ≥1 scenario llama API lib vía bin **o** scenario in-process si emit lo permite — **pin preferido:** bin CLI + stdout oracle |
| **OUT** | net/HTTP scenarios · crates.io · parallel Mutex |

## 1. Surface

Sin API nueva. Reusa `scenario` + `pub` lib + bin `main` + print/assert acceptance.

Ejemplo:

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
| `core04-scen-pkg-lib-fn` | output refleja `pub fn` |
| `neg-core04-scen-pkg-private` | scenario/bin no puede usar private (E0331) |
| `core04-scen-pkg-workspace-build` | measure build workspace verde |

## 3. Cierre (2026-09-20)

- Measure Lex **677/677** + Ingeniero GO (gate). Docs firma **CLOSED**.
- HOLDs intactos. Veyra companion REJECTED (`bytes` RUSTSEC) non-blocking.
- Siguiente: slice 5 REF-PKG (**GO**; cierra Core 0.4).

## Checklist

- [x] Pins scenarios lib+cli + oracles
- [x] IMPL + Lex **677/677** CLOSED
- [x] GO slice 5 REF-PKG
