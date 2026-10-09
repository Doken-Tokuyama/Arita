Translation of `229-lang-contract-mutex.md`; the original is normative. / Traducción de `229-lang-contract-mutex.md`; el original es el normativo.

# ADR-229 — Language contract: Mutex (surface + use)

- **Estado:** **aceptada** (DOC; surface PARK hasta CUT R7 + perfil)
- **CUT-ID:** `LANG-MUTEX-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto · <person> (ACK RFC) · Ingeniero (IMPL posterior)
- **Relacionados:** RFC perfiles `safe`/`service`; ADR-225 R7; ADR-038 Mutex×await PARK; async OUT v0.1
- **Gobernanza:** Contrato de uso + emit; formal proofs opcionales solo `high-assurance`.

## Context

Interior mutability without a contract = theater (forgotten lock, lock+await, data race via shared).

## Decision

### 1. v0.1 (now)

| Item | Pin |
|------|-----|
| Ident `Mutex` / path `std::sync::Mutex` on surface | **E0312** `Mutex not available in this profile/surface` |
| Measure | `neg-e0312-mutex` — stable expect_reject |
| async / await | still E0240/E0241/E0242; **no** Mutex×await until §3 |

This closes the **R7** gap (today only PARK DOC).

### 2. Future unpark (Core 2 / `service` profile or dedicated ADR)

Only after R7 green + GO:

| Surface (draft) | Rule |
|--------------------|-------|
| `Mutex<T>` | explicit type; nameable create/lock/unlock |
| `lock` | returns guard or `Result`; **not** a panicking lock as success |
| Hold across `await` | **E0313** (extends ADR-038) — illegal |
| `safe` profile | Mutex **OUT** or restricted; default without threads |
| Emit | audited adapters; `forbid(unsafe)` in user code |

### 3. OUT v0.1

- Mutex IMPL now
- Threads / channels
- Formal deadlock proofs (high-assurance opt-in later)

## Checklist

- [x] E0312 neg pins + deferred unpark
- [ ] CUT R7 `neg-e0312-mutex`
- [ ] Mutex surface ADR Core 2 (later)

## Queue

CUT `EVIDENCE-R7-MUTEX-NEG-20260919` after R4 and R0/R2.
