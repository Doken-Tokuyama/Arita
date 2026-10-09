Translation of `252-core-scenario-compose-v0.md`; the original is normative. / Traducción de `252-core-scenario-compose-v0.md`; el original es el normativo.

# ADR-252 — Core 0.3 slice 4: SCENARIO-COMPOSE

- **Estado:** **CLOSED** Lex **651/651** (2026-09-20) · siguiente [ADR-253](253-core-ref-compose-v0.md)
- **CUT-ID:** `CORE-0.3-SCENARIO-COMPOSE-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-249](249-core-0.3-pins.md) §0.1 slice 4
- **Prev:** slice 3 SEQ-PIPELINE **CLOSED** Lex **646/646** (ADR-251)
- **HOLD:** Mutex · `[]` · insert panic · idle-kill · TLS/WS · crates.io · repair · parallel fan-out

## Objective

Stable **multi-hop** scenarios/acceptance: gateway + upstream(s) on ephemeral port(s), curated client (ADR-247), errors (250), seq pipeline (251). No new std surface.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Harness** | 2 binds (`gateway` port 0 + `upstream` port 0) **or** 1 dual-listen process — Measure chooses; DOC in oracle |
| **Client** | only `HttpClient` 247 (get/post_text) |
| **Minimum coverage** | happy compose · fail-A · fail-B · (optional) 4xx forward |
| **Lifecycle** | start both → scenarios → shutdown both (no hang) |
| **OUT** | browser · TLS · chaos mesh · >2 hops |

## 1. Surface

Reuses `scenario`/`acceptance` (0.1/0.2) + already-IN HTTP server/client. Pattern: scenario arranges ports → POST gateway `/pipeline` or `/proxy` → asserts status/body.

## 2. Oracles (Measure) — CLOSED gate

| Id | Expect |
|----|--------|
| `core03-scen-compose-happy` | 2 hops OK → 200 + expected body |
| `core03-scen-compose-fail-a` | upstream A down → 502/`upstream_unavailable`; B not invoked if observable |
| `core03-scen-compose-fail-b` | A OK, B down → 502 (map 250) |
| `core03-scen-compose-shutdown` | clean dual shutdown |
| `neg-core03-scen-reqwest` | free crate/client → reject (if applicable) |

## 3. CLOSED criterion

Lex §2 green; tick ADR-249 slice 4; HOLDs intact. Next: slice 5 REF-COMPOSE (closes Core 0.3).

## Checklist

- [x] Harness pins + oracles
- [x] IMPL + Lex **651/651**
- [x] Slice 5 GO → ADR-253
