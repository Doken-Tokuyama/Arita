# ADR-274 — Core 0.7 slice 3: SCENARIO-IO

- **Estado:** **CLOSED** Lex **744/744** (2026-09-26) · gate [`DOC/GATE-CORE07-SCENARIO-IO-20260926.md`](../GATE-CORE07-SCENARIO-IO-20260926.md)
- **CUT-ID:** `CORE-0.7-SCENARIO-IO-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-271](271-core-0.7-pins.md) §0.1 slice 3
- **Prev:** slice 2 CLI-ARGV-H2 (ADR-273) — **CLOSED** Lex **739/739**; este CUT **no** reabre 272/273
- **Prereq surface (al GO IMPL):** E0340 (272) + E0341 (273) en tree + host ADR-238
- **HOLD:** I/O API nueva · Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · String.set · `?` · HTTP scenarios · reopen 0.6 set/Map

## Objetivo

Scenarios/acceptance que ejercitan **H1+H2 juntos** sobre host 238 (happy read+args/JSON · Err/None fail · neg E0340/E0341). Measure firma sin theater. **Sin API nueva.** Puente hacia slice 4 REF-IO.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Harness** | `scenario`/`acceptance` language-level (reusar 0.1 / 267 / 262 style) |
| **Entrada** | path file + args/JSON **requeridos** vía host 238 only |
| **Mínimo** | ≥3 scenarios: happy IO+CLI/JSON · read Err y/o required miss fail · ≥1 neg (E0340 **o** E0341) |
| **Preferido** | bin CLI + stdout oracle **o** in-process scenario si emit lo permite |
| **OUT** | new host APIs · Mutex · net/HTTP · IndexMut · crates.io |

## 1. Surface

Sin keyword nueva. Reusa `scenario` + `host.read_text`/`write_text` Result + `cli_arg`/`json_get_int` Option + print/assert.

```text
scenario io_happy {
  // cli_arg Some + read Ok (+ json_get Some) → stdout fijo
  acceptance { /* measure */ }
}
scenario io_fail {
  // read Err y/o required miss None → fail determinista; sin panic
  acceptance { /* measure */ }
}
// neg: discard Result → E0340  OR  unwrap_or tras cli_arg required → E0341
```

## 2. Oracles (Measure) — éxito medible

| Id | Expect |
|----|--------|
| `core07-scen-io-happy` | args/JSON + read Ok → stdout determinista → **PASS** |
| `core07-scen-io-fail` | read Err y/o required miss → fail path → **PASS** (no panic) |
| `neg-core07-scen-io-h1` | discard/default Result → **E0340** (si no cubierto solo en 272) |
| `neg-core07-scen-io-h2` | required miss-as-ok / argv Index → **E0341** (si no cubierto solo en 273) |
| `core07-scen-io-build` | measure build verde |

**Anti-theater:** skip ≠ PASS. No inventar PASS.

## 3. Qué NO tocar

| Prohibido | Motivo |
|-----------|--------|
| I/O bindings nuevos · Mutex · IndexMut · idle · TLS/WS · crates.io · repair | HOLD |
| GO IMPL antes CLOSED 272+273 + OK Ingeniero | gate Orquestador |
| Reabrir E0340/E0341 surface pins | ya en 272/273 |
| Declarar CLOSED Core 0.7 | solo slice 4 REF-IO |

## 4. Criterio CLOSED

Lex §2 verde; tick ADR-271 slice 3; HOLDs §3 intactos. Siguiente: slice 4 REF-IO (ADR-275).

## Checklist

- [x] Pins scenarios H1+H2 + oracles (GO-listo)
- [x] GO IMPL Orquestador / Ingeniero (post-CLOSED 272+273)
- [x] IMPL + Lex **744/744** CLOSED (Measure)
- [x] Slice 4 → ADR-275 **CLOSED** Lex **752/752** = Core **0.7 CLOSED**

## Cierre

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE07-SCENARIO-IO-20260926.md`](../GATE-CORE07-SCENARIO-IO-20260926.md)
- Lex measure **744/744** accepted
- Clippy workspace OK; Veyra companion REJECTED (RUSTSEC + rustfmt) **non-blocking**
- Siguiente (histórico): ADR-275 REF-IO — **CLOSED** Lex **752/752** = **Core 0.7 CLOSED**
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair/I/O-new/String.set/`?` intactos
