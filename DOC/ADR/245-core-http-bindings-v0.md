# ADR-245 — Core 0.2 slice 4: HTTP bindings surface

- **Estado:** **CLOSED** Lex **614/614** (2026-09-20) · siguiente ADR-246 SERVICE-POLICY
- **CUT-ID:** `CORE-0.2-HTTP-BINDINGS-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen (emit) · Ingeniero (gate)
- **Padre:** [ADR-233](233-core-0.2-pins.md) §0.1 slice 4
- **Prev:** ADR-244 TIMEOUT-CANCEL CLOSED
- **HOLD:** Mutex · sugar `[]` · insert panic · raw `std::net` · crates.io abierto · TLS full · WebSocket

## Decisión (nombre canónico)

**Tipos + métodos** — **no** free-fns `http_listen` / `http_serve` como surface primaria.

| Surface | Forma | Notas |
|---------|-------|-------|
| `HttpServer` | tipo opaco | binding curado |
| `HttpRequest` | tipo | method/path/headers/body |
| `HttpResponse` | tipo | status/headers/body |
| `HttpServer.bind(addr: Text, port: Int) -> Result[HttpServer, IoError]` | associated/constructor | `addr` p.ej. `"127.0.0.1"` |
| `server.serve(handler) -> Result[(), IoError]` | método **async** | handler abajo |
| `req.method() -> Text` | método | `"GET"` / `"POST"`… |
| `req.path() -> Text` | método | path only (query OUT 0.2) |
| `req.header(name: Text) -> Option[Text]` | método | case-insensitive impl OK |
| `req.body_text() -> Result[Text, IoError]` | método | respeta cap body |
| `req.body_json() -> Result[Json, IoError]` | método | si Json IN; else defer body_text only + DOC |
| `HttpResponse.ok_text(body: Text) -> HttpResponse` | constructor | status 200 |
| `HttpResponse.ok_json(v: Json) -> HttpResponse` | constructor | si Json IN |
| `HttpResponse.status(code: Int, body: Text) -> HttpResponse` | constructor | 4xx/5xx |
| `resp.set_header(name: Text, value: Text) -> HttpResponse` | builder o mut acotado | una de las dos; pin emit |

### Handler (pin v0)

```text
async fn handle(req: HttpRequest) -> Result[HttpResponse, IoError]
```

`serve` registra **un** handler catch-all en v0 (routing por path dentro del handler con `match`). Multi-route registry = OUT 0.2 / Core 0.3.

### Caps (perfil `service`, defaults)

- body max **1 MiB** → `Err` tipado (no panic)
- headers max **64 KiB**
- idle timeout hereda slice 3 / policy (ADR-244 + slice 5)

## OUT slice 4

- Free `http_listen` / `http_serve` (si aparecen en borradores → alias DOC-only o E0206)
- `HttpServer` sync blocking serve como path feliz
- TLS / HTTP2 / WS / multipart
- Query string / path params tipados
- Client HTTP (Core 0.3 borrador ROADMAP)

## Emit notes (Codegen)

- Adapter único curado (runtime pinado ADR-233); **no** hyper/axum en surface
- `serve` → async Rust; usuario `forbid(unsafe)`
- Oversize body → `IoError` / código estable (DOC E0xxx si nuevo; si no, IoError genérico + measure)

## Oracles (Measure)

| Id | Expect |
|----|--------|
| `core02-http-bind-serve-smoke` | bind localhost efímero + serve + shutdown path |
| `core02-http-echo-json` o text | POST body roundtrip via handler |
| `neg-core02-http-listen-freefn` | free `http_listen` → reject E0206 (o parse) |
| `core02-http-body-cap` | oversize → Err tipado |

## Criterio CLOSED slice 4

Lex measure + examples verdes; mirror; ADR-233 tick slice 4; **sin** unpark HOLD globales.

## Checklist

- [x] Pin tipos vs free-fns
- [x] IMPL surface Lex (bind/port/shutdown/neg E0206; 2026-09-20)
- [x] Measure / Lex **614/614** CLOSED
- [x] Slice 5 SERVICE-POLICY GO → ADR-246

## Drift note (2026-09-20)

Codegen land provisional usó free-fns (revertido) (`http_listen` / `http_serve` / `http_route_*`) y `Result<_, String>`.

**Nota surface:** canónica = tipos+métodos; free-fns OUT (E0206). Ingeniero+Orquestador **GO CLOSED** Lex **614/614** (trio HTTP); echo/body-cap → ADR-246.

