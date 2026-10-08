# ADR-022 — ARITA is safe-only

- **Estado:** **aceptada** (hard rule <person> 2026-09-13)
- **CUT-ID:** `SAFE-ONLY-20260913`
- **Fecha:** 2026-09-13
- **Autores:** Ingeniero Rust (GO) · Arquitecto (align)
- **Relacionados:** ADR-005/006/009, GOALS-NONGOALS, THREAT_MODEL, ADR-021

## Decisión

Todo programa ARITA es **safe**. No hay escape hatch en el dialecto que medimos.

1. **Surface:** no keyword/`unsafe`, no raw pointers, no FFI, no transmute, no lifetimes explícitos como puerta a unsafety.
2. **Emit:** Rust generado para programas de usuario lleva `#![forbid(unsafe_code)]` (o equivalente por crate); el backend no emite `unsafe` para código de usuario.
3. **Checker:** ownership/borrow + types rechazan patrones unsound; measure + Miri (ADR-021) son jueces, no theater.
4. **OUT explícito:** cualquier CUT que introduzca `unsafe` en surface o emit de usuario → **reject** / no merge a SoT.

## Nota sobre docs antiguos

`01-GOALS-NONGOALS.md` hablaba de “unsafe explícito (fase posterior)”. Esa fase queda **fuera del producto** bajo esta regla hasta nueva decisión explícita de <person>. Safe-by-default pasa a **safe-only**.

## Consecuencias

- Codegen / Parser / HIR: rechazar y diagnosticar intentos de unsafety.
- Measure: oráculos negativos si aparece `unsafe` en emit (añadir cuando haya gancho estable).
- Logic island: sin bypass vía efectos ocultos; `Io` explícito sigue siendo el canal de efectos.

## Alineación Arquitecto (con ADRs ownership / emit)

| Fuente | Qué decía | Bajo ADR-022 |
|--------|-----------|--------------|
| ADR-006 § ownership | No `unsafe`, no raw pointers (F2 OUT) | **Elevado a hard rule de producto** (todas las fases medidas) |
| ADR-009 | Sin `unsafe` / raw / lifetimes explícitos en F2 | Igual; checker rechaza unsound; no es escape hatch |
| ADR-001 emit-Rust §4 | `forbid(unsafe_code)` *salvo* módulos `unsafe` de dialecto (fase tardía) | **Superseded:** sin módulos `unsafe` de usuario; forbid siempre en emit de programas |
| ADR-021 Miri | Oracle UB en lib crates | Juez complementario; no sustituye forbid en emit de usuario |
| THREAT_MODEL | Bypass vía `unsafe`/FFI | Mitigación = este ADR + measure |

**Diagnóstico draft (si hace falta CUT IMPL):** `E0231` — `unsafe / FFI not allowed in ARITA` (surface o emit de usuario). **No usar E0230** (ya es CONTRACT-TARGET-FN / ADR-020). Códigos exactos = CUT aparte; no inventar PASS.

**Gobernanza:** cualquier CUT surface/emit que introduzca `unsafe` → reject (Ingeniero). Cambio de esta regla = decisión explícita de <person> + nuevo ADR.

