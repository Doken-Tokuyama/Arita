# ADR-273 — Core 0.7 slice 2: CLI-ARGV-H2

- **Estado:** **CLOSED** Lex **739/739** (2026-09-26 remasure) · gate [`DOC/GATE-CORE07-CLI-ARGV-H2-20260926.md`](../GATE-CORE07-CLI-ARGV-H2-20260926.md) · FIX E0341-CORE06
- **CUT-ID:** `CORE-0.7-CLI-ARGV-H2-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-271](271-core-0.7-pins.md) §0.1 slice 2 · HALLAZGOS H2
- **Prev:** slice 1 IO-PARSE-H1 (ADR-272) **CLOSED** Lex **733/733**; este CUT **no** reabre 272
- **Prereq surface (al GO IMPL):** host `cli_arg` → `Option` + `json_get_int` → `Option` (ADR-238) **ya IN** + E0340 land (272 CLOSED)
- **HOLD:** I/O API nueva · `args[i]` / Index argv · IndexMut-assign · Mutex · idle · TLS/WS · crates.io · repair · String.set · `?` / E0291 · reopen E0287 (Vec get+default) / E0340

## Objetivo

Pin **E0341** anti-theater: argv/JSON **requerido** no puede teatro-éxito ante miss (`None` → lit default / vacío sin fail). Preferir `host.cli_arg` / `json_get_int` + match None→fail. **Prohibido** indexar argv con `[]`. Sin API host nueva.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Diag** | **E0341** `required arg miss as ok` (texto canónico EN) |
| **Sujeto** | (a) `host.cli_arg(i)` / `host.json_get_int(doc,key)` en path **required**: `None` → lit default/`""`/`0` / `unwrap_or` sin fail; (b) indexación argv `args[i]` / `[]` sobre args |
| **OK** | `match` / `if let` None → usage/fail determinista; optional args explícitos DOC fuera de E0341 |
| **Emit** | solo `cli_arg`/`json_get_*` Option path — **cero** `Index`/`args[i]` panic |
| **OUT** | new argv parser API · IndexMut · reopen E0287 (colecciones) · E0340 (IO Result) |
| **No reabre** | SET/MAP 0.6 · IO-PARSE-H1 (272) |

## 1. Surface ejemplo

```text
// POS — required arg / key
match host.cli_arg(1) {
  Some(p) => ...,
  None => print("usage")   // fail determinista
}
match host.json_get_int(doc, "n") {
  Some(n) => ...,
  None => print("missing_n")
}

// NEG → E0341
let p = host.cli_arg(1).unwrap_or("")     // miss-as-ok theater
let n = host.json_get_int(doc, "n").unwrap_or(0)
// args[1] / argv[i] → E0341 (o E0310 index-ban si ya aplica; pin preferido E0341 en CLI)
```

Optional flags (miss = default **documentado** como optional) **fuera** de E0341 — DOC Measure marca required vs optional en oracle.

## 2. Oracles

| Id | Expect |
|----|--------|
| `core07-io-h2-arg-ok` | cli_arg Some → stdout esperado → PASS |
| `core07-io-h2-arg-miss` | required miss → None arm fail determinista → PASS |
| `core07-io-h2-json-miss` | required json_get_int None → fail determinista → PASS |
| `neg-core07-io-h2-default` | unwrap_or/lit tras cli_arg/json_get required → **E0341** |
| `neg-core07-io-h2-index` | `args[i]` / Index argv → **E0341** (o diag index-ban estable documentada) |
| `core07-io-h2-emit-ban` | emit grep: cero Index/argv`[]` / unwrap theater |

## 3. Pin E0341

**E0341** `required arg miss as ok` — solo host CLI/JSON **required** Option miss-as-ok o argv Index.
**No** reasignar E0287 / E0340 / E0310 (si E0310 ya cubre index ban genérico, oracle `neg-core07-io-h2-index` puede defer a E0310 — documentar en gate; preferencia slice = E0341 en sujeto CLI).

## 4. Qué NO tocar

| Prohibido | Motivo |
|-----------|--------|
| I/O bindings nuevos | HOLD H1/H2 *new* |
| Mutex / IndexMut-assign / idle / TLS/WS / crates.io / repair | HOLD global |
| GO IMPL antes CLOSED 272 + OK Ingeniero | gate Orquestador |
| Reabrir E0340 / E0272 / E0285 / E0287 | anti-colisión |

## 5. Criterio CLOSED

Lex §2 verde; tick ADR-271 slice 2; HOLDs §4 intactos. Siguiente: slice 3 SCENARIO-IO ([ADR-274](274-core-scenario-io-v0.md) GO-listo) → slice 4 [ADR-275](275-core-ref-io-v0.md).

## Checklist

- [x] Pins E0341 + oracles + qué NO tocar (GO-listo)
- [x] GO IMPL Orquestador / Ingeniero (post-CLOSED 272)
- [x] IMPL + Lex **739/739** CLOSED (remeasure)
- [x] Slice 3 GO (ADR-274) IMPL → ADR-274 (pins ya GO-listo)

## Cierre

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE07-CLI-ARGV-H2-20260926.md`](../GATE-CORE07-CLI-ARGV-H2-20260926.md) · remasure post FIX E0341-CORE06
- Lex measure **739/739** accepted
- Clippy workspace OK; Veyra companion REJECTED (RUSTSEC + rustfmt) **non-blocking**
- Siguiente (histórico): ADR-274 SCENARIO-IO — **CLOSED** Lex **744/744** (vertical Core **0.7 CLOSED** vía 275 Lex **752/752**)
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair/I/O-new/String.set/`?` intactos · E0341 intacto
