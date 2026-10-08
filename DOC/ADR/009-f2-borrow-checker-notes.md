# ADR-009 — Checker borrow / `&mut` (F2) + contrato DOC

- **Estado:** **aceptada** (DOC + IMPL HIR-CHECK-V0 **ACCEPTED** tras verify Documents)
- **CUT DOC:** aceptada as documentation (2026-09-13)
- **CUT IMPL:** `HIR-CHECK-V0-20260913` — **ACCEPTED** (Ingeniero, post Documents verify, 2026-09-13)
- **Historial:** DOC-only → HIR-TYPES verified → HIR-CHECK-V0 accepted
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO DOC)
- **Relacionados:** ADR-006 (rules v0), ADR-007 (`Borrow` en AST), ADR-001 (emit-Rust), `THREAT_MODEL.md`
- **Gobernanza:** **no** reivindicar ownership real sin HIR; **no** inventar PASS; skip ≠ PASS

## Propósito

Contrato esperado del checker move/borrow F2 (semántica + diagnósticos).  
`arita-hir` types-only ya existe; **HIR-CHECK-V0** implementa el algoritmo v0. **No** reivindicar ownership PASS hasta oráculos E0201/E0202 verdes.

## Pin anti-theater (obligatorio)

| Prohibido | Por qué |
|-----------|---------|
| Emitir Rust y “dejar que `rustc` sea el checker” **como** `accepted` de ownership ARITA | El oráculo de ownership debe vivir en ARITA (HIR); `rustc` es backend, no veredicto de surface |
| Tests que solo compilan sin forzar E0201/E0202 | Theater |
| Marcar “borrow done” sin ejemplos **negativos** E2E + measure | skip / ausencia ≠ PASS |
| Implementar half-checker en AST sin HIR y llamarlo ownership | Fake ownership |

AST `Borrow` (ADR-007) ≠ checker. Checker = `arita-hir` bajo CUT `HIR-CHECK-V0` (Parser).

## Modelo v0 (referencia ADR-006 § Ownership)

1. **Move:** `String`, `Vec<_>` — move por valor; uso posterior → **E0201** `use of moved value`.
2. **Copy:** `Int`, `Bool` — no invalidan origen.
3. **Shared:** `&x` — alias lectura; conflicto con `&mut` solapado → **E0202** `borrow conflict`.
4. **Exclusive:** `&mut x` — único; duración = scope léxico simplificado (último uso del préstamo); **sin** lifetimes en surface.
5. Binding: `let` vs `let mut`; `&mut` solo sobre binding `mut` (si no → **E0205** o **E0202**, pin en IMPL GO).
6. Sin `unsafe` / raw pointers / lifetimes explícitos en F2.

## Dónde vive el checker (arquitectura)

```text
.arita → pest/AST (ADR-007: Borrow, Path, Let, …)
              ↓
         HIR + type/ownership check   ← AQUÍ (`arita-hir`, HIR-CHECK-V0)
              ↓
         emit-Rust (ADR-001) → rustc   ← backend; no sustituye el veredicto ARITA
              ↓
         arita measure (oráculos + clippy-workspace)
```

- AST puede **representar** `&` / `&mut` (nodo `Borrow`).
- **Rechazar** move/borrow ilegal requiere **HIR** + tabla de bindings/préstamos.
- Measure F2 ownership: negativos E0201/E0202 en corpus `neg/`; cablear en measure = follow-up si falta.

## Algoritmo lexical v0 (contrato DOC — no código)

Por función / bloque:

1. Mantener mapa `binding → {ty, mut, state}` donde `state ∈ {live, moved}`.
2. En asignación / arg por valor de tipo move: marcar origen `moved`; lectura posterior → E0201.
3. En `&e` / `&mut e`: registrar préstamo `{kind: shared|exclusive, place, span}`.
4. Al crear préstamo: rechazar si place `moved`; si `exclusive`, rechazar overlapping shared/exclusive activo; si `shared`, rechazar exclusive activo → E0202.
5. Fin de scope / último uso: liberar préstamos del place (simplificación léxica; sin NLL completo).
6. Copy types: nunca pasan a `moved`.

**Fuera de v0:** NLL estilo Rust, reborrows sofisticados, slices, self-referential, generics user-defined.

## Pins E0201 / E0202 v0 (HIR-CHECK-V0)

| Code | English (canonical) | Trigger v0 |
|------|---------------------|------------|
| **E0201** | `use of moved value` | Tras move de `String` o `Vec<_>` (asignación / arg por valor), **uso** del place (Path / método / arg) |
| **E0202** | `borrow conflict` | (a) dos `&mut` solapados al mismo place; (b) `&` + `&mut` solapados; (c) préstamo sobre place ya `moved` |

**Copy:** `Int` / `Bool` — nunca E0201 por “move”.  
**Mut:** `&mut` solo sobre binding `mut`; si no → **E0205** o **E0202** (pin Parser: preferir **E0205** si el binding no es `mut`, **E0202** si conflicto de préstamos).

Algoritmo: § Algoritmo lexical v0 arriba.

## Oráculos E0201/E0202 — **verified** (`HIR-CHECK-V0-20260913`)

Corpus real bajo `ejemplos/f2/neg/` (ver `ejemplos/f2/neg/README.md`). `arita parse` debe **rejected** + código EN.

| File | Code | Esperado |
|------|------|----------|
| `ejemplos/f2/neg/e0201-use-after-move.arita` | **E0201** | `E0201: use of moved value` |
| `ejemplos/f2/neg/e0202-double-mut.arita` | **E0202** | `E0202: borrow conflict` |

```bash
cargo run -p arita-cli -- parse ejemplos/f2/neg/e0201-use-after-move.arita
cargo run -p arita-cli -- parse ejemplos/f2/neg/e0202-double-mut.arita
```

Ampliaciones futuras (`&`+`&mut`, move Vec dedicado, etc.) = nuevos oráculos E2E, no theater.

## Relación con String / Vec

- `String` / `Vec` E2E positivos ya verificados; ownership = **move** → E0201.
- Types-only / parse OK ≠ checker ownership.

## Checklist DOC GO — cerrado

- [x] Notas DOC-only + pins anti-theater OK
- [x] GO DOC Ingeniero → **aceptada como documentación** (final DOC)

## Checklist IMPL HIR-CHECK-V0 — cerrado

- [x] CUT `HIR-CHECK-V0-20260913` → Parser
- [x] Emit E0201/E0202 con texto EN canónico
- [x] Corpus `e0201-use-after-move` / `e0202-double-mut` rejected real
- [x] Documents verify + Ingeniero → **ACCEPTED**
- [ ] Measure: cablear E0201/E0202 en `NEG_ORACLES` si aún no (follow-up)

## Enlaces

- `DOC/ADR/006-f2-ownership-surface.md`
- `DOC/ADR/007-f2-ast-nodes.md`
- `DOC/ADR/011-arita-hir-scaffold.md`
- `DOC/THREAT_MODEL.md`
- `ROADMAP.md` — Fase 2
