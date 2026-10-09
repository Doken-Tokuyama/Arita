Translation of `009-f2-borrow-checker-notes.md`; the original is normative. / Traducción de `009-f2-borrow-checker-notes.md`; el original es el normativo.

# ADR-009 — Checker borrow / `&mut` (F2) + contrato DOC

- **Estado:** **aceptada** (DOC + IMPL HIR-CHECK-V0 **ACCEPTED** tras verify Documents)
- **CUT DOC:** aceptada as documentation (2026-09-13)
- **CUT IMPL:** `HIR-CHECK-V0-20260913` — **ACCEPTED** (Ingeniero, post Documents verify, 2026-09-13)
- **Historial:** DOC-only → HIR-TYPES verified → HIR-CHECK-V0 accepted
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO DOC)
- **Relacionados:** ADR-006 (rules v0), ADR-007 (`Borrow` en AST), ADR-001 (emit-Rust), `THREAT_MODEL.md`
- **Gobernanza:** **no** reivindicar ownership real sin HIR; **no** inventar PASS; skip ≠ PASS

## Purpose

Expected F2 move/borrow checker contract (semantics + diagnostics).  
`arita-hir` types-only already exists; **HIR-CHECK-V0** implementa el algoritmo v0. **No** claim ownership PASS until oracles E0201/E0202 verdes.

## Pin anti-theater (mandatory)

| Forbidden | Why |
|-----------|---------|
| Emitir Rust y “dejar que `rustc` sea el checker” **as** `accepted` de ownership ARITA | The ownership oracle must live en ARITA (HIR); `rustc` is backend, not the surface verdict |
| Tests que only compilan without forcing E0201/E0202 | Theater |
| Marcar “borrow done” without ejemplos **negativos** E2E + measure | skip / absence ≠ PASS |
| Implementar half-checker en AST without HIR y llamarlo ownership | Fake ownership |

AST `Borrow` (ADR-007) ≠ checker. Checker = `arita-hir` bajo CUT `HIR-CHECK-V0` (Parser).

## Modelo v0 (referencia ADR-006 § Ownership)

1. **Move:** `String`, `Vec<_>` — move por valor; uso later → **E0201** `use of moved value`.
2. **Copy:** `Int`, `Bool` — no invalidan origen.
3. **Shared:** `&x` — alias lectura; conflicto with `&mut` solapado → **E0202** `borrow conflict`.
4. **Exclusive:** `&mut x` — unique; duration = simplified lexical scope (last use of the borrow); **without** lifetimes on surface.
5. Binding: `let` vs `let mut`; `&mut` only sobre binding `mut` (si no → **E0205** o **E0202**, pin en IMPL GO).
6. No `unsafe` / raw pointers / explicit lifetimes in F2.

## Where the checker lives (architecture)

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
- **Reject** move/borrow illegal requires **HIR** + bindings/borrows table.
- Measure F2 ownership: negativos E0201/E0202 en corpus `neg/`; cablear en measure = follow-up si falta.

## Lexical algorithm v0 (DOC contract — no code)

Per function / block:

1. Keep mapa `binding → {ty, mut, state}` where `state ∈ {live, moved}`.
2. On assignment / pass-by-value arg of move type: mark origin `moved`; later read → E0201.
3. En `&e` / `&mut e`: register borrow `{kind: shared|exclusive, place, span}`.
4. When creating a borrow: reject if place `moved`; si `exclusive`, reject overlapping shared/exclusive active; si `shared`, rechazar exclusive active → E0202.
5. End of scope / last use: release borrows of the place (lexical simplification; without NLL full).
6. Copy types: never pasan a `moved`.

**Fuera de v0:** NLL estilo Rust, reborrows sofisticados, slices, self-referential, generics user-defined.

## Pins E0201 / E0202 v0 (HIR-CHECK-V0)

| Code | English (canonical) | Trigger v0 |
|------|---------------------|------------|
| **E0201** | `use of moved value` | After move of `String` or `Vec<_>` (assignment / pass-by-value arg), **use** of the place (Path / method / arg) |
| **E0202** | `borrow conflict` | (a) two overlapping `&mut` on the same place; (b) overlapping `&` + `&mut`; (c) borrow on a place already `moved` |

**Copy:** `Int` / `Bool` — never E0201 por “move”.  
**Mut:** `&mut` only on binding `mut`; si no → **E0205** o **E0202** (pin Parser: preferir **E0205** si el binding no es `mut`, **E0202** si conflicto de borrows).

Algoritmo: § Algoritmo lexical v0 arriba.

## Oracles E0201/E0202 — **verified** (`HIR-CHECK-V0-20260913`)

Corpus real bajo `ejemplos/f2/neg/` (ver `ejemplos/f2/neg/README.md`). `arita parse` must **rejected** + code EN.

| File | Code | Esperado |
|------|------|----------|
| `ejemplos/f2/neg/e0201-use-after-move.arita` | **E0201** | `E0201: use of moved value` |
| `ejemplos/f2/neg/e0202-double-mut.arita` | **E0202** | `E0202: borrow conflict` |

```bash
cargo run -p arita-cli -- parse ejemplos/f2/neg/e0201-use-after-move.arita
cargo run -p arita-cli -- parse ejemplos/f2/neg/e0202-double-mut.arita
```

Ampliaciones futuras (`&`+`&mut`, move Vec dedicated, etc.) = nuevos oracles E2E, no theater.

## Relation to String / Vec

- `String` / `Vec` E2E positivos ya verificados; ownership = **move** → E0201.
- Types-only / parse OK ≠ checker ownership.

## Checklist DOC GO — cerrado

- [x] Notas DOC-only + pins anti-theater OK
- [x] GO DOC Ingeniero → **accepted as documentation** (final DOC)

## Checklist IMPL HIR-CHECK-V0 — cerrado

- [x] CUT `HIR-CHECK-V0-20260913` → Parser
- [x] Emit E0201/E0202 with texto EN canonical
- [x] Corpus `e0201-use-after-move` / `e0202-double-mut` rejected real
- [x] Documents verify + Ingeniero → **ACCEPTED**
- [ ] Measure: cablear E0201/E0202 en `NEG_ORACLES` si still no (follow-up)

## Enlaces

- `DOC/ADR/006-f2-ownership-surface.md`
- `DOC/ADR/007-f2-ast-nodes.md`
- `DOC/ADR/011-arita-hir-scaffold.md`
- `DOC/THREAT_MODEL.md`
- `ROADMAP.md` — Phase 2
