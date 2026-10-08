# ADR-036 — Std hot-path v0 — PARK

- **Estado:** **park parcial** — `clear`→049, `pop`→052; insert/index siguen park
- **CUT-ID:** `STD-HOTPATH-PARK-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto
- **Relacionados:** ADR-026 (std minima), ROADMAP 4b “APIs std hot paths”, `PERF-HOST-BORDER.md`

## Decisión

**Park.** No hay API std segura clara *nueva* para un CUT hot-path v0 sin abrir OUT de ADR-026:

- Index/slice/`get` → OUT (bounds / panic theater)
- `clear`/`pop`/`insert` → ownership + mut semantics; no “hot path” medible sin umbrales timed (OUT PERF-V0)
- `len`/`is_empty`/`push`/`Vec::new` → **ya** en ADR-026 y medidos

Hot-path v0 = **la whitelist 026 existente** + perfiles release (028). Ampliar std = CUT dedicado con oráculos E2E no-timed (p.ej. `clear` + `len==0`) cuando Ingeniero priorice.

## Alternativa ROADMAP

1. Seguir 026 + 028 como superficie perf std.
2. ~~Próximo CUT `clear`~~ → **ADR-049** `STD-CLEAR-20260918` (DOC aceptada; IMPL post E0272).
3. No abrir index/slice sin threat model aparte.

## Checklist

- [x] Park justificado
- [x] Alternativa documentada
- [x] Unpark `clear` → **ADR-049** / CUT `STD-CLEAR-20260918`
- [x] Unpark `pop` → **ADR-052** / CUT `STD-POP-20260918` (**verified** Lex **108/108**; insert/index PARK)
