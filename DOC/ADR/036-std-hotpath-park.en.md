Translation of `036-std-hotpath-park.md`; the original is normative. / Traducción de `036-std-hotpath-park.md`; el original es el normativo.

# ADR-036 — Std hot-path v0 — PARK

- **Estado:** **park parcial** — `clear`→049, `pop`→052; insert/index siguen park
- **CUT-ID:** `STD-HOTPATH-PARK-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto
- **Relacionados:** ADR-026 (std minima), ROADMAP 4b “APIs std hot paths”, `PERF-HOST-BORDER.md`

## Decision

**Park.** No hay API std segura clear *nueva* for a CUT hot-path v0 without opening OUT of ADR-026:

- Index/slice/`get` → OUT (bounds / panic theater)
- `clear`/`pop`/`insert` → ownership + mut semantics; no “hot path” measurable without timed thresholds (OUT PERF-V0)
- `len`/`is_empty`/`push`/`Vec::new` → **ya** en ADR-026 y medidos

Hot-path v0 = **la whitelist 026 existente** + perfiles release (028). Ampliar std = CUT dedicated with oracles E2E no-timed (p.ej. `clear` + `len==0`) when Ingeniero priorice.

## Alternativa ROADMAP

1. Follow 026 + 028 as std perf surface.
2. ~~Next CUT `clear`~~ → **ADR-049** `STD-CLEAR-20260918` (DOC accepted; IMPL post E0272).
3. Do not open index/slice without a separate threat model.

## Checklist

- [x] Park justificado
- [x] Alternativa documentada
- [x] Unpark `clear` → **ADR-049** / CUT `STD-CLEAR-20260918`
- [x] Unpark `pop` → **ADR-052** / CUT `STD-POP-20260918` (**verified** Lex **108/108**; insert/index PARK)
