Translation of `246-core-service-policy-v0.md`; the original is normative. / Traducción de `246-core-service-policy-v0.md`; el original es el normativo.

# ADR-246 — Core 0.2 slice 5: SERVICE-POLICY

- **Estado:** **CLOSED** Lex **622/622** (2026-09-20) · siguiente ADR-247 SCENARIO-HTTP
- **CUT-ID:** `CORE-0.2-SERVICE-POLICY-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen (emit) · Measure (oracles) · Ingeniero (gate)
- **Padre:** [ADR-233](233-core-0.2-pins.md) §0.1 slice 5 · [ADR-245](245-core-http-bindings-v0.md) (slice 4 CLOSED Lex **614/614**)
- **Prev:** ADR-244 timeout/cancel · ADR-245 HTTP bindings
- **HOLD:** Mutex · sugar `[]` · insert panic · raw sockets · TLS PKI full · crates.io abierto

## Objective

Make `service` profile caps **enforceable** (not DOC-only): body / headers / idle / (optional) request timeout. Policy = typed data + fail oracles; no theater.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Surface** | `ServicePolicy` type + defaults + override on `HttpServer` |
| **Defaults** | body **1 MiB** · headers **64 KiB** · idle **30_000 ms** · header-count **100** |
| **Fail** | violation → `Err(IoError::…)` / stable code (**no** panic, no silent truncate) |
| **Manifiesto** | optional `arita.toml` / project policy table (names below) — if absent, defaults |
| **TLS** | `tls: Bool` field DOC/stub only — full IMPL OUT |
| **OUT** | raw socket opts · custom allocator · per-route policy matrix |


## 0.1 Deferred from slice 4 (CLOSED 614/614)

Orchestrator: echo E2E + body-cap measure **DEFER → this slice** (do not reopen 245).

| Oracle | Expect |
|--------|--------|
| `core02-http-echo-json` (or text) | POST echo roundtrip under default policy |
| `core02-policy-body-cap` | body > `max_body_bytes` → `IoError` (handler not 200) |
| `core02-http-health` | already slice4 smoke — re-run under policy defaults OK |

These oracles are **slice 5 CLOSED gate** together with §4.


## 0.2 Idle enforce — DEFER (Engineer 2026-09-20)

| Piece | Slice 5 | Later |
|-------|---------|-------|
| `idle_timeout_ms` field + `default()` + persist/`policy()` | **IN** | — |
| Runtime idle kill / connection drop on idle | **OUT / DEFER** | ADR-246a addendum **or** slice 6 SCENARIO-HTTP |

**CLOSED slice 5** = §4 oracles (defaults/set/E0323/body-cap/header-cap) + deferred echo/body-cap §0.1. **Does not** require idle-kill E2E. Do not reopen surface pins.

## 1. Canonical surface

```text
record ServicePolicy {
  max_body_bytes: Int,       // default 1_048_576
  max_header_bytes: Int,     // default 65_536
  max_header_count: Int,     // default 100
  idle_timeout_ms: Int,      // default 30_000; store IN; runtime kill DEFER §0.2
  // request_timeout_ms: Int // OPCIONAL v0.1 slice5 — si no entra, hereda ADR-244 timeout en handler
}

ServicePolicy.default() -> ServicePolicy

HttpServer.bind(addr, port) -> Result[HttpServer, IoError]
server.set_policy(p: ServicePolicy) -> Result[HttpServer, IoError]   // or builder; pin one emit form
server.policy() -> ServicePolicy                                      // read
```

**Rules:**
- ints **< 0** in policy → **compile/check E0323** `invalid policy value` (new; do not reuse E0321 HTTP)
- `max_body_bytes == 0` allowed only if DOC says “reject all bodies”; default pin = **reject** with E0323 or runtime Err — **pin: 0 illegal in v0** (E0323)
- Caps apply on the `serve` path (request parse) **before** the user handler

### Manifest (optional)

```toml
[profile.service]
max_body_bytes = 1048576
max_header_bytes = 65536
max_header_count = 100
idle_timeout_ms = 30000
```

If code has `set_policy` **and** TOML: **code wins** for that server (DOC). CI may pin TOML.

## 2. Codes / diags

| Code | When |
|------|------|
| **E0323** | invalid policy literal/field (neg, zero, ridiculous overflow) at check |
| **IoError** (runtime) | body/header oversize on request path — stable EN message; **idle exceed DEFER** §0.2 |

(Reuse E0321 only for HTTP surface misuse already pinned in 245; do not mix.)

## 3. Emit notes (Codegen)

- Policy → Rust struct in host `arita-host-http` / single adapter
- Defaults emitted if no `set_policy`
- Oversize: cut read + `Err`, **do not** hand partial body to handler
- Idle field stored; runtime kill DEFER §0.2 (handler timeout still ADR-244)

## 4. Oracles (Measure)

| Id | Expect |
|----|--------|
| `core02-policy-defaults` | server without set_policy → ADR defaults |
| `core02-policy-set-ok` | custom set_policy + health smoke |
| `core02-policy-body-cap` | body > max → typed Err (handler does not run / not 200) |
| `core02-policy-header-cap` | headers oversize/count → Err |
| `neg-core02-policy-neg-int` | `max_body_bytes: -1` → **E0323** |
| `neg-core02-policy-zero-body` | `0` → **E0323** |

## 5. CLOSED criterion

Lex measure green (table §4) + mirror; ADR-233 tick slice 5; **no** HOLD unpark. Then GO slice 6 SCENARIO-HTTP.

## Checklist

- [x] Surface pins + caps + E0323
- [x] idle enforce marked DEFER (§0.2)
- [x] IMPL + Lex **622/622**
- [x] Slice 6 GO → ADR-247
