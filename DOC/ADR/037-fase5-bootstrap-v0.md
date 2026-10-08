# ADR-037 — Fase 5 bootstrap v0 (frontend parts in ARITA)

- **Estado:** **aceptada** + **verified** (Lex measure **77** accepted + **2** gated targets; CUT `BOOTSTRAP-V0-20260914`)
- **CUT-ID:** `BOOTSTRAP-V0-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (DOC/pins) + Ingeniero Rust (IMPL executor)
- **Relacionados:** ROADMAP Fase 5, ADR-001, ADR-022, ADR-033/035, ADR-034 verified
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Sin self-host.
- **Barra:** dual-oracle medible; acceptance de suite = `arita measure`; skip ≠ PASS.

## Contexto

ROADMAP Fase 5 (opcional): *partes del frontend en ARITA*. Compilador SoT = **Rust**. Bootstrap ≠ reescribir pest/HIR/codegen.

## Decisión (pins GO 2026-09-14)

### 1. IN v0

| Pieza | Regla |
|-------|--------|
| Slice | **Dual-oracle** de **una fn pura concreta** (misma lógica en Rust + ARITA). **No** catálogo E0xxx v0 |
| Invocación | **Solo measure** + `cargo test` Rust del mismo algoritmo. Sin CLI `arita bootstrap-check` v0 |
| Golden SoT | **Constante documentada** (stdout / `assert` esperado); Rust test y oráculo ARITA **deben coincidir** |
| Acceptance | suite = **`arita measure`** (Rust test = evidencia auxiliar, no sustituye measure) |
| Layout | `ejemplos/bootstrap/` |
| Safe-only | ADR-022 |

### 2. Oráculos (IMPL)

| Id | Expect |
|----|--------|
| `bootstrap-01` (nombre exacto Ingeniero) | ARITA corre; stdout/assert = constante documentada |
| Rust `cargo test` | mismo resultado que la constante |
| opc. neg | theater → E021x existente |

### 3. OUT v0

- Self-host / rewrite pest / HIR / codegen en ARITA
- Catálogo E0xxx como slice v0
- CLI `arita bootstrap-check`
- Dual-impl decorativo sin measure
- Afirmar “compilador en ARITA”

## Decisiones abiertas

**Cerradas** (GO Ingeniero):

1. ~~Slice~~ → dual-oracle fn pura concreta (no E0xxx catalog)
2. ~~Invocación~~ → solo measure + cargo test; sin CLI
3. ~~Golden~~ → constante documentada; Rust ≡ ARITA; acceptance = measure

## Checklist GO

- [x] Slice + invocación + golden OK
- [x] OUT self-host claro OK
- [x] Abiertas 1–3 cerradas
- [x] ADR **aceptada** + IMPL GO
- [x] Landed + measure (Ingeniero)

## IMPL evidence (`BOOTSTRAP-V0-20260914`)

| Id | Path | Expect | Verdict |
|----|------|--------|---------|
| `bootstrap-01` | `ejemplos/bootstrap/01-fact.arita` | E2E stdout `120` (`fact5` unrolled 1‥5) | accepted (measure) |
| Rust `cargo test -p arita-cli bootstrap` | `crates/arita-cli/src/bootstrap.rs` | `fact5() == 120` (`GOLDEN_FACT5`) | pass (aux) |

**Algorithm:** factorial-lite `fact5` = `1*2*3*4*5` (unrolled; Int helpers = Expr/Let only).  
**Golden:** `120`. No CLI `arita bootstrap-check`. Parser/Codegen HOLD.

HOLD crates → review-only post verify.

## Addendum

**ADR-042** bootstrap-02 (`sum5→15`, CUT `BOOTSTRAP-02-20260915`) — **verified** Lex **83**+2 gated; segunda dual-oracle; no self-host.
