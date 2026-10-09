Translation of `227-lang-contract-indexget.md`; the original is normative. / Traducción de `227-lang-contract-indexget.md`; el original es el normativo.

# ADR-227 — Language contract: IndexGet (fallible/total)

- **Estado:** **aceptada** (DOC; IMPL por CUT evidencia / Núcleo 1)
- **CUT-ID:** `LANG-INDEXGET-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins) · <person> (ACK RFC rev. 2) · Ingeniero (IMPL posterior)
- **Relacionados:** RFC AI-native rev. 2; ADR-225 R4; ADR-085 `get`→Option; ADR-036 park; ADR-226
- **Gobernanza:** Contratos **de lenguaje** (semántica + emit), **no** `requires`/`ensures` formales. skip ≠ PASS.

## Context

RFC: indexing **must not** emit panicking `values[i]`. Core 1 requires **fallible** index. ADR-085 already pinned `get`→Option.

## Decision

### 1. Canonical (IN today)

| Surface | Ret | Semantics | Emit |
|---------|-----|-----------|------|
| `v.get(i)` / `s.get(i)` | `Option<T>` | OOB / `i < 0` → `None` | `.get(usize)` / None — **never** `v[i]` |

ADR-085 remains the canonical API.

### 2. `[]` operator (two phases)

**Phase A — evidence (CUT R4, mandatory before sugar):**

- `v[i]` / `s[i]` on surface → stable reject **E0310** `indexing operator not allowed` (canonical EN text).
- Measure: `neg-e0310-index-vec`, `neg-e0310-index-string` — reproducible parse/check fail (not “inconsistent parse”).
- Emit grep: **zero** `][` / `.index(` / `unwrap` on index paths.

**Phase B — Core 1 sugar (only after Phase A green + Arquitecto GO):**

- `v[i]` / `s[i]` desugar to the **same** semantics as `get` → `Option<T>` (total/fallible).
- Emit identical to `get`. **Forbidden** to emit Rust `Index`/`IndexMut`.
- `get_mut` / slices / `IndexMut` → **OUT** until a separate ADR.

### 3. OUT

- OOB panic as a success channel
- Formal `requires` for index (optional only in `high-assurance`, does not unlock surface)
- Emit `values[i]` / `panic!` in index helpers

### 4. Oracles

| Id | Kind | Phase |
|----|------|------|
| `neg-e0310-index-vec` | expect_reject E0310 | A (GO now) |
| `neg-e0310-index-string` | expect_reject E0310 | A |
| `emit-no-panic-index` (grep/CI) | fail if emit uses panicking `[]` | A/B |
| reuse `std-vec-get-*` | accepted | already |

## Checklist

- [x] Canonical pins = get; `[]` = E0310 then Option sugar
- [x] Evidence CUT R4 (Phase A) — **CLOSED** Lex **583/583**
- [x] Phase B Vec/String sugar — **CLOSED** Core 0.5 ([ADR-261](261-core-index-sugar-v0.en.md))
- [x] Map `m[k]` → get/Option extension — **CLOSED** Lex **715/715** ([ADR-266](266-core-map-index-v0.en.md))

## Queue

R4 CLOSED. Phase B Vec/String: [ADR-261](261-core-index-sugar-v0.en.md). Map sugar: [ADR-266](266-core-map-index-v0.en.md) CLOSED Lex **715/715**. IndexMut assign remains HOLD.
