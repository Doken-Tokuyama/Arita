Translation of `240-core-ref-json-cli-v0.md`; the original is normative. / Traducción de `240-core-ref-json-cli-v0.md`; el original es el normativo.

# ADR-240 — Core 0.1 ref JSON CLI

- **Estado:** **CLOSED** Lex **603/603** · Core 0.1 CLOSED (2026-09-19) (CUT `CORE-0.1-REF-JSON-CLI-20260919`)
- **CUT-ID:** `CORE-0.1-REF-JSON-CLI-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 8)
- **Relacionados:** ADR-232 … ADR-239; host-core01
- **Gobernanza:** skip ≠ PASS; evidence hash; Core 0.1 CLOSED when Lex verde

## Ref program

`ejemplos/core01/ref/arita-ref-json-cli.arita`:
- Reads JSON fixture (`n`, `m`)
- `scen_sum` → print `n+m`
- `scen_prod` → print `n*m`
- ≥2 `scenario` + `acceptance` + `target`
- `[host-bridges]` → `arita-host-core01`

## Evidence

Contract/scenario attest sidecars (existing ADR-018 path) + measure oracles.

## Checklist

- [x] ref program + 2 scenarios
- [x] evidence + Lex **603/603** = Core **0.1 CLOSED**
