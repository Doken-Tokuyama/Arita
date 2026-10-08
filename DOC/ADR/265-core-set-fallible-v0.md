# ADR-265 — Core 0.6 slice 1: SET-FALLIBLE

- **Estado:** **CLOSED** Lex **710/710** (2026-09-26) — gate [`DOC/GATE-CORE06-SET-FALLIBLE-20260926.md`](../GATE-CORE06-SET-FALLIBLE-20260926.md)
- **CUT-ID:** `CORE-0.6-SET-FALLIBLE-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero
- **Padre:** [ADR-264](264-core-0.6-pins.md) §0.1 slice 1 · contrato [ADR-270](270-lang-contract-set.md)
- **Prev:** Core **0.5 CLOSED** Lex **705/705** — este CUT **no** reabre 259–263
- **Unpark acotado:** método `set` (escritura índice) — **solo** semántica Result; **no** unpark IndexMut assign / Mutex
- **HOLD:** Mutex · `v[i]=` · idle · TLS/WS · crates.io · repair · String.set · Map numérico set

## Objetivo

Whitelist `Vec`/`List.set` fallible según ADR-270: `Result<(), Int>`, E0319 lit negativo, emit **sin** panic ni IndexMut.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Signature** | `fn set(self: mut Vec<T>, i: Int, x: T) -> Result<(), Int>` |
| **OK** | `0 ≤ i < len` → replace + `Ok(())` |
| **Err runtime** | `i ≥ len` o `i < 0` no-lit → `Err(0)` |
| **Compile** | lit `i < 0` → **E0319** `negative set index` |
| **Emit** | helper `__arita_vec_set` (o equiv.) — **nunca** `IndexMut` / `v[i] =` panic |
| **OUT** | `set` → `()` · `v[i]=` · String.set |

## 1. Surface ejemplo

```text
let mut v = Vec.new()
v.push(1)
v.push(2)
match v.set(1, 9) { Ok(()) => ..., Err(_) => ... }
match v.set(99, 0) { Ok(()) => ..., Err(0) => ... }  // OOB
// v.set(-1, 0) → E0319
// v[0] = 1 → IndexMut OUT (diag estable)
```

## 2. Oracles

| Id | Expect |
|----|--------|
| `core06-set-ok` | in-bounds set → Ok + valor leído vía get/`[]` |
| `core06-set-oob` | OOB → Err(0), no panic |
| `neg-core06-set-neg-lit` | lit i<0 → E0319 |
| `core06-set-emit-ban` | emit grep: cero IndexMut / panic assign |
| `neg-core06-index-mut-assign` | `v[i]=` → diag estable (E0314 u vigente) |


### Pin E0319 (set-neg lit)

**E0319** `negative set index` — *solo* `set` con índice literal negativo (`v.set(-1, x)`).
Neg oracle: `neg-core06-set-neg-lit` → **E0319**.

**E0315** no se toca: SoT = `missing field` ([ADR-233](233-core-record-v0.md) + HIR + `neg-e0315-missing-field`). Prohibido reasignar E0315 a set-neg.


## 3. Criterio CLOSED

Lex §2 verde; tick ADR-264 slice 1; HOLDs Mutex/IndexMut-assign intactos. Siguiente: slice 2 MAP-INDEX.

## Checklist

- [x] Pins set Result + E0319 + emit-ban + oracles (GO-listo)
- [x] GO IMPL Orquestador
- [x] IMPL + Lex **CLOSED** 710/710
- [x] Slice 2 GO (MAP-INDEX ADR-266)


## Cierre

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE06-SET-FALLIBLE-20260926.md`](../GATE-CORE06-SET-FALLIBLE-20260926.md)
- Lex measure **710/710** accepted
- Clippy workspace OK; Veyra companion REJECTED (RUSTSEC + rustfmt) **non-blocking**
- Siguiente: ADR-266 GO IMPL MAP-INDEX (`CORE-0.6-MAP-INDEX-20260926`)
- HOLDs Mutex/IndexMut-assign/idle/TLS/WS/crates.io/repair intactos · Core 0.6 still open
