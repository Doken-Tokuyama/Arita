Translation of `011-arita-hir-scaffold.md`; the original is normative. / Traducción de `011-arita-hir-scaffold.md`; el original es el normativo.

# ADR-011 — Scaffold crate `arita-hir` (ownership future)

- **Estado:** **aceptada** (DOC-only)
- **CUT-ID:** `HIR-SCAFFOLD-DOC-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO DOC)
- **DOC CUT:** `HIR-SCAFFOLD-DOC-20260913` accepted (design).
- **TYPES CUT:** `HIR-TYPES-20260913` **verified** — crate `arita-hir` existe (mirror AST + `lower_ast`); **NOT** checker; **no emit**
- **Relacionados:** ADR-003 (workspace), ADR-006 (ownership rules), ADR-007 (AST F2), ADR-009 (borrow notes DOC-only), ADR-001 (emit-Rust), `THREAT_MODEL.md`
- **Gobernanza:** **DOC only first**. **No** fake checker. **No** PASS de ownership hasta oráculos **E0201/E0202**.

## Context

ADR-009 documents the move/borrow contract but **forbids** claiming real ownership without HIR. The workspace now includes `arita-hir` (types-only). Production pipeline remains AST→codegen until an emit/HIR CUT.

This ADR introduces the **crate** `arita-hir` as a module boundary for the future checker — first in DOC; scaffold IMPL **only** after GO, and **without** an “empty check that always passes” disguised as ownership.

## Decision (proposal)

### 1. Metas of the HIR (v0 path)

| Meta | Notas |
|------|--------|
| Lowering AST → typed HIR (subset F2) | Names, types, places, explicit borrows |
| Type check + ownership/borrow check | Algoritmo lexical v0 = ADR-009 |
| Diagnostics **E0201** / **E0202** (y E0203–E0206 per scope) | Texto EN canonical (ADR-006) |
| API consumable by `arita-cli` / measure | Before emit-Rust |

**No-meta v0:** full NLL, lifetimes on surface, self-referential, user-defined generics, “trust rustc”.

### 2. Module boundary (workspace)

Extension of ADR-003:

```text
crates/arita-syntax   — parse / AST
crates/arita-hir      — HIR + type/ownership check   ← NUEVO (este ADR)
crates/arita-codegen  — emit-Rust (consume HIR o AST bridged; pin en IMPL GO)
crates/arita-cli      — orquesta pipeline
```

Dependencias previstas:

- `arita-hir` depende de `arita-syntax` (AST in).
- `arita-codegen` **eventualmente** depende de `arita-hir` (no de AST crudo for paths ownership-sensitive).
- Until the checker truly exists, the production pipeline **may** follow AST→codegen (as hoy); the HIR crate **no** se vende as “ownership on”.

### 3. Anti-theater — pins obligatorios

| Forbidden | Why |
|-----------|---------|
| `check()` / `lower()` que **always** `Ok(())` y marcar ownership PASS | Fake checker / theater |
| Cablear HIR en measure as oracle de ownership **without** casos neg E0201/E0202 | skip / absence ≠ PASS |
| Decir “borrow done” because the crate exists or compiles | Theater |
| Dejar que only `rustc` rechace moves y llamarlo accepted ARITA | ADR-009 |

**Golden rule:** no hay `accepted` de ownership ARITA until there exist oracles **negativos** stable (parse/check/measure) for **E0201** y **E0202**, plus positive controls.

### 4. Fases de IMPL (after GO DOC; cada una with CUT/GO propio si hace falta)

1. **DOC GO** (this ADR) — **hecho** (`HIR-SCAFFOLD-DOC-20260913`).
2. **Scaffold types-only** — **hecho** (`HIR-TYPES-20260913` verified): tipos/API + `lower_ast`; **without** claim de check.
3. **Checker v0** — **hecho** (`HIR-CHECK-V0-20260913` ACCEPTED): E0201/E0202.
4. **Oracles** E0201/E0202 en `ejemplos/f2/neg/`; measure NEG follow-up si falta.

**Architect preference (aligned with the request):** **DOC only first** — do not open IMPL “pass-through always Ok” not even as a public placeholder.

### 5. Out of this ADR

- Emitir E021x (ya ADR-010).
- Logic island F3.
- Cambiar surface F2 / AST without CUT.
- Autorizar crates de checker **before** de GO of this ADR.

## Checklist GO (Ingeniero) — cerrado

- [x] Metas HIR v0 OK
- [x] Module boundary / deps OK
- [x] Pins anti-theater (no fake pass-through; no PASS without E0201/E0202) OK
- [x] DOC only first OK
- [x] GO `HIR-SCAFFOLD-DOC-20260913` → **accepted** (DOC-only)
- [x] Explicit: DOC CUT without checker; types-only (`HIR-TYPES`) **verified** after — remains **NOT** checker

## Estado IMPL (post-DOC GO)

- CUT `HIR-TYPES-20260913` **verified** (Ingeniero Rust, 2026-09-13): mechanical mirror AST F2 + `lower_ast` thin.
- **NOT** checker — without `check()` / Ok theater / E020x.
- Arquitecto / Codegen: **no emit changes** until a later CUT.
- **Checker** ownership: CUT `HIR-CHECK-V0-20260913` **ACCEPTED** (E0201/E0202; ADR-009; Documents verify).

## Enlaces

- `DOC/ADR/009-f2-borrow-checker-notes.md`
- `DOC/ADR/006-f2-ownership-surface.md`
- `DOC/ADR/003-f1-workspace-surface.md`
- `DOC/02-ARCHITECTURE.md`
- `ROADMAP.md` — Phase 2 (move/borrow / HIR)
