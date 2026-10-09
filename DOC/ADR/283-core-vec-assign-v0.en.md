Translation of `283-core-vec-assign-v0.md`; the original is normative. / Traducción de `283-core-vec-assign-v0.md`; el original es el normativo.

# ADR-283 — Core 0.9 slice 2: VEC-ASSIGN

- **Estado:** **CLOSED** Lex **816/816** (Ingeniero 2026-09-27 · REMEASURE3 exclusivo · Veyra ACCEPTED `20260927T013200Z`) → `GATE-CORE09-VEC-ASSIGN-20260926.md` (not in the public export / no incluido en el export público) · antes: GO IMPL 2026-09-26 tras CLOSED ADR-282
- **CUT-ID:** `CORE-0.9-VEC-ASSIGN-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero (pins OK 2026-09-26 19:54 · R1/R2)
- **Padre:** [ADR-281](281-core-0.9-pins.md) §0.1 slice 2 · contrato [ADR-270](270-lang-contract-set.md)
- **Prev:** slice 1 MAP-ASSIGN (ADR-282) — prereq CLOSED al GO IMPL; este CUT **no** reabre 265/270 (set) ni 277/278 (fn→Result / `?`)
- **Prereq surface:** `v.set(i, x) -> Result<(), Int>` + `Err(0)` OOB + **E0319** + helper `__arita_vec_set` (ADR-265/270) **ya IN** · `fn … -> Result<T,E>` (ADR-277) + `?` desugar (ADR-278) **ya IN**
- **HOLD:** receptor loan `&mut` / params `&mut Vec` (ADR-281 D1) · Mutex · threads · idle · TLS/WS · crates.io · repair · I/O-new · String.set · IndexMut residual — receptor no-Map (String, otros tipos) o sin binding → E0314 (Vec sale a E0344 en este slice) · `?` on Option · `?` in Io main · async Result · surface unwrap/expect (241) · reopen E0272/E0291/E0340/E0341/E0342/E0343

## Goal

Unpark **`v[i] = x`** on `Vec`/`List` with **zero** panic path in the emit. The sugar is an implicit `?` over ADR-265’s fallible set: `v[i] = x` ≡ `v.set(i, x)?`. It is therefore legal **only** as a statement inside the body of `fn … -> Result<_, Int>` (ADR-277), where OOB propagates as `Err(0)` visible in the signature. Outside that context → **E0344** (new) with canonical message `index assign outside result fn` + statement span `@a..b`, **no hint** (fix-hint → Backlog; does not block CLOSED). Negative lit stays **E0319**. No new API.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Desugar** | stmt `v[i] = x` where `v: Vec<T>`/`List<T>` ⇒ `v.set(i, x)?` (265 + 278) — same `IndexAssign` node from ADR-282 extended to Vec/List receivers; exact diag = equivalent `v.set` (except context E0344) |
| **Scope** | **only** the body of a synchronous `fn … -> Result<U, E>` (including nested `while`/`if`/`match` inside that body) |
| **OK** | `0 ≤ i < len` → replace at `i`; `len` unchanged; execution continues |
| **Runtime Err** | `i ≥ len` or non-lit `i < 0` → early-return `Err(0)` (ADR-270 OOB code); `v` **intact** |
| **Error type** | that of set: **`Int`**, value `0` (ADR-265/270) — **no** new type is invented |
| **Compat E** | ADR-278 (`?`) rule: the fn’s `E` must **unify** with `Int` (same E; **no** new From / coerce). `E ≠ Int` → **E0203** `type mismatch` (HIR emits it for the sugar; ADR-281 D4 · Engineer review 2026-09-26 §2). General `?` is not touched (ADR-278 closed; `g()?` gap → backlog) |
| **Out of scope** | `fn main() -> Io<()>` · any fn→Io · non-Result fn · `async fn` (async Result OUT) · `test` blocks · `scenario`/`acceptance` outside a helper fn→Result → **E0344** `index assign outside result fn` |
| **Negative lit** | `v[-1] = x` (and any `int_lit < 0`) → **E0319** `negative set index` (applied to set sugar; ADR-281 D3) |
| **Precedence** | **E0344 > E0319 > E0203** for the sugar: context (**E0344**) > negative lit (**E0319**) > type (**E0203**) — same criterion as E0343 “beats type noise” (ADR-278). `v[-1] = "x"` → **E0319**; `v.set(-1, "x")` still yields **E0203** (ADR-265/270, **not** reopened). Divergence accepted only in multi-error programs and documented; Measure pins both exact codes with the pair (`v[-1] = "x"` E0319 · `v.set(-1, "x")` E0203). **Between statements** (one diag per fn; multi-error → backlog): **first offender in source order wins**. In a non-Result fn, `v[i] = x` before `g()?` → **E0344**; `g()?` before `v[i] = x` → **E0343** (ADR-278, not retagged). Precedence E0344 > E0319 > E0203 applies only within the same `v[i] = x` statement (Engineer sign-off 23:58). |
| **Receiver / borrow** | owned `let mut` binding **IN** · loan `&mut` and params `&mut Vec` → **HOLD** (ADR-281 D1) · non-mut / shared loan / live loan → **E0202** (same diag as non-mut `v.set`, ADR-049 §1b) · moved → **E0201** |
| **Evaluation** | order ≡ `v.set(i, x)`: `i`, then `x`, then bounds-check; an RHS `?` (278) that fails returns **before** touching `v` |
| **R1 — no `as usize`** (Engineer 19:58) | (a) lowering of `v[i] = x` does **not** emit `as usize` casts; conversion via `usize::try_from(i)` **or** by delegating to `__arita_vec_set` · (b) **helpers**: `__arita_vec_set` (ADR-265) **and** `__arita_vec_insert` (ADR-260) in `crates/arita-codegen/src/lib.rs` (`ARITA_VEC_INSERT_HELPER` / `ARITA_VEC_SET_HELPER`, ~L152–175; today `i as usize` after `i < 0` guard) are rewritten with `usize::try_from(i)` → failure = `Err(0)`. Semantics **identical** (OK/`Err(0)`; `set` does not grow len; `insert` shifts) — does **not** reopen 260/265 |
| **Parser peek → HIR** (Engineer 20:23) | the parser peek `has_non_map_index_assign` (`crates/arita-syntax/src/lib.rs`) used by ADR-282 is a **temporary bridge**; in this slice (E0344) **all** Map/Vec (and fn→Result context) decisions move to HIR (`HirStmt::IndexAssign`) and the peek is **removed** |
| **R2 — temporaries before the borrow** | index and value (and RHS temporaries) are evaluated **before** taking the Vec’s `&mut` (ARITA checker + Rust emit): `v[0] = v.len()` and self-referential forms compile **without** E0202 or a rustc borrow error |
| **Emit** | **never** IndexMut nor Rust `v[i] = x` · **never** unwrap/expect/panic (241) · **never** discard the set’s result (`let _ =`, `.ok()`, `unwrap_or`) |
| **OUT** | compound `v[i] += x` (`-=`, `*=`…) · nested `v[i][j] = x` · field-on-index `v[i].f = x` → **E0006** `construct outside F1.1 (parse failure)` (decided; no new grammar in 0.9) · `s[i] = x` String · other types → **E0314** (ADR-281 §0.2) · index-assign as an expression |
| **Does not reopen** | E0319 text/code (265/270) · E0342 (277) · E0343 (278) · E0272 · E0340 · E0341 · E0291 |

## 1. Surface example

```text
// POS — v[i] = x only in fn → Result<_, Int>; OOB → Err(0) propagated
fn bump(i: Int, x: Int) -> Result<Int, Int> {
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  v.push(2)
  v[i] = x          // ≡ v.set(i, x)?   (i ≥ len → return Err(0))
  let n: Int = v.len()
  Ok(n)
}

fn main() -> Io<()> {
  // PIN: no v[i] = nor ? here — match-convert
  match bump(1, 9) {
    Ok(n) => print(n),
    Err(_) => print("oob")
  }
  match bump(99, 0) {
    Ok(n) => print(n),
    Err(_) => print("oob")     // Err(0) propagated, no panic
  }
}

// NEG → E0344 (outside fn→Result)
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  v[0] = 2          // E0344 index assign outside result fn @a..b (no hint)
}

// NEG → E0319 (lit negativo)          fn f() -> Result<Int, Int> { …; v[-1] = 0; Ok(0) }
// NEG → E0203 (E incompatible, D4)    fn f() -> Result<Int, Text> { …; v[0] = 1; Ok(0) }
// NEG → E0202 (binding no-mut, = set)  fn f() -> Result<Int, Int> { let v: Vec<Int> = Vec::new(); v[0] = 1; Ok(0) }
// NEG → E0006 (no grammar)           v[0] += 1  ·  v[0][1] = 2  ·  v[0].f = 1
// NEG → E0314 (residual)               s[0] = "x"
```

### 1.1 Reference emit (Codegen chooses A or B; both without panic)

```text
// A — get_mut + usize::try_from (R1: no `as usize`; R2: temporaries before the &mut)
{ let __i: i64 = <i>; let __x = <x>;
  let __slot = match usize::try_from(__i) { Ok(__u) => <v>.get_mut(__u), Err(_) => None };
  match __slot {
    Some(__s) => *__s = __x,
    None => return Err(0),
  } }

// B — helper ADR-265 + early-return (≡ desugar `?`); R2: temporaries before the &mut
{ let __i: i64 = <i>; let __x = <x>; __arita_vec_set(&mut <v>, __i, __x)?; }
```

Both: zero `IndexMut`, zero `<v>[..] =`, zero unwrap/expect/panic, zero `as usize` (R1) in the lowering **and** in the helpers; `Err(0)` identical to `set`’s.

### 1.2 Helpers without `as usize` (R1b — reference form; Codegen PHASE 1 deviations accepted)

```text
fn __arita_vec_set<T>(v: &mut [T], i: i64, x: T) -> Result<(), i64> {
    use std::convert::TryFrom;
    let Ok(u) = usize::try_from(i) else { return Err(0) };
    match v.get_mut(u) { Some(slot) => { *slot = x; Ok(()) } None => Err(0) }
}

fn __arita_vec_insert<T>(v: &mut Vec<T>, i: i64, x: T) -> Result<(), i64> {
    use std::convert::TryFrom;
    let Ok(u) = usize::try_from(i) else { return Err(0) };
    if u > v.len() { return Err(0); }
    v.insert(u, x);
    Ok(())
}
```

Acceptable equivalent if: zero `as usize`, zero unwrap/expect/panic, `i < 0` or out of range → `Err(0)`. Other `as usize` in the emit (swap/remove/resize/reserve/…) stay **outside** this slice (not touched or scanned here).

**Codegen PHASE 1 deviations (Engineer review 2026-09-26, accepted):**

- `__arita_vec_set(&mut [T])` instead of `&mut Vec<T>` (clippy `ptr_arg`). Correct: set does not change length. Emitted calls and behavior unchanged; covered by helpers-semantics-unchanged. `__arita_vec_insert` must keep `&mut Vec<T>` because it does change length.
- `use std::convert::TryFrom;` inside each helper due to `arita build`’s 2015 edition. OK. Emit-clippy must stay clean on 2015 and 2021 (no `unused_imports` on 2021).

## 2. Oracles

Required (to be measured at GO IMPL; **none** measured in this DOC):

| Id | Expect |
|----|--------|
| `core09-vec-assign-ok` | in-bounds `v[i] = x` in fn→Result → Ok + value read via `get`/`[]` → expected stdout |
| `core09-vec-assign-oob-err` | OOB (`i ≥ len`) → propagated `Err(0)` → deterministic caller/main match fail (no panic); `v` intact |
| `core09-vec-assign-neg-nonlit` | non-lit `i < 0` → propagated `Err(0)` (no panic) |
| `core09-vec-assign-eq-set` | golden: `v[i] = x` ≡ `v.set(i, x)?` (same Ok/Err stdout) |
| `core09-vec-assign-self-ref` | **POSITIVE**: `v[0] = v.len()` (and temporaries that read `v`) in fn→Result → compiles and runs without E0202 or rustc error (R2) |
| `core09-vec-assign-qmark-chain` | `v[i] = x` + `?` (278) in the same fn → first Err cuts the chain |
| `neg-core09-vec-assign-outside-main` | `v[i] = x` in `fn main() -> Io<()>` → **E0344** |
| `neg-core09-vec-assign-outside-fn` | `v[i] = x` in a non-Result fn → **E0344** (surface fixture); `test` → **E0344** only via direct HIR test (`test {}` rejects `let`; see Backlog) |
| `neg-core09-vec-assign-qmark-chain` | `v[i] = x` followed by `let n = g()?` (g: fn→`Result<_, Int>`) in the same **non-Result** fn, with `v[i] = x` first in source order → **E0344** (pin 23:50, Engineer sign-off 23:58: first offender in source order wins; inverse pair in `neg-core09-vec-assign-qmark-chain-rev`). `g()?` with `E ≠ Int` stays in Backlog (rustc E0277) and has **no** neg |
| `neg-core09-vec-assign-qmark-chain-rev` | `let n = g()?` (g: fn→`Result<_, Int>`) **before** `v[i] = x` in the same **non-Result** fn → exact **E0343** (existing ADR-278 code, not retagged; pair of `-qmark-chain`, source-order rule; Engineer sign-off 23:58) |
| `neg-core09-vec-assign-neg-lit` | `v[-1] = x` in fn→Result → **E0319** |
| `neg-core09-vec-assign-neg-lit-vs-set` | Measure pair (Engineer review 2026-09-26 item 1): `v[-1] = "x"` → **E0319** · `v.set(-1, "x")` → **E0203** (ADR-265/270, **not** reopened) — both exact codes pinned so neither can change unnoticed; divergence accepted only in multi-error programs (§0 Precedence) |
| `neg-core09-vec-assign-err-type` | fn→Result with `E ≠ Int` → **E0203** (HIR; D4 · Engineer review 2026-09-26 §2) |
| `neg-core09-vec-assign-non-mut` | non-mut binding / shared loan → **E0202** (= `v.set`) |
| `neg-core09-vec-assign-compound` | `v[i] += x` → exact **E0006** (green build = Rejected); nested `v[i][j] = x` / field `v[i].f = x` = same **E0006** rule |
| `neg-core09-vec-assign-unwrap` | bind `let r: Result<(), Int> = v.set(i, x)` and `r.unwrap()` / `r.expect(…)` (same code as existing invent-unwrap negs) → exact **E0206** `method not in F2 std whitelist` (new pin 23:50; the oracle requires only E0206 and does **not** accept E0342, same as `neg-core09-map-assign-unwrap`; chained form `v.set(i, x).unwrap()` is not used in the fixture); ≠ E0291 reopen |
| `neg-core09-vec-assign-bad-return` | fn→Result that mutates and returns a non-Result payload (`v[0] = 1; 0`) → **E0342** (not retagged) |
| `neg-core09-vec-assign-err-swallow` | caller `match f() { …, Err(_) => <success lit> }` → exact **E0272** `result error swallowed` (new pin 23:50; ADR-048 · same code as `neg-e0272-err-default-lit`). E0223 applies only if the pattern does not match the scrutinee type, and the fixture does not — **no** H4 reopen |
| `core09-vec-assign-emit-ban` | emit grep: zero Rust `\w+\[[^\]]+\]\s*=[^=]` (IndexMut-style), zero `IndexMut`/`std::ops::Index`, zero `.unwrap()`/`.expect(`/`panic!`, zero `let _ = __arita_vec_set` / `.ok()` over set; presence of `get_mut(`+`usize::try_from` or `__arita_vec_set(…)?`; build without warnings |
| `core09-vec-assign-emit-no-as-usize` | emit grep (R1): zero `as usize` in the lowering of `v[i] = x` **and** in the bodies of `__arita_vec_set` and `__arita_vec_insert`; presence of `usize::try_from` in both helpers (covers programs that use `set`, `insert` and `v[i] = x`) |
| `core09-vec-helpers-semantics-unchanged` | regression: existing `set` (265) and `insert` (260) oracles keep the same expectation (OK / `Err(0)` / E0319 / E0311) after rewriting helpers |

**Discard (ADR-281 D2 → must-use backlog):** discarding the `Result` of a fn→Result that uses `v[i] = x` (`let _ = f()`) or of bare `v.set` has **no** diag in 0.9. **No E0345**; E0340 (host IO only) is **not** retagged; no oracle in this slice.

**Migration (option A, ADR-281 §0.3):** same ids and files; expect E0314 → **E0344**: `neg-core05-index-mut` · `neg-core05-scen-coll-index-mut` · `neg-core05-ref-coll-index-mut` · `neg-core06-index-mut-assign` · `neg-core06-scen-gp-index-mut` · `neg-core06-ref-gp-index-mut` · `neg-e0310-index-vec`. `neg-e0310-index-string` unchanged (E0314). Edited in this slice’s IMPL; does **not** reopen the close of 261/262/263/265/267/268.

### 2.1 `<arita:deferred-shape>` marker (Engineer review 2026-09-26 item 3)

The design is accepted (HIR decides E0344/E0314 or returns the deferred E0001/E0006), with three CLOSED conditions:

- `arita parse` **must fail on its own** if the resulting AST contains a marker: it reports the parser’s deferred code (E0001/E0006), which was its prior behavior. No invalid program may leave `parse` with exit 0.
- The marker must never reach the emitted Rust: emit-ban oracle for `arita:deferred-shape` on the positives, and codegen rejects with an error (no panic) if it sees it.
- A test showing it cannot be written from the surface (it is not a valid identifier and yields a parse error).

## 3. E0344 pin

**E0344** `index assign outside result fn` — statement `v[i] = x` on `Vec`/`List` outside the body of a synchronous function whose return type is `Result<_,_>` (e.g. `fn main() -> Io<()>`, non-Result fn, `async fn`, `test`).

- **Emission:** canonical message `index assign outside result fn` + statement span `@a..b`, **no hint**. Missing hint **does not block CLOSED**; the fix-hint moves to Backlog (Engineer 2026-09-26 22:20, backlog accepted).
- **Owner:** this ADR (283).
- **No** reassign E0343 / E0342 / E0319 / E0314 / E0272 / E0340 / E0341 / E0291.
- Note: E0344 today appears only in a negative HIR test assertion (Option? → E0203, `!starts_with("E0344")`); no prior meaning.

## 4. What NOT to touch

| Forbidden | Reason |
|-----------|--------|
| `?` in Io main · Option? · async Result | OUT (278) |
| Residual IndexMut (non-Map receiver (String, other types) or no binding → E0314; Vec → E0344 here) · String.set | HOLD (E0314) |
| Raw `IndexMut` / Rust `v[i] = x` in emit | panic path — forbidden |
| New OOB error type / From magic | `Err(0): Int` is SoT (270) |
| Mutex / threads / idle / TLS/WS / crates.io / repair / I/O-new | Global HOLD |
| Surface unwrap/expect | emit-ban 241 HOLD |
| Receiver loan `&mut` · params `&mut Vec` · widen `fn_param` | HOLD (ADR-281 D1) |
| `as usize` casts in the lowering and in `__arita_vec_set` / `__arita_vec_insert` | R1 |
| GO IMPL before CLOSED 282 + Engineer OK | Orchestrator gate |
| Invent PASS / Lex N/N | DOC only |

## 5. CLOSED criterion

Green Lex §2 (N/N set by the Engineer at GO IMPL); ADR-281 slice 2 tick; §0.3 migration applied; §4 HOLDs intact; §2.1 marker conditions (item 3); no existing fixture changes code outside the declared migration (item 4, Measure verifies in the full run). Next: slice 3 SCENARIO-MUT ([ADR-284](284-core-scenario-mut-v0.en.md)).

## Backlog (outside Core 0.9)

- Preexisting ADR-278 gap: `g()?` with incompatible `E` passes the frontend and ends in rustc **E0277**. Outside Core 0.9; does **not** reopen ADR-278 (closed). The sugar `v[i] = x` with `E ≠ Int` does not depend on this: HIR emits **E0203** (§0 Compat E). (Engineer review 2026-09-26 §2)
- Multi-error edge case (main without print + another HIR error before `v[i]=x` yields E0001): accepted and goes to **backlog** (multi-error diagnostic order), on the condition that **no** existing fixture changes code outside the declared migration (7 Vec negs to E0344). Measure verifies in the full run. (Engineer review 2026-09-26 item 4)
- Fix-hint for **E0344** (previously pinned in §3; EN reference text: `` `v[i] = x` can fail (index out of bounds → Err(0)); here use `match v.set(i, x) { Ok(()) => …, Err(_) => … }`, or move the assignment into a `fn … -> Result<_, Int>` ``). Fits the parked pin of the diag JSON scout `rationale`/`fix`/`spec_ref`. (Engineer 2026-09-26 22:20, backlog accepted)
- `test {}` rejects `let`: E0344 inside `test` is covered only by direct HIR tests, not by surface fixtures. (Engineer 2026-09-26 22:20, backlog accepted)
- `fn f(v: Vec<Int>)` does not parse: collection params stay HOLD (ADR-281 D1). (Engineer 2026-09-26 22:20, backlog accepted)

## Checklist

- [x] Pins `v[i] = x` ≡ `v.set(i, x)?` + E0344 + E0319 + emit A/B + oracles (DOC GO-ready)
- [x] Engineer review 2026-09-26 19:54 — pins OK · option A (7 negs → E0344) · D1 HOLD · R1/R2 · no E0345
- [x] Engineer 20:23 — peek `has_non_map_index_assign` = 282 temporary bridge; in slice 2 Map/Vec decision → HIR and the peek is removed
- [x] Pre-gate cleanup 2026-09-26 — compound/nested/field = E0006 decided; E0314 only String · unbound (Vec → E0344 here) “superseded (→ non-Map receiver (Vec until 283, String, other types) or no binding → E0314, Architect rev. 2026-09-26)”
- [x] **Architect decision 2026-09-26** (aligned with HIR `HirStmt::IndexAssign`, no IMPL change): residual E0314 = non-Map receiver (Vec until 283, String, other types) or no binding → E0314; in this slice Vec moves to E0344
- [x] Engineer 19:58 — R1 covers helpers `__arita_vec_set` + `__arita_vec_insert` (`usize::try_from` → `Err(0)`) + oracle `core09-vec-assign-emit-no-as-usize` · compound E0006
- [x] Engineer review 2026-09-26 §2 (`DOC/reviews/INGENIERO-ADR283-DECISIONS-20260926.md`) — sugar `E ≠ Int` = **E0203** (HIR emits it; replaces “current `?` diag with incompatible E”); backlog: `g()?` with incompatible E → rustc E0277 (preexisting 278 gap, outside 0.9, 278 not reopened); precedence E0344 > E0319 > E0203 explicit; `v.set(-1, "x")` stays E0203 (265/270 not reopened)
- [x] Engineer review 2026-09-26 items 1/3/4 + Codegen PHASE 1 deviations (`DOC/reviews/INGENIERO-ADR283-DECISIONS-20260926.md`) — E0319/E0203 pair oracle (§2) · `<arita:deferred-shape>` marker conditions for CLOSED (§2.1) · multi-error backlog without fixture changes outside the 7 negs (Backlog) · §1.2: `__arita_vec_set(&mut [T])`, `__arita_vec_insert(&mut Vec<T>)`, `use std::convert::TryFrom;` per helper, emit-clippy clean on 2015 and 2021 · alignment with GO IMPL header (item 5): L10 HOLD, checklist and Close
- [x] Engineer 2026-09-26 22:20 (written approval, backlog accepted) — E0344 emitted with canonical message `index assign outside result fn` + span `@a..b`, **no hint** (§3, L14, §1 example); missing hint does not block CLOSED · Backlog: E0344 fix-hint (with parked diag JSON rationale/fix/spec_ref pin) · `test {}` rejects `let` (E0344 in tests only via HIR tests) · `fn f(v: Vec<Int>)` does not parse (HOLD ADR-281 D1) · outside-fn oracle aligned (Architect)
- [x] 2026-09-26 23:50 · **NEW PIN (Architect), Engineer sign-off 23:58:** unwrap → exact E0206 (no E0342) · err-swallow → exact E0272 (E0223 does not apply) · neg qmark-chain → E0344 by source order · `g()?` E≠Int stays backlog · E0344/E0203/E0202/E0342 unchanged
- [x] 2026-09-26 23:58 · “first offender in source order” rule (E0344/E0343) written in §0 · inverse oracle `neg-core09-vec-assign-qmark-chain-rev` → E0343 (existing code) · Engineer condition met
- [x] GO IMPL (Engineer 2026-09-26, after CLOSED ADR-282)
- [ ] IMPL + Lex CLOSED

## Close

- pending: measure N/N + Veyra + Engineer OK
