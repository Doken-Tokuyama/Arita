# ADR-228 — Contrato de lenguaje: Insert (fallible)

- **Estado:** **aceptada** (DOC; IMPL tras evidencia R5 + GO Núcleo 1)
- **CUT-ID:** `LANG-INSERT-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto · <person> (ACK RFC) · Ingeniero (IMPL posterior)
- **Relacionados:** RFC rev. 2 Núcleo 1; ADR-225 R5; ADR-036; ADR-222 append; ADR-224 extend; ADR-227
- **Gobernanza:** Fallible/total en lenguaje; **no** burocracia formal. skip ≠ PASS.

## Contexto

Rust `Vec::insert` paniquea si `index > len`. ARITA prohíbe parciales con precondición invisible.

## Decisión

### 1. Surface IN (Núcleo 1, post R5)

| Surface | Receiver | Args | Ret | Semántica |
|---------|----------|------|-----|-----------|
| `v.insert(i, x)` | `mut Vec<T>` | `Int i`, `T x` | **`Result<(), Int>`** | OK insert at `i` si `0 ≤ i ≤ len`; else `Err(code)` |

Códigos `Err` (pin v0, Int):

| Code | Meaning |
|------|---------|
| `0` | OOB runtime (`i > len` o `i < 0` no-lit) |
| — | lit `i < 0` → **compile** **E0311** `negative insert index` (no defer) |

Emit orientativo:

- lit `i < 0` → E0311 (no emit)
- else bounds-check → `Ok(())` + insert, o `Err(0)` — **nunca** `Vec::insert` bare que pueda panic
- Helper permitido: `__arita_vec_insert` que retorna `Result` y **no** paniquea

### 2. OUT v0

- `insert` que retorne `()` y paniquee
- `String.insert` / `drain` / closures (`retain`/`map`) — ADR aparte
- `insert` sin `mut` → E0202
- Formal `requires i <= len` como **único** gate (opcional high-assurance; no sustituye Result)

### 3. Evidencia previa (R5)

Antes de whitelist `insert`:

| Id | Expect |
|----|--------|
| `neg-e0206-insert-park` | hoy: método no whitelisted → **E0206** (hasta unpark) |
| post-unpark: `std-vec-insert-ok` / `std-vec-insert-err-oob` / `neg-e0311-insert-neg` | accepted / E0311 |

### 4. Relación append/extend

- `append` drena other; `extend` copia; `insert` desplaza en índice — no confundir (oráculos distintos).

## Checklist

- [x] Pins Result + E0311 + emit no-panic
- [ ] CUT R5 neg-E0206 dedicado insert (ADR-230)
- [ ] GO IMPL insert Núcleo 1 tras R5

## Cola

Evidencia R5 tras R4 y R0/R2; luego GO IMPL `STD-VEC-INSERT-…`.
