# ADR-272 — Core 0.7 slice 1: IO-PARSE-H1

- **Estado:** **CLOSED** Lex **733/733** (2026-09-26) · gate [`DOC/GATE-CORE07-IO-PARSE-H1-20260926.md`](../GATE-CORE07-IO-PARSE-H1-20260926.md)
- **CUT-ID:** `CORE-0.7-IO-PARSE-H1-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-271](271-core-0.7-pins.md) §0.1 slice 1 · HALLAZGOS H1
- **Prev:** Core **0.6 CLOSED** Lex **728/728** — este CUT **no** reabre 264–268
- **Prereq surface:** host `read_text`/`write_text` → `Result` (ADR-238) **ya IN**
- **HOLD:** I/O API nueva · Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · String.set · `?` / E0291 unwrap · reopen E0272/E0285

## Objetivo

Pin **E0340** anti-theater: tras `host.read_text` / `host.write_text` (Result), prohibido descartar `Err` o sustituir por lit vacío/`Default`/`unwrap_or` theater **sin** arm Err/fail. Oracles measure. Sin API host nueva.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Diag** | **E0340** `io result discarded` (texto canónico EN) |
| **Sujeto** | expresión `host.read_text` / `host.write_text` cuyo `Result` se ignora, se `unwrap_or`/`unwrap_or_else` a lit vacío/`""`/`0`, o match sin arm `Err` que falle |
| **OK** | `match` / `if let` con arm `Err` → fail/propagate explícito (print+exit tipado o return Err) **sin** inventar `?` |
| **Emit** | path happy: sin `.unwrap()`/`.expect(`/panic (241) |
| **OUT** | new `host.json_parse` · Path API · reabrir E0272 (Result genérico) / E0285 (checked+default) |
| **No reabre** | set/Map 0.6 · insert/`[]` collections |

## 1. Surface ejemplo

```text
// POS — Err visible
match host.read_text(path) {
  Ok(t) => print(t),
  Err(_) => print("read_failed")   // fail determinista (measure)
}

// NEG → E0340
let _ = host.read_text(path)              // discard
let t = host.read_text(path).unwrap_or("")  // default-as-success theater
match host.read_text(path) { Ok(t) => ..., Err(_) => "" }  // Err → lit vacío theater
```

`host.json_get_int` miss → Option: **fuera** de este slice (va H2 / E0341 slice 2) salvo que un parse Result host exista ya IN (no inventar).

## 2. Oracles

| Id | Expect |
|----|--------|
| `core07-io-h1-read-ok` | read Ok → stdout esperado → PASS |
| `core07-io-h1-read-err` | missing file → Err arm fail determinista → PASS (no panic) |
| `neg-core07-io-h1-discard` | discard Result → **E0340** |
| `neg-core07-io-h1-default` | unwrap_or/lit vacío tras read → **E0340** |
| `core07-io-h1-emit-ban` | emit grep: cero unwrap/expect/panic en path usuario (241) |

## 3. Pin E0340

**E0340** `io result discarded` — solo host IO `Result` (read/write) discard o default-as-success.
**No** reasignar E0272 / E0285 / E0315 / E0319.

## 4. Criterio CLOSED

Lex §2 verde; tick ADR-271 slice 1; HOLDs intactos. Siguiente: slice 2 CLI-ARGV-H2 ([ADR-273](273-core-cli-argv-h2-v0.md) GO-listo DOC).

## Checklist

- [x] Pins E0340 + oracles (GO-listo)
- [x] GO IMPL Orquestador / Ingeniero (post GO <person> «sigue»)
- [x] IMPL + Lex (Codegen Lex smoke 5/5; measure_pass:false → Measure)
- [x] Slice 2 → ADR-273 **CLOSED** Lex **739/739** (pins ya GO-listo)

## Cierre

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE07-IO-PARSE-H1-20260926.md`](../GATE-CORE07-IO-PARSE-H1-20260926.md)
- Lex measure **733/733** accepted
- Clippy workspace OK; Veyra companion REJECTED (RUSTSEC + rustfmt) **non-blocking**
- Siguiente (histórico): ADR-273 CLI-ARGV-H2 — **CLOSED** Lex **739/739** (vertical Core **0.7 CLOSED** vía 275 Lex **752/752**)
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair/I/O-new/String.set/`?` intactos
