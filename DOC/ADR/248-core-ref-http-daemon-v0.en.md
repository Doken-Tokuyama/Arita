Translation of `248-core-ref-http-daemon-v0.md`; the original is normative. / Traducción de `248-core-ref-http-daemon-v0.md`; el original es el normativo.

# ADR-248 — Core 0.2 slice 7: REF-HTTP-DAEMON

- **Estado:** **CLOSED** Lex **632/632** (2026-09-20) · Core **0.2 vertical CLOSED** · siguiente Core 0.3 = ADR-249
- **CUT-ID:** `CORE-0.2-REF-HTTP-DAEMON-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen (emit) · Measure (barra E2E) · Ingeniero (gate)
- **Padre:** [ADR-233](233-core-0.2-pins.md) §0.1 slice 7 · §3 programa ref
- **Prev:** ADR-247 SCENARIO-HTTP **CLOSED** Lex **627/627**
- **Cierra:** vertical Core **0.2** (perfil `service`)
- **HOLD:** Mutex · sugar `[]` · insert panic · raw sockets · crates.io abierto · idle-kill runtime (DEFER 246 §0.2) · TLS PKI full · `arita repair` IMPL

## Goal

**Non-trivial** reference program + evidence + Lex bar that **closes Core 0.2**: ARITA HTTP daemon → real Rust → net scenarios → evidence hash.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Nombre ref** | `arita-ref-http-json` (path `ejemplos/core02/ref-http-json/` o equivalente Lex) |
| **Bind** | `HttpServer.bind("127.0.0.1", 0)` + `.port()` |
| **Min. routes** | `GET /health` → 200 simple text/json · `POST /echo` → body roundtrip |
| **Policy** | `ServicePolicy.default()` (ADR-246); body-cap enforceable |
| **Lifecycle** | serve async + graceful `.shutdown()` |
| **Scenarios** | ≥2 acceptance via ADR-247 client (health + echo); body-cap re-run OK |
| **Evidence** | JSON sidecar hash source↔emit↔scenario results (`tested` level) |
| **Profile** | compile/measure under `service` profile |
| **OUT** | auth · multi-route registry fancy · TLS · idle-kill · Mutex · client externo |

## 1. Surface (only APIs already IN)

Only ADR-245 HTTP + ADR-246 policy + ADR-247 client/scenarios + already-CLOSED async/timeout. **No** new surface except local example helpers.

## 2. Oracles / Lex bar CLOSED Core 0.2

| Id | Expect |
|----|--------|
| `core02-ref-health` | GET health 200 |
| `core02-ref-echo` | POST echo roundtrip |
| `core02-ref-body-cap` | oversize → Err / non-200 |
| `core02-ref-shutdown` | no hang post-shutdown |
| `core02-ref-evidence` | evidence JSON present + stable hash (re-run) |

skip ≠ PASS. Aggregated Lex suite signs the agreed bar (Engineer sets N/N).

## 3. Emit / repo layout

- `.arita` example + measure harness (if needed)
- User emit `#![forbid(unsafe_code)]`
- Curated path-dep host crates (ADR-035/245)

## 4. CLOSED criterion (slice 7 = Core 0.2 CLOSED)

- Lex measure green bar (§2 table)
- Evidence artifact in tree/local CI
- ADR-233 / ROADMAP: Core 0.2 **CLOSED**
- **Sin** unpark global HOLDs

## 5. Post-0.2 (not this CUT)

Core 0.3 ROADMAP draft (HTTP client composition, etc.). Idle-kill → ADR-246a. Mutex/`[]`/insert stay on R5/R7 evidence.

## Checklist

- [x] Ref pins + oracles + evidence
- [x] IMPL + Lex Core 0.2 CLOSED **632/632**
