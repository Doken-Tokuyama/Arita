Translation of `251-core-seq-pipeline-v0.md`; the original is normative. / Traducción de `251-core-seq-pipeline-v0.md`; el original es el normativo.

# ADR-251 — Core 0.3 slice 3: SEQ-PIPELINE

- **Estado:** **CLOSED** Lex **646/646** (2026-09-20) · siguiente [ADR-252](252-core-scenario-compose-v0.md)
- **CUT-ID:** `CORE-0.3-SEQ-PIPELINE-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-249](249-core-0.3-pins.md) §0.1 slice 3
- **Prev:** slice 2 ERROR-PROP **CLOSED** Lex **643/643** (ADR-250)
- **HOLD:** Mutex · `[]` · insert panic · idle-kill · TLS/WS · crates.io · repair · fan-out paralelo/join

## Objective

**Sequential** A→B pipeline (max **2 hops** v0): handler calls upstream₁, then upstream₂ (or local transform + 1 hop), propagating `Result`/HTTP errors (ADR-250). No parallelism or shared mutable state.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Hops** | **≤2** chained `HttpClient` awaits (A then B) |
| **Orden** | strict seq — B only after A’s Ok (or short-circuit Err→map 250) |
| **Datos** | A’s body/text feeds B’s request (already-IN typed Text/Json) |
| **Errores** | any hop Err/4xx-5xx → ADR-250 map; **do not** continue to B if A failed |
| **OUT** | parallel `spawn`+join · Mutex buffer · >2 hops · retry loops |

## 1. Surface

No new std API. Pattern:

```text
match client.post_text(url_a, in_body) {
  Err(_) => /* 502/504 per ADR-250 */
  Ok(a) => {
    if a.status() >= 400 { /* forward per 250 */ }
    else {
      match client.post_text(url_b, a.body_text()) {
        Err(_) => /* 502/504 */
        Ok(b) => /* forward / ok */
      }
    }
  }
}
```

Suggested ref routes: `POST /pipeline` (gateway) · upstream A `/step-a` · upstream B `/step-b` (2 binds or mocks).

## 2. Oracles (Measure)

| Id | Expect |
|----|--------|
| `core03-pipe-happy` | A+B OK → 200 + composed/expected body |
| `core03-pipe-fail-a` | A down/4xx → **does not** call B (observable: B counter 0 or 502/forward A) |
| `core03-pipe-fail-b` | A OK, B down → 502/`upstream_unavailable` (or map 250) |
| `core03-pipe-seq-order` | (optional) timestamp/log A before B |
| `neg-core03-pipe-parallel` | DOC: no surface pin for parallel join; if spawn-join compose API appears → reject or OUT |

## 3. CLOSED criterion

Lex §2 green; ADR-249 tick slice 3; HOLDs intact. Next: slice 4 SCENARIO-COMPOSE.

## Checklist

- [x] Seq ≤2 hops pins + oracles
- [x] IMPL + Lex **646/646**
- [x] Slice 4 GO → ADR-252
