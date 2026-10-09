Translation of `037-fase5-bootstrap-v0.md`; the original is normative. / Traducción de `037-fase5-bootstrap-v0.md`; el original es el normativo.

# ADR-037 — Phase 5 bootstrap v0 (frontend parts in ARITA)

- **Estado:** **aceptada** + **verified** (Lex measure **77** accepted + **2** gated targets; CUT `BOOTSTRAP-V0-20260914`)
- **CUT-ID:** `BOOTSTRAP-V0-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (DOC/pins) + Ingeniero Rust (IMPL executor)
- **Relacionados:** ROADMAP Fase 5, ADR-001, ADR-022, ADR-033/035, ADR-034 verified
- **Gobernanza:** **aceptada** + **verified**. HOLD crates → review-only. Sin self-host.
- **Barra:** dual-oracle medible; acceptance de suite = `arita measure`; skip ≠ PASS.

## Context

ROADMAP Phase 5 (optional): *partes of the frontend en ARITA*. Compilador SoT = **Rust**. Bootstrap ≠ reescribir pest/HIR/codegen.

## Decision (pins GO 2026-09-14)

### 1. IN v0

| Pieza | Regla |
|-------|--------|
| Slice | **Dual-oracle** de **one concrete pure fn** (same logic en Rust + ARITA). **No** catalog E0xxx v0 |
| Invocation | **Solo measure** + `cargo test` Rust of the same algorithm. No CLI `arita bootstrap-check` v0 |
| Golden SoT | **Documented constant** (stdout / `assert` expected); Rust test y oracle ARITA **must match** |
| Acceptance | suite = **`arita measure`** (Rust test = evidence auxiliar, no sustituye measure) |
| Layout | `ejemplos/bootstrap/` |
| Safe-only | ADR-022 |

### 2. Oracles (IMPL)

| Id | Expect |
|----|--------|
| `bootstrap-01` (nombre exacto Ingeniero) | ARITA corre; stdout/assert = constante documentada |
| Rust `cargo test` | same resultado que la constante |
| opc. neg | theater → E021x existente |

### 3. OUT v0

- Self-host / rewrite pest / HIR / codegen en ARITA
- Catalog E0xxx as v0 slice
- CLI `arita bootstrap-check`
- Decorative dual-impl without measure
- Afirmar “compilador en ARITA”

## Decisiones abiertas

**Cerradas** (GO Ingeniero):

1. ~~Slice~~ → dual-oracle fn pura concreta (no E0xxx catalog)
2. ~~Invocation~~ → only measure + cargo test; no CLI
3. ~~Golden~~ → constante documentada; Rust ≡ ARITA; acceptance = measure

## Checklist GO

- [x] Slice + invocation + golden OK
- [x] OUT self-host claro OK
- [x] Abiertas 1–3 cerradas
- [x] ADR **accepted** + IMPL GO
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
