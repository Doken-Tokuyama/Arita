# ADR-229 — Contrato de lenguaje: Mutex (surface + uso)

- **Estado:** **aceptada** (DOC; surface PARK hasta CUT R7 + perfil)
- **CUT-ID:** `LANG-MUTEX-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto · <person> (ACK RFC) · Ingeniero (IMPL posterior)
- **Relacionados:** RFC perfiles `safe`/`service`; ADR-225 R7; ADR-038 Mutex×await PARK; async OUT v0.1
- **Gobernanza:** Contrato de uso + emit; formal proofs opcionales solo `high-assurance`.

## Contexto

Interior mutability sin contrato = theater (lock olvidado, lock+await, data race vía shared).

## Decisión

### 1. v0.1 (ahora)

| Item | Pin |
|------|-----|
| Ident `Mutex` / path `std::sync::Mutex` en surface | **E0312** `Mutex not available in this profile/surface` |
| Measure | `neg-e0312-mutex` — expect_reject estable |
| async / await | siguen E0240/E0241/E0242; **sin** Mutex×await hasta §3 |

Esto cierra gap **R7** (hoy solo PARK DOC).

### 2. Unpark futuro (Núcleo 2 / perfil `service` o ADR dedicado)

Solo tras R7 verde + GO:

| Surface (borrador) | Regla |
|--------------------|-------|
| `Mutex<T>` | tipo explícito; create/lock/unlock nombrables |
| `lock` | retorna guard o `Result`; **no** panicking lock como éxito |
| Hold across `await` | **E0313** (extiende ADR-038) — illegal |
| Perfil `safe` | Mutex **OUT** o restringido; default sin threads |
| Emit | adapters auditados; `forbid(unsafe)` en user code |

### 3. OUT v0.1

- IMPL Mutex ahora
- Threads / channels
- Formal deadlock proofs (high-assurance opt-in más tarde)

## Checklist

- [x] Pins E0312 neg + unpark diferido
- [ ] CUT R7 `neg-e0312-mutex`
- [ ] ADR surface Mutex Núcleo 2 (posterior)

## Cola

CUT `EVIDENCE-R7-MUTEX-NEG-20260919` tras R4 y R0/R2.
