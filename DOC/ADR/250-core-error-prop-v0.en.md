Translation of `250-core-error-prop-v0.md`; the original is normative. / Traducción de `250-core-error-prop-v0.md`; el original es el normativo.

# ADR-250 — Core 0.3 slice 2: ERROR-PROP

- **Estado:** **CLOSED** Lex **643/643** (2026-09-20) · siguiente [ADR-251](251-core-seq-pipeline-v0.md)
- **CUT-ID:** `CORE-0.3-ERROR-PROP-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-249](249-core-0.3-pins.md) §0.1 slice 2
- **Prev:** slice 1 CLIENT-COMPOSE **CLOSED** Lex **638/638**
- **HOLD:** Mutex · `[]` · insert panic · idle-kill · TLS/WS · crates.io · repair IMPL · panic/`unwrap` surface

## Objective

Map **upstream HTTP** / `IoError` failures to typed **gateway responses** — no panic, no swallowing `Err`.

## 0. Decisive cut (map table)

| Condition | Gateway response |
|-----------|------------------|
| `HttpClient.*` → `Err` (connect / IO; no response) | **502** body `"upstream_unavailable"` |
| `Err` distinguishable as timeout (ADR-244) | **504** body `"upstream_timeout"`; if not yet distinguishable → 502 OK v0 + DOC |
| Upstream status **4xx/5xx** | **forward** `status` + `body_text` (policy cap 246) |
| Upstream **2xx** | forward body (echo/proxy) |
| Local body-cap | follows ADR-246 pre-handler (do not mix) |

**No** mandatory new std API — `match` on `Result` + `resp.status()` is the pin. Sugar helpers = OUT v0.

## 1. Canonical pattern (example)

```text
match client.post_text(url, body) {
  Ok(up) => HttpResponse.status(up.status(), /* body forward */)
  Err(_) => HttpResponse.status(502, "upstream_unavailable")
}
```

(Timeout → 504 when `IoError` allows.)

## 2. Oracles (Measure)

| Id | Expect |
|----|--------|
| `core03-err-upstream-down` | no upstream → **502** + `upstream_unavailable` |
| `core03-err-upstream-4xx` | upstream 404 → gateway **404** |
| `core03-err-upstream-5xx` | upstream 503 → gateway **503** |
| `core03-err-timeout` | timeout → **504** or **502** if no distinction (DOC) |
| `neg-core03-err-unwrap` | compose path with no unwrap/expect/panic in emit |

## 3. CLOSED criterion

Lex §2 green; tick ADR-249 slice 2; HOLDs intact. Next: slice 3 SEQ-PIPELINE.

## Checklist

- [x] Map pins + oracles
- [x] IMPL + Lex **643/643**
- [x] Slice 3 GO → ADR-251
