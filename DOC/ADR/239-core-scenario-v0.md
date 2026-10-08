# ADR-239 — Core 0.1 scenario / acceptance v0

- **Estado:** **CLOSED** Lex **602/602** (2026-09-19) (CUT `CORE-0.1-SCENARIO-20260919`)
- **CUT-ID:** `CORE-0.1-SCENARIO-20260919`
- **Fecha:** 2026-09-19
- **Autores:** Arquitecto (pins ADR-232) · Ingeniero (IMPL slice 7)
- **Relacionados:** ADR-232; ADR-019 lang-contract (JSON/contract siguen válidos)
- **Gobernanza:** skip ≠ PASS; scenarios ejecutan E2E build+stdout

## Objetivo

`scenario` / `acceptance` **language-level** (además de `contract` JSON/lang).

## Surface IN

```
scenario name {
  acceptance "stdout-line"
}
```

Semántica v0: igual que `contract` + `expect_stdout` (entry `main`). Varias `acceptance` = varias líneas stdout esperadas (orden).

## OUT

- `requires`/`ensures` formales, fixtures argv en scenario (slice 8 ref puede ampliar)

## Oráculos

- `ejemplos/core01/11-scenario-hello.arita` → acceptance `scenario-ok`

## Checklist

- [x] pest scenario/acceptance
- [x] lower → Contract stdout
- [x] Lex measure **602/602**
