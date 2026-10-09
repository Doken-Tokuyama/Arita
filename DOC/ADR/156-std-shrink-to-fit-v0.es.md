Translation of `156-std-shrink-to-fit-v0.md`; the original is normative. / Traducción de `156-std-shrink-to-fit-v0.md`; el original es el normativo.

# ADR-156 — String/Vec `shrink_to_fit()` v0

- **Estado:** **aceptada** + **verified** Lex **407/407**
- **CUT-ID:** `STD-SHRINK-TO-FIT-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins) · Ingeniero (IMPL)
- **Relacionados:** ADR-154 reserve; ADR-153 capacity; ADR-049 clear
- **Gobernanza:** Sin idle; shrink_to/reserve_exact OUT; Mutex/`[]` PARK.
- **Barra:** E2E measure; skip ≠ PASS.
- Mirrors `_mirror_156.tgz`.

## IN v0

| Surface | Ret | Emit |
|---------|-----|------|
| `mut s.shrink_to_fit()` / `mut v.shrink_to_fit()` | **()** | `.shrink_to_fit()` |

Arity 0; sin mut → E0202; tipo incorrecto → E0206.

## Oráculos (Lex 407/407)

| Id | Expect |
|----|--------|
| `std-shrink-to-fit-string` | reserve+shrink → capacity≥len **accepted** |
| `std-shrink-to-fit-vec` | reserve+shrink → capacity≥len **accepted** |
| `neg-e0202-shrink-to-fit-not-mut` | **E0202** |
| `neg-e0206-shrink-to-fit-int` | **E0206** |

## Checklist

- [x] Pins + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **407/407** (baseline 403/403)
