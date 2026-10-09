Translation of `228-lang-contract-insert.md`; the original is normative. / Traducción de `228-lang-contract-insert.md`; el original es el normativo.

# ADR-228 — Language contract: Insert (fallible)

- **Estado:** **aceptada** (DOC; IMPL tras evidencia R5 + GO Núcleo 1)
- **CUT-ID:** `LANG-INSERT-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto · <person> (ACK RFC) · Ingeniero (IMPL posterior)
- **Relacionados:** RFC rev. 2 Núcleo 1; ADR-225 R5; ADR-036; ADR-222 append; ADR-224 extend; ADR-227
- **Gobernanza:** Fallible/total en lenguaje; **no** burocracia formal. skip ≠ PASS.

## Context

Rust `Vec::insert` panics if `index > len`. ARITA forbids partials with an invisible precondition.

## Decision

### 1. Surface IN (Core 1, post R5)

| Surface | Receiver | Args | Ret | Semantics |
|---------|----------|------|-----|-----------|
| `v.insert(i, x)` | `mut Vec<T>` | `Int i`, `T x` | **`Result<(), Int>`** | OK insert at `i` if `0 ≤ i ≤ len`; else `Err(code)` |

`Err` codes (pin v0, Int):

| Code | Meaning |
|------|---------|
| `0` | OOB runtime (`i > len` or non-lit `i < 0`) |
| — | lit `i < 0` → **compile** **E0311** `negative insert index` (no defer) |

Orientative emit:

- lit `i < 0` → E0311 (no emit)
- else bounds-check → `Ok(())` + insert, or `Err(0)` — **never** bare `Vec::insert` that can panic
- Helper allowed: `__arita_vec_insert` that returns `Result` and **does not** panic

### 2. OUT v0

- `insert` that returns `()` and panics
- `String.insert` / `drain` / closures (`retain`/`map`) — separate ADR
- `insert` without `mut` → E0202
- Formal `requires i <= len` as the **only** gate (optional high-assurance; does not replace Result)

### 3. Prior evidence (R5)

Before whitelisting `insert`:

| Id | Expect |
|----|--------|
| `neg-e0206-insert-park` | today: method not whitelisted → **E0206** (until unpark) |
| post-unpark: `std-vec-insert-ok` / `std-vec-insert-err-oob` / `neg-e0311-insert-neg` | accepted / E0311 |

### 4. Relation to append/extend

- `append` drains other; `extend` copies; `insert` shifts at an index — do not confuse (distinct oracles).

## Checklist

- [x] Result + E0311 + no-panic emit pins
- [ ] CUT R5 dedicated neg-E0206 for insert (ADR-230)
- [ ] GO IMPL insert Core 1 after R5

## Queue

R5 evidence after R4 and R0/R2; then GO IMPL `STD-VEC-INSERT-…`.
