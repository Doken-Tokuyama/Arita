# ADR-253 — Core 0.3 slice 5: REF-COMPOSE

- **Estado:** **CLOSED** Lex **658/658** (2026-09-20) · Core **0.3 CLOSED** · siguiente [ADR-254](254-core-0.4-pins.md)
- **CUT-ID:** `CORE-0.3-REF-COMPOSE-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-249](249-core-0.3-pins.md) §0.1 slice 5 · §2 programa ref
- **Prev:** slice 4 SCENARIO-COMPOSE **CLOSED** Lex **651/651** (ADR-252)
- **Cierra:** vertical Core **0.3** (composición)
- **HOLD:** Mutex · `[]` · insert panic · idle-kill · TLS/WS · crates.io · repair · fan-out paralelo

## Objetivo

Ref **no trivial** `arita-ref-http-compose` + evidence + Lex barra que **cierra Core 0.3**: gateway BFF → upstream(s) seq → JSON out.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Nombre** | `arita-ref-http-compose` (`ejemplos/core03/ref-http-compose/` o Lex equiv.) |
| **Topology** | gateway `bind(127.0.0.1,0)` + upstream mock/bind port 0 |
| **Rutas gateway** | `GET /health` · `POST /pipeline` (A→B seq, ADR-251) |
| **Upstream** | `/step-a` · `/step-b` (o un echo + transform local = 2ª etapa) |
| **Errores** | mapa ADR-250 |
| **Scenarios** | ≥ happy + fail-A + fail-B (252) |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`) |
| **OUT** | auth · WS · Mutex · >2 hops · circuit breaker |

## 1. Surface

Solo APIs ya IN (245–252). Sin std nueva.

## 2. Oracles — gate CLOSED Core 0.3

| Id | Expect |
|----|--------|
| `core03-ref-health` | GET health 200 |
| `core03-ref-pipeline-happy` | POST /pipeline 200 |
| `core03-ref-fail-a` | 502 / upstream_unavailable |
| `core03-ref-fail-b` | 502 (B down) |
| `core03-ref-evidence` | evidence JSON + hash estable |
| `core03-ref-shutdown` | no hang |

skip ≠ PASS. Ingeniero fija N/N Lex.

## 3. Criterio CLOSED (slice 5 = Core 0.3 CLOSED)

Lex §2 verde; ROADMAP/ADR-249 Core 0.3 **CLOSED**; HOLDs no unpark.

## 4. Post-0.3 (no este CUT)

0.4+ según ROADMAP; idle-kill 246a; Mutex/`[]`/insert siguen R5/R7.

## Checklist

- [x] Pins ref + oracles + evidence
- [x] IMPL + Lex Core 0.3 CLOSED **658/658**
