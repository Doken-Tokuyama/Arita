Translation of `270-lang-contract-set.md`; the original is normative. / Traducción de `270-lang-contract-set.md`; el original es el normativo.

# ADR-270 — Language contract: Set (fallible write)

- **Estado:** **aceptada** (DOC; IMPL vía Core 0.6 slice SET-FALLIBLE)
- **CUT-ID:** `LANG-SET-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto · GO <person> post-0.5 · Ingeniero (IMPL posterior)
- **Relacionados:** ADR-225 R3/R4; ADR-227 IndexGet; ADR-228 Insert; Core 0.6 [ADR-264](264-core-0.6-pins.md)
- **Gobernanza:** Fallible/total; skip ≠ PASS. **No** desbloquea IndexMut assign (`v[i] = x`).

## Context

After IndexGet sugar (read → Option) and Insert (write-displace → Result), **index write** without panic or IndexMut is still missing. Rust `v[i] = x` / `IndexMut` panics on OOB — illegal in ARITA without a contract.

## Decision

### 1. Surface IN (Core 0.6)

| Surface | Receiver | Args | Ret | Semantics |
|---------|----------|------|-----|-----------|
| `v.set(i, x)` | `mut Vec<T>` / `mut List<T>` | `Int i`, `T x` | **`Result<(), Int>`** | OK replace at `i` if `0 ≤ i < len`; else `Err(0)` |

| Code | Meaning |
|------|---------|
| `0` | OOB runtime (`i ≥ len` o `i < 0` no-lit) |
| — | lit `i < 0` → **E0319** `negative set index` (compile; no defer) |

Emit:

- lit `i < 0` → E0319 (no emit)
- else bounds-check → assign + `Ok(())` o `Err(0)` — **nunca** `IndexMut` / `v[i] =` panic path
- Helper OK: `__arita_vec_set` (or equiv.) without panic

### 2. OUT v0

- `v[i] = x` / IndexMut sugar (**stays HOLD** / stable diag, e.g. E0314)
- `set` → panicking `()`
- `String.set` / Map.set by numeric index (Map uses `put` / key sugar — ADR-266)
- formal-only gate without Result

### 3. Relation get / insert / []

- `get` / `[]` → **Option** (read; 227/261)
- `insert` → Result + **shifts** (228/260)
- `set` → Result + **replaces** in-bounds (this ADR); does not grow len
- `push` / `put` stay prior IN


### E0319 pin (set-neg lit)

**E0319** `negative set index` — *only* `set` with a negative literal index (`v.set(-1, x)`).
Neg oracle: `neg-core06-set-neg-lit` → **E0319**.

**E0315** is not touched: SoT = `missing field` ([ADR-233](233-core-record-v0.en.md) + HIR + `neg-e0315-missing-field`). Forbidden to reassign E0315 to set-neg.


## Checklist

- [x] set Result pins + E0319 + no-panic emit + IndexMut OUT
- [x] Core 0.6 SET-FALLIBLE IMPL (ADR-265) — CLOSED Lex **710/710**
