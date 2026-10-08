# ADR-247 — Core 0.2 slice 6: SCENARIO-HTTP

- **Estado:** **CLOSED** Lex **627/627** (2026-09-20) · siguiente [ADR-248](248-core-ref-http-daemon-v0.md)
- **CUT-ID:** `CORE-0.2-SCENARIO-HTTP-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen (emit/runner) · Measure (oracles) · Ingeniero (gate)
- **Padre:** [ADR-233](233-core-0.2-pins.md) §0.1 slice 6
- **Prev:** ADR-246 SERVICE-POLICY **CLOSED** Lex **622/622**
- **HOLD:** Mutex · sugar `[]` · insert panic · raw sockets · crates.io abierto · idle-kill (sigue DEFER ADR-246 §0.2)

## Objetivo

Runner de **scenarios/acceptance HTTP** contra servidor en **puerto efímero** + **client HTTP curado** (binding allowlist, no crates.io). Extiende scenario language-level de Core 0.1 al vertical `service`.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Surface** | `scenario` / `acceptance` (reusar 0.1) + helpers HTTP client curados |
| **Server under test** | `HttpServer.bind("127.0.0.1", 0)` + `.serve(handler)` async (ADR-245) |
| **Client** | APIs tipadas abajo — **no** `reqwest`/`curl` en surface |
| **Puerto** | `0` → OS ephemeral; leer `server.port()` |
| **Lifecycle** | start → run scenarios → `shutdown()` (graceful) |
| **Policy** | defaults ADR-246; body-cap oracle IN gate |
| **OUT** | multi-host matrix · TLS real · browser · idle-kill enforce |

## 1. Surface client (curado)

```text
HttpClient.get(url: Text) -> Result[HttpResponse, IoError]
HttpClient.post_text(url: Text, body: Text) -> Result[HttpResponse, IoError]
HttpClient.post_json(url: Text, body: Json) -> Result[HttpResponse, IoError]  // si Json IN; else post_text

resp.status() -> Int
resp.body_text() -> Result[Text, IoError]
```

URL form v0: `"http://127.0.0.1:{port}/path"` (Text interpolado o helper `http_url(port, path)` **opcional** — si helper, DOC; no free `http_listen`).

### Scenario shape (v0)

```text
scenario echo_ok {
  // arrange: server already serving in test harness OR spawn in scenario prelude
  acceptance {
    // client call + asserts tipados (status/body)
  }
}
```

Pin harness Measure: puede levantar server en proceso Rust del oracle **o** programa `.arita` autocontenido; **skip ≠ PASS**.

## 2. Oracles (Measure) — gate CLOSED

| Id | Expect |
|----|--------|
| `core02-scen-health` | GET `/health` → 200 |
| `core02-scen-echo` | POST echo body roundtrip |
| `core02-scen-body-cap` | oversize → Err / non-200 (policy) |
| `core02-scen-shutdown` | serve + shutdown limpio (no hang) |
| `neg-core02-scen-reqwest` | surface `reqwest::` / crate libre → reject |

Idle-kill E2E: **OUT** este slice (ADR-246 §0.2).

## 3. Emit / runner notes

- Client → adapter único host (mismo stack que server o thin curated)
- Scenarios net en `arita measure` con timeout global (ADR-244)
- Evidence: opcional hash request/response en sidecar — full evidence → slice 7

## 4. Criterio CLOSED

Lex measure §2 verde + mirror; ADR-233 tick slice 6; **sin** unpark HOLD. Luego GO slice 7 REF-HTTP-DAEMON.

## Checklist

- [x] Pins client + oracles
- [x] IMPL + Lex **627/627**
- [x] Slice 7 GO → ADR-248
