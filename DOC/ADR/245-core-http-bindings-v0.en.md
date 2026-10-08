Translation of `245-core-http-bindings-v0.md`; the original is normative. / Traducción de `245-core-http-bindings-v0.md`; el original es el normativo.

# ADR-245 — Core 0.2 slice 4: HTTP bindings surface

- **Estado:** **CLOSED** Lex **614/614** (2026-09-20) · siguiente ADR-246 SERVICE-POLICY
- **CUT-ID:** `CORE-0.2-HTTP-BINDINGS-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen (emit) · Ingeniero (gate)
- **Padre:** [ADR-233](233-core-0.2-pins.md) §0.1 slice 4
- **Prev:** ADR-244 TIMEOUT-CANCEL CLOSED
- **HOLD:** Mutex · sugar `[]` · insert panic · raw `std::net` · crates.io abierto · TLS full · WebSocket

## Decision (canonical name)

**Types + methods** — **not** free-fns `http_listen` / `http_serve` as primary surface.

| Surface | Form | Notes |
|---------|------|-------|
| `HttpServer` | opaque type | curated binding |
| `HttpRequest` | type | method/path/headers/body |
| `HttpResponse` | type | status/headers/body |
| `HttpServer.bind(addr: Text, port: Int) -> Result[HttpServer, IoError]` | associated/constructor | `addr` e.g. `"127.0.0.1"` |
| `server.serve(handler) -> Result[(), IoError]` | **async** method | handler below |
| `req.method() -> Text` | method | `"GET"` / `"POST"`… |
| `req.path() -> Text` | method | path only (query OUT 0.2) |
| `req.header(name: Text) -> Option[Text]` | method | case-insensitive impl OK |
| `req.body_text() -> Result[Text, IoError]` | method | respects body cap |
| `req.body_json() -> Result[Json, IoError]` | method | if Json IN; else defer body_text only + DOC |
| `HttpResponse.ok_text(body: Text) -> HttpResponse` | constructor | status 200 |
| `HttpResponse.ok_json(v: Json) -> HttpResponse` | constructor | if Json IN |
| `HttpResponse.status(code: Int, body: Text) -> HttpResponse` | constructor | 4xx/5xx |
| `resp.set_header(name: Text, value: Text) -> HttpResponse` | builder or bounded mut | one of the two; pin emit |

### Handler (v0 pin)

```text
async fn handle(req: HttpRequest) -> Result[HttpResponse, IoError]
```

`serve` registers **one** catch-all handler in v0 (path routing inside the handler with `match`). Multi-route registry = OUT 0.2 / Core 0.3.

### Caps (`service` profile, defaults)

- body max **1 MiB** → typed `Err` (no panic)
- headers max **64 KiB**
- idle timeout inherits slice 3 / policy (ADR-244 + slice 5)

## OUT slice 4

- Free `http_listen` / `http_serve` (if they appear in drafts → DOC-only alias or E0206)
- Sync blocking `HttpServer` serve as happy path
- TLS / HTTP2 / WS / multipart
- Typed query string / path params
- HTTP client (Core 0.3 ROADMAP draft)

## Emit notes (Codegen)

- Single curated adapter (runtime pinned ADR-233); **no** hyper/axum on surface
- `serve` → async Rust; user `forbid(unsafe)`
- Oversize body → `IoError` / stable code (DOC E0xxx if new; else generic IoError + measure)

## Oracles (Measure)

| Id | Expect |
|----|--------|
| `core02-http-bind-serve-smoke` | bind ephemeral localhost + serve + shutdown path |
| `core02-http-echo-json` or text | POST body roundtrip via handler |
| `neg-core02-http-listen-freefn` | free `http_listen` → reject E0206 (or parse) |
| `core02-http-body-cap` | oversize → typed Err |

## Slice 4 CLOSED criterion

Lex measure + green examples; mirror; ADR-233 tick slice 4; **no** global HOLD unpark.

## Checklist

- [x] Pin types vs free-fns
- [x] IMPL Lex surface (bind/port/shutdown/neg E0206; 2026-09-20)
- [x] Measure / Lex **614/614** CLOSED
- [x] Slice 5 SERVICE-POLICY GO → ADR-246

## Drift note (2026-09-20)

Provisional Codegen land used free-fns (reverted) (`http_listen` / `http_serve` / `http_route_*`) and `Result<_, String>`.

**Surface note:** canonical = types+methods; free-fns OUT (E0206). Engineer+Orchestrator **GO CLOSED** Lex **614/614** (HTTP trio); echo/body-cap → ADR-246.
