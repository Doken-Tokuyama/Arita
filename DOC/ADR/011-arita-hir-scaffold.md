# ADR-011 — Scaffold crate `arita-hir` (ownership futuro)

- **Estado:** **aceptada** (DOC-only)
- **CUT-ID:** `HIR-SCAFFOLD-DOC-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO DOC)
- **DOC CUT:** `HIR-SCAFFOLD-DOC-20260913` aceptada (diseño).
- **TYPES CUT:** `HIR-TYPES-20260913` **verified** — crate `arita-hir` existe (mirror AST + `lower_ast`); **NOT** checker; **no emit**
- **Relacionados:** ADR-003 (workspace), ADR-006 (ownership rules), ADR-007 (AST F2), ADR-009 (borrow notes DOC-only), ADR-001 (emit-Rust), `THREAT_MODEL.md`
- **Gobernanza:** **DOC only first**. **No** fake checker. **No** PASS de ownership hasta oráculos **E0201/E0202**.

## Contexto

ADR-009 documenta el contrato move/borrow pero **prohíbe** reivindicar ownership real sin HIR. El workspace ahora incluye `arita-hir` (types-only). Pipeline de producción sigue AST→codegen hasta un CUT de emit/HIR.

Este ADR introduce el **crate** `arita-hir` como frontera de módulo para el checker futuro — primero en DOC; IMPL de scaffold **solo** tras GO, y **sin** “check vacío que siempre pasa” disfrazado de ownership.

## Decisión propuesta

### 1. Metas del HIR (v0 path)

| Meta | Notas |
|------|--------|
| Lowering AST → HIR tipado (subset F2) | Nombres, tipos, places, borrows explícitos |
| Type check + ownership/borrow check | Algoritmo lexical v0 = ADR-009 |
| Diagnósticos **E0201** / **E0202** (y E0203–E0206 según scope) | Texto EN canónico (ADR-006) |
| API consumible por `arita-cli` / measure | Antes de emit-Rust |

**No-meta v0:** NLL completo, lifetimes en surface, self-referential, generics user-defined, “trust rustc”.

### 2. Frontera de módulo (workspace)

Extensión de ADR-003:

```text
crates/arita-syntax   — parse / AST
crates/arita-hir      — HIR + type/ownership check   ← NUEVO (este ADR)
crates/arita-codegen  — emit-Rust (consume HIR o AST bridged; pin en IMPL GO)
crates/arita-cli      — orquesta pipeline
```

Dependencias previstas:

- `arita-hir` depende de `arita-syntax` (AST in).
- `arita-codegen` **eventualmente** depende de `arita-hir` (no de AST crudo para paths ownership-sensitive).
- Hasta que el checker exista de verdad, el pipeline de producción **puede** seguir AST→codegen (como hoy); el crate HIR **no** se vende como “ownership on”.

### 3. Anti-theater — pins obligatorios

| Prohibido | Por qué |
|-----------|---------|
| `check()` / `lower()` que **siempre** `Ok(())` y marcar ownership PASS | Fake checker / theater |
| Cablear HIR en measure como oráculo de ownership **sin** casos neg E0201/E0202 | skip / ausencia ≠ PASS |
| Decir “borrow done” porque el crate existe o compila | Theater |
| Dejar que solo `rustc` rechace moves y llamarlo accepted ARITA | ADR-009 |

**Regla de oro:** no hay `accepted` de ownership ARITA hasta que existan oráculos **negativos** estables (parse/check/measure) para **E0201** y **E0202**, más positivos de control.

### 4. Fases de IMPL (tras GO DOC; cada una con CUT/GO propio si hace falta)

1. **DOC GO** (este ADR) — **hecho** (`HIR-SCAFFOLD-DOC-20260913`).
2. **Scaffold types-only** — **hecho** (`HIR-TYPES-20260913` verified): tipos/API + `lower_ast`; **sin** claim de check.
3. **Checker v0** — **hecho** (`HIR-CHECK-V0-20260913` ACCEPTED): E0201/E0202.
4. **Oráculos** E0201/E0202 en `ejemplos/f2/neg/`; measure NEG follow-up si falta.

**Preferencia Arquitecto (alineada al pedido):** **DOC only first** — no abrir IMPL “pass-through always Ok” ni siquiera como placeholder público.

### 5. Fuera de este ADR

- Emitir E021x (ya ADR-010).
- Isla lógica F3.
- Cambiar surface F2 / AST sin CUT.
- Autorizar crates de checker **antes** de GO de este ADR.

## Checklist GO (Ingeniero) — cerrado

- [x] Metas HIR v0 OK
- [x] Frontera de módulo / deps OK
- [x] Pins anti-theater (no fake pass-through; no PASS sin E0201/E0202) OK
- [x] DOC only first OK
- [x] GO `HIR-SCAFFOLD-DOC-20260913` → **aceptada** (DOC-only)
- [x] Explícito: DOC CUT sin checker; types-only (`HIR-TYPES`) **verified** después — sigue **NOT** checker

## Estado IMPL (post-DOC GO)

- CUT `HIR-TYPES-20260913` **verified** (Ingeniero Rust, 2026-09-13): mirror mecánico AST F2 + `lower_ast` thin.
- **NOT** checker — sin `check()` / Ok theater / E020x.
- Arquitecto / Codegen: **no emit changes** until a later CUT.
- **Checker** ownership: CUT `HIR-CHECK-V0-20260913` **ACCEPTED** (E0201/E0202; ADR-009; Documents verify).

## Enlaces

- `DOC/ADR/009-f2-borrow-checker-notes.md`
- `DOC/ADR/006-f2-ownership-surface.md`
- `DOC/ADR/003-f1-workspace-surface.md`
- `DOC/02-ARCHITECTURE.md`
- `ROADMAP.md` — Fase 2 (move/borrow / HIR)
