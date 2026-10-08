# ADR-251 — Core 0.3 slice 3: SEQ-PIPELINE

- **Estado:** **CLOSED** Lex **646/646** (2026-09-20) · siguiente [ADR-252](252-core-scenario-compose-v0.md)
- **CUT-ID:** `CORE-0.3-SEQ-PIPELINE-20260920`
- **Fecha:** 2026-09-20
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-249](249-core-0.3-pins.md) §0.1 slice 3
- **Prev:** slice 2 ERROR-PROP **CLOSED** Lex **643/643** (ADR-250)
- **HOLD:** Mutex · `[]` · insert panic · idle-kill · TLS/WS · crates.io · repair · fan-out paralelo/join

## Objetivo

Pipeline **secuencial** A→B (máx **2 hops** v0): handler llama upstream₁, luego upstream₂ (o transform local + 1 hop), propagando `Result`/HTTP errors (ADR-250). Sin paralelismo ni estado compartido mutable.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Hops** | **≤2** awaits `HttpClient` en cadena (A luego B) |
| **Orden** | estricto seq — B solo tras Ok de A (o short-circuit Err→mapa 250) |
| **Datos** | body/text de A alimenta request de B (tipado Text/Json ya IN) |
| **Errores** | cualquier hop Err/4xx-5xx → mapa ADR-250; **no** continuar B si A falló |
| **OUT** | `spawn`+join paralelo · Mutex buffer · >2 hops · retry loops |

## 1. Surface

Sin API std nueva. Pattern:

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

Rutas ref sugeridas: `POST /pipeline` (gateway) · upstream A `/step-a` · upstream B `/step-b` (2 binds o mocks).

## 2. Oracles (Measure)

| Id | Expect |
|----|--------|
| `core03-pipe-happy` | A+B OK → 200 + body compuesto/esperado |
| `core03-pipe-fail-a` | A down/4xx → **no** llama B (observable: B counter 0 o 502/forward A) |
| `core03-pipe-fail-b` | A OK, B down → 502/`upstream_unavailable` (o mapa 250) |
| `core03-pipe-seq-order` | (opcional) timestamp/log A before B |
| `neg-core03-pipe-parallel` | DOC: no pin surface para join paralelo; si aparece API spawn-join compose → reject o OUT |

## 3. Criterio CLOSED

Lex §2 verde; ADR-249 tick slice 3; HOLDs intactos. Siguiente: slice 4 SCENARIO-COMPOSE.

## Checklist

- [x] Pins seq ≤2 hops + oracles
- [x] IMPL + Lex **646/646**
- [x] Slice 4 GO → ADR-252
