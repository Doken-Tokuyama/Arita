Translation of `239-core-scenario-v0.md`; the original is normative. / Traducción de `239-core-scenario-v0.md`; el original es el normativo.

# ADR-239 — Core 0.1 scenario / acceptance v0

- **Estado:** **CLOSED** Lex **602/602** (2026-09-19) (CUT `CORE-0.1-SCENARIO-20260919`)
- **CUT-ID:** `CORE-0.1-SCENARIO-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 7)
- **Relacionados:** ADR-232; ADR-019 lang-contract (JSON/contract siguen válidos)
- **Gobernanza:** skip ≠ PASS; scenarios ejecutan E2E build+stdout

## Objective

Language-level `scenario` / `acceptance` (besides JSON/lang `contract`).

## Surface IN

```
scenario name {
  acceptance "stdout-line"
}
```

v0 semantics: same as `contract` + `expect_stdout` (entry `main`). Several `acceptance` = several expected stdout lines (order).

## OUT

- Formal `requires`/`ensures`, argv fixtures in scenario (slice 8 ref may widen)

## Oracles

- `ejemplos/core01/11-scenario-hello.arita` → acceptance `scenario-ok`

## Checklist

- [x] pest scenario/acceptance
- [x] lower → Contract stdout
- [x] Lex measure **602/602**
