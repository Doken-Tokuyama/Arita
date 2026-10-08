Translation of `253-core-ref-compose-v0.md`; the original is normative. / Traducción de `253-core-ref-compose-v0.md`; el original es el normativo.

# ADR-253 — Core 0.3 slice 5: REF-COMPOSE

- **Estado:** **CLOSED** Lex **658/658** (2026-09-20) · Core **0.3 CLOSED** · siguiente [ADR-254](254-core-0.4-pins.md)
- **CUT-ID:** `CORE-0.3-REF-COMPOSE-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-249](249-core-0.3-pins.md) §0.1 slice 5 · §2 programa ref
- **Prev:** slice 4 SCENARIO-COMPOSE **CLOSED** Lex **651/651** (ADR-252)
- **Cierra:** vertical Core **0.3** (composición)
- **HOLD:** Mutex · `[]` · insert panic · idle-kill · TLS/WS · crates.io · repair · fan-out paralelo

## Objective

Non-trivial ref `arita-ref-http-compose` + evidence + Lex bar that **closes Core 0.3**: BFF gateway → seq upstream(s) → JSON out.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Nombre** | `arita-ref-http-compose` (`ejemplos/core03/ref-http-compose/` or Lex equiv.) |
| **Topology** | gateway `bind(127.0.0.1,0)` + upstream mock/bind port 0 |
| **Rutas gateway** | `GET /health` · `POST /pipeline` (A→B seq, ADR-251) |
| **Upstream** | `/step-a` · `/step-b` (or one echo + local transform = 2nd stage) |
| **Errores** | ADR-250 map |
| **Scenarios** | ≥ happy + fail-A + fail-B (252) |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`) |
| **OUT** | auth · WS · Mutex · >2 hops · circuit breaker |

## 1. Surface

Only already-IN APIs (245–252). No new std.

## 2. Oracles — Core 0.3 CLOSED gate

| Id | Expect |
|----|--------|
| `core03-ref-health` | GET health 200 |
| `core03-ref-pipeline-happy` | POST /pipeline 200 |
| `core03-ref-fail-a` | 502 / upstream_unavailable |
| `core03-ref-fail-b` | 502 (B down) |
| `core03-ref-evidence` | evidence JSON + stable hash |
| `core03-ref-shutdown` | no hang |

skip ≠ PASS. Engineer fixes Lex N/N.

## 3. CLOSED criterion (slice 5 = Core 0.3 CLOSED)

Lex §2 green; ROADMAP/ADR-249 Core 0.3 **CLOSED**; HOLDs not unparked.

## 4. Post-0.3 (not this CUT)

0.4+ per ROADMAP; idle-kill 246a; Mutex/`[]`/insert stay R5/R7.

## Checklist

- [x] Ref pins + oracles + evidence
- [x] IMPL + Lex Core 0.3 CLOSED **658/658**
