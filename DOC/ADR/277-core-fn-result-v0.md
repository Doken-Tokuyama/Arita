# ADR-277 — Core 0.8 slice 1: FN-RESULT

- **Estado:** **CLOSED** Lex **758/758** (2026-09-26) · gate [`DOC/GATE-CORE08-FN-RESULT-20260926.md`](../GATE-CORE08-FN-RESULT-20260926.md)
- **CUT-ID:** `CORE-0.8-FN-RESULT-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero (OK 2026-09-26) · Orquestador (asigna Codegen/Measure)
- **Padre:** [ADR-276](276-core-0.8-pins.md) §0.1 slice 1 · HALLAZGOS H3
- **Prev:** Core **0.7 CLOSED** Lex **752/752** — este CUT **no** reabre 271–275
- **Prereq surface:** `Result<T,E>` match + Ok/Err (ADR-047) **ya IN** · `fn main() -> Io<()>` **ya IN**
- **HOLD:** Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · I/O-new · String.set · threads · `?` (**slice 2**) · surface unwrap/expect (241) · Option? · async Result · reopen E0272/E0340/E0341/E0291

## Objetivo

Unpark **`fn name(...) -> Result<T,E>`** (era OUT ADR-047 para return usable en propagate). Surface: fn→Result + **match** propagate **sin** `?` aún. Oracles: Result-returning fn works; neg invent unwrap / bad return theater → **E0342**. Sin API host nueva. Sin `?` (va slice 2).

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **IN** | `fn name(params) -> Result<T,E> { … }` con `T,E` alineados ADR-047; body return `Ok`/`Err` o match propagate |
| **Keep** | `fn main() -> Io<()>` sin cambio; main **no** exige Result |
| **Diag** | **E0342** `result return type required` (texto canónico EN) — theater que finge propagate sin return Result (p.ej. claim Err path sin tipo Result; return type mismatch theater) |
| **OK** | helper `fn → Result` + caller `match` Ok/Err; emit Rust `Result` sin unwrap |
| **Emit** | path happy/fail: **cero** `.unwrap()`/`.expect(`/panic (241) |
| **OUT** | `?` (slice 2) · invent surface `unwrap`/`expect` method · async Result · `Option?` · `?` in Io main |
| **No reabre** | E0272 / E0340 / E0341 / E0291 · Core 0.7 H1/H2 |

## 1. Surface ejemplo

```text
// POS — fn → Result + match propagate (sin ? aún)
fn load(path: String) -> Result<String, Int> {
  match host.read_text(path) {
    Ok(t) => Ok(t),
    Err(_) => Err(1)
  }
}

fn main() -> Io<()> {
  match load("a.txt") {
    Ok(t) => print(t),
    Err(_) => print("load_failed")
  }
}

// NEG → E0342 (bad return theater) — ejemplos Measure
// fn load(...) -> Int { Err(1) }           // Err sin Result return
// fn load(...) -> Result<String, Int> { 0 } // return lit no Result
// invent: r.unwrap() / r.expect("x")         // surface method → reject (+ emit-ban 241)
```

## 2. Oracles

| Id | Expect |
|----|--------|
| `core08-fn-result-ok` | helper `fn → Result` Ok path → stdout esperado → PASS |
| `core08-fn-result-err` | helper Err → caller match fail determinista → PASS (no panic) |
| `core08-fn-result-main-io` | `fn main() -> Io<()>` sigue OK llamando helper Result vía match |
| `neg-core08-fn-result-theater` | bad return / Err sin Result type → **E0342** |
| `neg-core08-fn-result-unwrap` | invent surface unwrap/expect method → reject / diag estable (≠ E0291 reopen; documentar gate) |
| `core08-fn-result-emit-ban` | emit grep: cero unwrap/expect/panic en path usuario (241) |

## 3. Pin E0342

**E0342** `result return type required` — theater de propagate/return sin `Result` tipado donde el pin exige `fn → Result`, o return payload incompatible con Ok/Err.
**No** reasignar E0272 / E0340 / E0341 / E0291 / E0270 / E0271.

**Nota soft (Ingeniero):** **E0342** solo theater pinneado en este ADR — **no** reetiquetar type-errors genéricos ya cubiertos (**E0203**/etc.).

## 4. Qué NO tocar

| Prohibido | Motivo |
|-----------|--------|
| `?` desugar | slice 2 ADR-278 |
| Mutex / IndexMut / idle / TLS/WS / crates.io / repair / I/O-new / String.set / threads | HOLD global |
| Surface unwrap/expect | emit-ban 241 HOLD |
| GO IMPL autorizado; IMPL/Lex pendientes | coordinación |
| Reabrir E0272 / E0340 / E0341 | anti-colisión |
| Inventar PASS / Lex N/N | DOC only |

## 5. Criterio CLOSED

Lex §2 verde; tick ADR-276 slice 1; HOLDs §4 intactos. Siguiente: slice 2 QMARK ([ADR-278](278-core-question-mark-v0.md)).

## Checklist

- [x] Pins `fn → Result` + E0342 + oracles (GO-listo DOC)
- [x] GO IMPL autorizado — Ingeniero OK 2026-09-26; Orquestador asigna Codegen/Measure
- [x] IMPL + Lex **758/758** CLOSED (Measure)
- [ ] Slice 2 → ADR-278 (pins ya GO-listo DOC; **GO IMPL** post-CLOSED 277)

## Cierre

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE08-FN-RESULT-20260926.md`](../GATE-CORE08-FN-RESULT-20260926.md)
- Lex measure **758/758** accepted → slice 1 **CLOSED** (evidence `DOC/reviews/MEASURE_ADR277_FN_RESULT_20260926.json`; exit 0)
- `measure_pass:false` pre-CLOSED era metadata «await Measure» (no fail material); flipped `true` on CLOSED
- Clippy workspace OK; Veyra companion **REJECTED** exit 1 (`rustfmt-diff` + VT*) **non-blocking** · `.veyra/evidence/20260926T132822Z/`
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair/I/O-new/String.set/threads/unwrap-surface/`?` **no** unpark
- Core **0.8** still open · Siguiente: **GO IMPL** ADR-278 QMARK only (279–280 HOLD)
