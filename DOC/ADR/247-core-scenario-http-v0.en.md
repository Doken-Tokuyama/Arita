Translation of `247-core-scenario-http-v0.md`; the original is normative. / Traducción de `247-core-scenario-http-v0.md`; el original es el normativo.

# ADR-247 — Core 0.2 slice 6: SCENARIO-HTTP

- **Estado:** **CLOSED** Lex **627/627** (2026-09-20) · siguiente [ADR-248](248-core-ref-http-daemon-v0.md)
- **CUT-ID:** `CORE-0.2-SCENARIO-HTTP-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen (emit/runner) · Measure (oracles) · Ingeniero (gate)
- **Padre:** [ADR-233](233-core-0.2-pins.md) §0.1 slice 6
- **Prev:** ADR-246 SERVICE-POLICY **CLOSED** Lex **622/622**
- **HOLD:** Mutex · sugar `[]` · insert panic · raw sockets · crates.io abierto · idle-kill (sigue DEFER ADR-246 §0.2)

## Objective

**HTTP scenario/acceptance** runner against a server on an **ephemeral port** + **curated HTTP client** (binding allowlist, no crates.io). Extends Core 0.1 language-level scenario to the `service` vertical.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Surface** | `scenario` / `acceptance` (reuse 0.1) + curated HTTP client helpers |
| **Server under test** | `HttpServer.bind("127.0.0.1", 0)` + async `.serve(handler)` (ADR-245) |
| **Client** | typed APIs below — **no** `reqwest`/`curl` on surface |
| **Puerto** | `0` → OS ephemeral; read `server.port()` |
| **Lifecycle** | start → run scenarios → `shutdown()` (graceful) |
| **Policy** | ADR-246 defaults; body-cap oracle IN gate |
| **OUT** | multi-host matrix · real TLS · browser · idle-kill enforce |

## 1. Client surface (curated)

```text
HttpClient.get(url: Text) -> Result[HttpResponse, IoError]
HttpClient.post_text(url: Text, body: Text) -> Result[HttpResponse, IoError]
HttpClient.post_json(url: Text, body: Json) -> Result[HttpResponse, IoError]  // si Json IN; else post_text

resp.status() -> Int
resp.body_text() -> Result[Text, IoError]
```

v0 URL form: `"http://127.0.0.1:{port}/path"` (interpolated Text or optional `http_url(port, path)` helper — if helper, DOC; no free `http_listen`).

### Scenario shape (v0)

```text
scenario echo_ok {
  // arrange: server already serving in test harness OR spawn in scenario prelude
  acceptance {
    // client call + asserts tipados (status/body)
  }
}
```

Measure harness pin: may lift the server in the oracle’s Rust process **or** a self-contained `.arita` program; **skip ≠ PASS**.

## 2. Oracles (Measure) — CLOSED gate

| Id | Expect |
|----|--------|
| `core02-scen-health` | GET `/health` → 200 |
| `core02-scen-echo` | POST echo body roundtrip |
| `core02-scen-body-cap` | oversize → Err / non-200 (policy) |
| `core02-scen-shutdown` | clean serve + shutdown (no hang) |
| `neg-core02-scen-reqwest` | surface `reqwest::` / free crate → reject |

Idle-kill E2E: **OUT** this slice (ADR-246 §0.2).

## 3. Emit / runner notes

- Client → single host adapter (same stack as server or thin curated)
- Net scenarios in `arita measure` with global timeout (ADR-244)
- Evidence: optional request/response hash in sidecar — full evidence → slice 7

## 4. CLOSED criterion

Lex measure §2 green + mirror; ADR-233 tick slice 6; **no** HOLD unpark. Then GO slice 7 REF-HTTP-DAEMON.

## Checklist

- [x] Client pins + oracles
- [x] IMPL + Lex **627/627**
- [x] Slice 7 GO → ADR-248
