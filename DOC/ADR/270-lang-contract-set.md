# ADR-270 — Contrato de lenguaje: Set (fallible write)

- **Estado:** **aceptada** (DOC; IMPL vía Core 0.6 slice SET-FALLIBLE)
- **CUT-ID:** `LANG-SET-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto · GO <person> post-0.5 · Ingeniero (IMPL posterior)
- **Relacionados:** ADR-225 R3/R4; ADR-227 IndexGet; ADR-228 Insert; Core 0.6 [ADR-264](264-core-0.6-pins.md)
- **Gobernanza:** Fallible/total; skip ≠ PASS. **No** desbloquea IndexMut assign (`v[i] = x`).

## Contexto

Tras IndexGet sugar (read → Option) e Insert (write-displace → Result), falta **escritura en índice** sin panic ni IndexMut. Rust `v[i] = x` / `IndexMut` paniquea OOB — ilegal en ARITA sin contrato.

## Decisión

### 1. Surface IN (Core 0.6)

| Surface | Receiver | Args | Ret | Semántica |
|---------|----------|------|-----|-----------|
| `v.set(i, x)` | `mut Vec<T>` / `mut List<T>` | `Int i`, `T x` | **`Result<(), Int>`** | OK replace at `i` si `0 ≤ i < len`; else `Err(0)` |

| Code | Meaning |
|------|---------|
| `0` | OOB runtime (`i ≥ len` o `i < 0` no-lit) |
| — | lit `i < 0` → **E0319** `negative set index` (compile; no defer) |

Emit:

- lit `i < 0` → E0319 (no emit)
- else bounds-check → assign + `Ok(())` o `Err(0)` — **nunca** `IndexMut` / `v[i] =` panic path
- Helper OK: `__arita_vec_set` (o equiv.) sin panic

### 2. OUT v0

- `v[i] = x` / IndexMut sugar (**sigue HOLD** / diag estable, p.ej. E0314)
- `set` → `()` paniqueando
- `String.set` / Map.set por índice numérico (Map usa `put` / sugar key — ADR-266)
- formal-only gate sin Result

### 3. Relación get / insert / []

- `get` / `[]` → **Option** (lectura; 227/261)
- `insert` → Result + **desplaza** (228/260)
- `set` → Result + **reemplaza** in-bounds (este ADR); no crece len
- `push` / `put` siguen IN previos


### Pin E0319 (set-neg lit)

**E0319** `negative set index` — *solo* `set` con índice literal negativo (`v.set(-1, x)`).
Neg oracle: `neg-core06-set-neg-lit` → **E0319**.

**E0315** no se toca: SoT = `missing field` ([ADR-233](233-core-record-v0.md) + HIR + `neg-e0315-missing-field`). Prohibido reasignar E0315 a set-neg.


## Checklist

- [x] Pins set Result + E0319 + emit no-panic + IndexMut OUT
- [x] IMPL Core 0.6 SET-FALLIBLE (ADR-265) — CLOSED Lex **710/710**
