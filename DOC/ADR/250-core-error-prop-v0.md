# ADR-250 — Core 0.3 slice 2: ERROR-PROP

- **Estado:** **CLOSED** Lex **643/643** (2026-09-20) · siguiente [ADR-251](251-core-seq-pipeline-v0.md)
- **CUT-ID:** `CORE-0.3-ERROR-PROP-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-249](249-core-0.3-pins.md) §0.1 slice 2
- **Prev:** slice 1 CLIENT-COMPOSE **CLOSED** Lex **638/638**
- **HOLD:** Mutex · `[]` · insert panic · idle-kill · TLS/WS · crates.io · repair IMPL · panic/`unwrap` surface

## Objetivo

Mapear fallos **upstream HTTP** / `IoError` a **responses tipadas** en el gateway — sin panic, sin tragar `Err`.

## 0. Corte decisivo (tabla de mapa)

| Condición | Respuesta gateway |
|-----------|-------------------|
| `HttpClient.*` → `Err` (connect / IO; sin response) | **502** body `"upstream_unavailable"` |
| `Err` distinguible como timeout (ADR-244) | **504** body `"upstream_timeout"`; si aún no distinguible → 502 OK v0 + DOC |
| Upstream status **4xx/5xx** | **forward** `status` + `body_text` (cap policy 246) |
| Upstream **2xx** | forward body (echo/proxy) |
| Body-cap local | sigue ADR-246 pre-handler (no mezclar) |

**Sin** API std nueva obligatoria — `match` sobre `Result` + `resp.status()` es el pin. Helpers sugar = OUT v0.

## 1. Pattern canónico (ejemplo)

```text
match client.post_text(url, body) {
  Ok(up) => HttpResponse.status(up.status(), /* body forward */)
  Err(_) => HttpResponse.status(502, "upstream_unavailable")
}
```

(Timeout → 504 cuando `IoError` lo permita.)

## 2. Oracles (Measure)

| Id | Expect |
|----|--------|
| `core03-err-upstream-down` | sin upstream → **502** + `upstream_unavailable` |
| `core03-err-upstream-4xx` | upstream 404 → gateway **404** |
| `core03-err-upstream-5xx` | upstream 503 → gateway **503** |
| `core03-err-timeout` | timeout → **504** o **502** si no hay distinción (DOC) |
| `neg-core03-err-unwrap` | path compose sin unwrap/expect/panic en emit |

## 3. Criterio CLOSED

Lex §2 verde; tick ADR-249 slice 2; HOLDs intactos. Siguiente: slice 3 SEQ-PIPELINE.

## Checklist

- [x] Pins mapa + oracles
- [x] IMPL + Lex **643/643**
- [x] Slice 3 GO → ADR-251
