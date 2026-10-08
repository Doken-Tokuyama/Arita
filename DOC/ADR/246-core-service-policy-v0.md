# ADR-246 — Core 0.2 slice 5: SERVICE-POLICY

- **Estado:** **CLOSED** Lex **622/622** (2026-09-20) · siguiente ADR-247 SCENARIO-HTTP
- **CUT-ID:** `CORE-0.2-SERVICE-POLICY-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen (emit) · Measure (oracles) · Ingeniero (gate)
- **Padre:** [ADR-233](233-core-0.2-pins.md) §0.1 slice 5 · [ADR-245](245-core-http-bindings-v0.md) (slice 4 CLOSED Lex **614/614**)
- **Prev:** ADR-244 timeout/cancel · ADR-245 HTTP bindings
- **HOLD:** Mutex · sugar `[]` · insert panic · raw sockets · TLS PKI full · crates.io abierto

## Objetivo

Hacer **enforzables** (no solo DOC) los caps del perfil `service`: body / headers / idle / (opcional) request timeout. Policy = datos tipados + fail oracles; no theater.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Surface** | tipo `ServicePolicy` + defaults + override en `HttpServer` |
| **Defaults** | body **1 MiB** · headers **64 KiB** · idle **30_000 ms** · header-count **100** |
| **Fail** | violación → `Err(IoError::…)` / código estable (**no** panic, no truncate silent) |
| **Manifiesto** | opcional `arita.toml` / project policy table (nombres abajo) — si ausente, defaults |
| **TLS** | campo `tls: Bool` DOC/stub only — IMPL full OUT |
| **OUT** | raw socket opts · custom allocator · per-route policy matrix |


## 0.1 Diferidos desde slice 4 (CLOSED 614/614)

Orquestador: echo E2E + body-cap measure **DEFER → este slice** (no reabrir 245).

| Oracle | Expect |
|--------|--------|
| `core02-http-echo-json` (o text) | POST echo roundtrip bajo policy default |
| `core02-policy-body-cap` | body > `max_body_bytes` → `IoError` (handler no 200) |
| `core02-http-health` | ya smoke slice4 — re-run bajo policy defaults OK |

Estos oracles son **gate CLOSED slice 5** junto a §4.


## 0.2 Idle enforce — DEFER (Ingeniero 2026-09-20)

| Pieza | Slice 5 | Más tarde |
|-------|---------|-----------|
| `idle_timeout_ms` field + `default()` + persist/`policy()` | **IN** | — |
| Runtime idle kill / connection drop por idle | **OUT / DEFER** | addenda ADR-246a **o** slice 6 SCENARIO-HTTP |

**CLOSED slice 5** = oracles §4 (defaults/set/E0323/body-cap/header-cap) + echo/body-cap diferidos §0.1. **No** exige idle-kill E2E. No reabrir surface pins.

## 1. Surface canónica

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
server.set_policy(p: ServicePolicy) -> Result[HttpServer, IoError]   // o builder; pin emit una forma
server.policy() -> ServicePolicy                                      // lectura
```

**Reglas:**
- ints **< 0** en policy → **compile/check E0323** `invalid policy value` (nuevo; no reusar E0321 HTTP)
- `max_body_bytes == 0` permitido solo si DOC dice “reject all bodies”; default pin = **reject** con E0323 o runtime Err — **pin: 0 ilegal en v0** (E0323)
- Caps aplican en `serve` path (request parse) **antes** del handler usuario

### Manifiesto (opcional)

```toml
[profile.service]
max_body_bytes = 1048576
max_header_bytes = 65536
max_header_count = 100
idle_timeout_ms = 30000
```

Si hay `set_policy` en código **y** TOML: **código gana** para ese server (DOC). CI puede fijar TOML.

## 2. Códigos / diags

| Código | Cuándo |
|--------|--------|
| **E0323** | policy literal/campo inválido (neg, zero, overflow ridículo) en check |
| **IoError** (runtime) | body/header oversize en request path — message estable EN; **idle exceed DEFER** §0.2 |

(Reuse E0321 solo para HTTP surface misuse ya pinado en 245; no mezclar.)

## 3. Emit notes (Codegen)

- Policy → struct Rust en host `arita-host-http` / adapter único
- Defaults emitidos si no hay `set_policy`
- Oversize: cortar read + `Err`, **no** entregar body parcial al handler
- Idle field stored; runtime kill DEFER §0.2 (handler timeout sigue ADR-244)

## 4. Oracles (Measure)

| Id | Expect |
|----|--------|
| `core02-policy-defaults` | server sin set_policy → defaults ADR |
| `core02-policy-set-ok` | set_policy custom + smoke health |
| `core02-policy-body-cap` | body > max → Err tipado (handler no corre / no 200) |
| `core02-policy-header-cap` | headers oversize/count → Err |
| `neg-core02-policy-neg-int` | `max_body_bytes: -1` → **E0323** |
| `neg-core02-policy-zero-body` | `0` → **E0323** |

## 5. Criterio CLOSED

Lex measure verdes (tabla §4) + mirror; ADR-233 tick slice 5; **sin** unpark HOLD. Luego GO slice 6 SCENARIO-HTTP.

## Checklist

- [x] Pins surface + caps + E0323
- [x] idle enforce marcado DEFER (§0.2)
- [x] IMPL + Lex **622/622**
- [x] Slice 6 GO → ADR-247
