# ADR-227 — Contrato de lenguaje: IndexGet (fallible/total)

- **Estado:** **aceptada** (DOC; IMPL por CUT evidencia / Núcleo 1)
- **CUT-ID:** `LANG-INDEXGET-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins) · <person> (ACK RFC rev. 2) · Ingeniero (IMPL posterior)
- **Relacionados:** RFC AI-native rev. 2; ADR-225 R4; ADR-085 `get`→Option; ADR-036 park; ADR-226
- **Gobernanza:** Contratos **de lenguaje** (semántica + emit), **no** `requires`/`ensures` formales. skip ≠ PASS.

## Contexto

RFC: indexación **no** emite `values[i]` panic. Núcleo 1 pide index **fallible**. ADR-085 ya pinó `get`→Option.

## Decisión

### 1. Canónico (IN hoy)

| Surface | Ret | Semántica | Emit |
|---------|-----|-----------|------|
| `v.get(i)` / `s.get(i)` | `Option<T>` | OOB / `i < 0` → `None` | `.get(usize)` / None — **nunca** `v[i]` |

ADR-085 permanece el API canónico.

### 2. Operador `[]` (dos fases)

**Fase A — evidencia (CUT R4, obligatorio antes de sugar):**

- `v[i]` / `s[i]` en surface → rechazo estable **E0310** `indexing operator not allowed` (texto EN canónico).
- Measure: `neg-e0310-index-vec`, `neg-e0310-index-string` — parse/check fail reproducible (no “parse inconsistente”).
- Grep emit: **cero** `][` / `.index(` / `unwrap` en paths de index.

**Fase B — sugar Núcleo 1 (solo tras Fase A verde + GO Arquitecto):**

- `v[i]` / `s[i]` desugar a la **misma** semántica que `get` → `Option<T>` (total/fallible).
- Emit idéntico a `get`. **Prohibido** emitir `Index`/`IndexMut` de Rust.
- `get_mut` / slices / `IndexMut` → **OUT** hasta ADR aparte.

### 3. OUT

- Panic OOB como canal de éxito
- `requires` formal para index (opcional solo en `high-assurance`, no desbloquea surface)
- Emit `values[i]` / `panic!` en helpers de index

### 4. Oráculos

| Id | Kind | Fase |
|----|------|------|
| `neg-e0310-index-vec` | expect_reject E0310 | A (GO ahora) |
| `neg-e0310-index-string` | expect_reject E0310 | A |
| `emit-no-panic-index` (grep/CI) | fail si emit usa `[]` panic | A/B |
| reusar `std-vec-get-*` | accepted | ya |

## Checklist

- [x] Pins canónico = get; `[]` = E0310 luego sugar Option
- [x] CUT evidencia R4 (Fase A) — **CLOSED** Lex **583/583**
- [x] Fase B sugar Vec/String — **CLOSED** Core 0.5 ([ADR-261](261-core-index-sugar-v0.md))
- [x] Extensión Map `m[k]` → get/Option — **CLOSED** Lex **715/715** ([ADR-266](266-core-map-index-v0.md))

## Cola

R4 CLOSED. Fase B Vec/String: [ADR-261](261-core-index-sugar-v0.md). Map sugar: [ADR-266](266-core-map-index-v0.md) CLOSED Lex **715/715**. IndexMut assign sigue HOLD.
