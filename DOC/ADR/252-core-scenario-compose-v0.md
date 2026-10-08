# ADR-252 — Core 0.3 slice 4: SCENARIO-COMPOSE

- **Estado:** **CLOSED** Lex **651/651** (2026-09-20) · siguiente [ADR-253](253-core-ref-compose-v0.md)
- **CUT-ID:** `CORE-0.3-SCENARIO-COMPOSE-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-249](249-core-0.3-pins.md) §0.1 slice 4
- **Prev:** slice 3 SEQ-PIPELINE **CLOSED** Lex **646/646** (ADR-251)
- **HOLD:** Mutex · `[]` · insert panic · idle-kill · TLS/WS · crates.io · repair · parallel fan-out

## Objetivo

Scenarios/acceptance **multi-hop** estables: gateway + upstream(s) en puerto(s) efímero(s), client curado (ADR-247), errores (250), pipeline seq (251). Sin surface std nueva.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Harness** | 2 binds (`gateway` port 0 + `upstream` port 0) **o** 1 proceso dual-listen — Measure elige; DOC en oracle |
| **Client** | solo `HttpClient` 247 (get/post_text) |
| **Coverage mínima** | happy compose · fail-A · fail-B · (opcional) 4xx forward |
| **Lifecycle** | start both → scenarios → shutdown both (no hang) |
| **OUT** | browser · TLS · chaos mesh · >2 hops |

## 1. Surface

Reusa `scenario`/`acceptance` (0.1/0.2) + HTTP server/client ya IN. Pattern: scenario arrange ports → POST gateway `/pipeline` o `/proxy` → asserts status/body.

## 2. Oracles (Measure) — gate CLOSED

| Id | Expect |
|----|--------|
| `core03-scen-compose-happy` | 2 hops OK → 200 + body esperado |
| `core03-scen-compose-fail-a` | upstream A down → 502/`upstream_unavailable`; B no invocado si observable |
| `core03-scen-compose-fail-b` | A OK, B down → 502 (mapa 250) |
| `core03-scen-compose-shutdown` | dual shutdown limpio |
| `neg-core03-scen-reqwest` | crate/client libre → reject (si aplica) |

## 3. Criterio CLOSED

Lex §2 verde; tick ADR-249 slice 4; HOLDs intactos. Siguiente: slice 5 REF-COMPOSE (cierra Core 0.3).

## Checklist

- [x] Pins harness + oracles
- [x] IMPL + Lex **651/651**
- [x] Slice 5 GO → ADR-253
