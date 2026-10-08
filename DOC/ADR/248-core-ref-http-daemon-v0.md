# ADR-248 — Core 0.2 slice 7: REF-HTTP-DAEMON

- **Estado:** **CLOSED** Lex **632/632** (2026-09-20) · Core **0.2 vertical CLOSED** · siguiente Core 0.3 = ADR-249
- **CUT-ID:** `CORE-0.2-REF-HTTP-DAEMON-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen (emit) · Measure (barra E2E) · Ingeniero (gate)
- **Padre:** [ADR-233](233-core-0.2-pins.md) §0.1 slice 7 · §3 programa ref
- **Prev:** ADR-247 SCENARIO-HTTP **CLOSED** Lex **627/627**
- **Cierra:** vertical Core **0.2** (perfil `service`)
- **HOLD:** Mutex · sugar `[]` · insert panic · raw sockets · crates.io abierto · idle-kill runtime (DEFER 246 §0.2) · TLS PKI full · `arita repair` IMPL

## Objetivo

Programa de referencia **no trivial** + evidence + Lex barra que **cierra Core 0.2**: daemon HTTP ARITA → Rust real → scenarios net → evidence hash.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Nombre ref** | `arita-ref-http-json` (path `ejemplos/core02/ref-http-json/` o equivalente Lex) |
| **Bind** | `HttpServer.bind("127.0.0.1", 0)` + `.port()` |
| **Rutas mín.** | `GET /health` → 200 texto/json simple · `POST /echo` → body roundtrip |
| **Policy** | `ServicePolicy.default()` (ADR-246); body-cap enforceable |
| **Lifecycle** | serve async + graceful `.shutdown()` |
| **Scenarios** | ≥2 acceptance vía ADR-247 client (health + echo); body-cap re-run OK |
| **Evidence** | JSON sidecar hash source↔emit↔scenario results (nivel `tested`) |
| **Perfil** | compile/measure bajo perfil `service` |
| **OUT** | auth · multi-route registry fancy · TLS · idle-kill · Mutex · client externo |

## 1. Surface (solo APIs ya IN)

Solo ADR-245 HTTP + ADR-246 policy + ADR-247 client/scenarios + async/timeout ya CLOSED. **Sin** surface nueva salvo helpers locales del ejemplo.

## 2. Oracles / barra Lex CLOSED Core 0.2

| Id | Expect |
|----|--------|
| `core02-ref-health` | GET health 200 |
| `core02-ref-echo` | POST echo roundtrip |
| `core02-ref-body-cap` | oversize → Err / non-200 |
| `core02-ref-shutdown` | no hang post-shutdown |
| `core02-ref-evidence` | evidence JSON presente + hash estable (re-run) |

skip ≠ PASS. Suite agregada Lex firma barra acordada (Ingeniero fija N/N).

## 3. Emit / repo layout

- Ejemplo `.arita` + (si hace falta) harness measure
- Emit `#![forbid(unsafe_code)]` usuario
- Host crates path-dep curados (ADR-035/245)

## 4. Criterio CLOSED (slice 7 = Core 0.2 CLOSED)

- Lex measure barra verde (tabla §2)
- Evidence artifact en tree/CI local
- ADR-233 / ROADMAP: Core 0.2 **CLOSED**
- **Sin** unpark HOLD globales

## 5. Post-0.2 (no este CUT)

Core 0.3 borrador ROADMAP (HTTP client composición, etc.). Idle-kill → ADR-246a. Mutex/`[]`/insert siguen evidencia R5/R7.

## Checklist

- [x] Pins ref + oracles + evidence
- [x] IMPL + Lex Core 0.2 CLOSED **632/632**
