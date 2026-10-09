Translation of `282-core-map-assign-v0.md`; the original is normative. / Traducción de `282-core-map-assign-v0.md`; el original es el normativo.

# ADR-282 — Core 0.9 slice 1: MAP-ASSIGN

- **Estado:** **CLOSED** Lex **792/792** (Ingeniero 2026-09-26 · `GATE-CORE09-MAP-ASSIGN-20260926.md` (not in the public export / no incluido en el export público) · Veyra Proof **ACCEPTED** `.veyra/evidence/20260926T193713Z/`) · CUT `CORE-0.9-MAP-ASSIGN-20260926` · pins OK Ingeniero (19:54 · 19:58 · A6) + Orquestador 19:59
- **CUT-ID:** `CORE-0.9-MAP-ASSIGN-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero (pins OK + GO IMPL 2026-09-26; decisiones 19:58 + corrección A6) · Orquestador (nodo `IndexAssign` / emit 19:59)
- **Padre:** [ADR-281](281-core-0.9-pins.md) §0.1 slice 1
- **Prev:** Core **0.8 CLOSED** (ADR-276/280) — este CUT **no** reabre 276–280 ni 264–268
- **Prereq surface:** `Map<Text|String, Int>` + `m.put(k, v)` (ADR-237; su emit **no** se toca) **ya IN** · `m[k]` → `Option` (ADR-266) **ya IN** · receptor exclusivo no-mut / loan → **E0202** (ADR-049 §1b) **ya IN**
- **HOLD:** Mutex · threads · idle · TLS/WS · crates.io · repair · I/O-new · String.set · IndexMut residual — receptor no-Map (Vec hasta 283, String, otros tipos) o sin binding → E0314 (ADR-281 §0.2) · `v[i] = x` (slice 2 ADR-283) · receptor loan `&mut` / params `&mut Map` (ADR-281 D1) · asignación compuesta / anidada / campo sobre índice (sin gramática; E0006) · relajar args de `put` (backlog) · surface unwrap/expect (241) · Option? · `?` in Io main · reopen E0272/E0291/E0340/E0341/E0342/E0343

## Goal

Unpark **`m[k] = v`** on `Map` via its **own `IndexAssign` node** whose semantics and checks are those of `m.put(k, v)` (ADR-237): **total** write (insert/overwrite), no possible panic. **Base rule (Engineer 19:58):** in **every** error case the sugar yields the **same exact diag** as the equivalent `put`. **Sole exception** (A6 correction): `put`’s literal/ident arg restriction is **grammatical** (E0006) and the sugar does **not** inherit it → RHS = any v0 expression and self-ref is a **positive**. Receiver: **only** an owned `let mut` binding (loans / params `&mut Map` **HOLD**, D1). No new API; no new code; **no new grammar** (compound / nested / field-on-index stay E0006).

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Node** | Own `IndexAssign` (Orchestrator 19:59) for the already-parsed form `ident[key] = rhs` when the receiver is `Map`; invokes `put`’s checks (mutability, loans, types). Non-Map receiver (Vec until 283, String, other types) or no binding → **E0314** from HIR with span `@a..b` (§0.2); the 9 current E0314 ids stay E0314 in this slice |
| **Equivalence with `put`** (Engineer 19:58/19:59) | (a) same semantics and same stdout on programs that **both** forms accept · (b) same **exact** diag (and same message text) for mutability (**E0202**), loans (**E0202**) and types (**E0203**) · (c) eq-put oracle requires `insert(` in the emit — **not** exact emit |
| **RHS** | **any** v0 `assign_rhs` expression (ident, literal, `m.len()`, arithmetic, `f(x)`, `f(x)?` in fn→Result…). Does **not** inherit `put`’s arg limit (`method_push_arg` literal/ident; outside → E0006, grammar limit). Relaxing `put` → **backlog** |
| **Semantics** | absent key → insert; present → overwrite; old value **discarded** (statement-level; not a `Result`, no discard theater) |
| **Total** | no `Result`, no OOB, no panic; legal in **any** context (main `Io<()>`, fn→Result, non-Result fn, `test`) |
| **v0 types** | `K` ∈ {`Text`, `String`}; `V` = `Int` (ADR-237). Wrong-typed key/value → **E0203** `type mismatch` (= `put`; **no** mint) · numeric-key Map stays HOLD |
| **Receiver** | owned `let mut m` binding **IN** · loan `borrow mut m` / `&mut m` and params `&mut Map` → **HOLD** (ADR-281 D1) · `fn_param` is **not** widened |
| **Borrow negs** (= `put`) | non-`mut` binding → **E0202** `borrow conflict` (ADR-049 §1b) · write via shared loan (`borrow m` / `&m`) → **E0202** · live loan over `m` → **E0202** · `m` moved → **E0201** `use of moved value` |
| **R2 — temporaries before the borrow** | key and value (and any RHS temporary) are **evaluated before** taking the map’s `&mut` borrow, in the ARITA checker and in the Rust emit: self-referential forms (`m["b"] = m.len()`, key/value that read `m`) compile **without** E0202 or rustc E0502 |
| **R3 — clippy-clean emit statement** | bare statement `m.insert(k, v);` preceded by R2 temporaries (Orchestrator 19:59); current Codegen form: `{ let __arita_mk = <k>; let __arita_mv = <v>; m.insert(__arita_mk, __arita_mv); }`; `cargo clippy -- -D warnings` green on the emit |
| **Emit ban** | **zero** Rust `m[k] = …`, **zero** `Index`/`IndexMut`, **zero** unwrap/expect/panic (241) |
| **Evaluation** | order: `k`, then `v` (R2: both into temporaries), then insert |
| **OUT** | compound `m[k] += v` (`-=`, `*=`…) · nested `m[k][j] = v` / `v[i][j] = x` · field-on-index `m[k].f = v` / `v[i].f = x` → **E0006** `construct outside F1.1 (parse failure)` (decided; no new grammar in 0.9; Engineer A3) · index-assign as an expression · numeric `Map.set` · changing `put` |
| **Does not reopen** | ADR-266 (read sugar) · ADR-237 (`put` and its emit) · ADR-049 · E0272/E0340/E0341/E0342/E0343 |

### 0.1 Note — sugar emit ≠ `put` emit (expected; `put` is not touched)

The sugar emits the statement `m.insert(k, v);` (with R2 temporaries), **not** ADR-237’s `put` form (`{ let _ = m.insert(k, v); }`). That is **expected** and does not break equivalence: `core09-map-assign-eq-put` compares **stdout + diag + presence of `insert(`**, never exact emit. The `put` path does **not** change (regression `core09-map-assign-put-emit-unchanged`).

### 0.2 E0314 from HIR — span `@a..b` + tests

With the `IndexAssign` node, the Map vs non-Map decision is taken in HIR after typing (`HirStmt::IndexAssign`, `crates/arita-hir/src/lib.rs`): non-Map receiver (Vec until 283, String, other types) or no binding → **E0314** with the pre-282 text and the **statement span** in the parser’s format: `E0314: IndexMut assign not allowed (v[i]= / m[k]= HOLD) @a..b`.

- Existing tests in `crates/arita-hir/src/lib.rs`: `adr282_e0314_carries_index_assign_stmt_span` (exact message with `@start..end` and `&src[start..end] == "v[0] = 1"`) · `adr282_vec_string_unbound_stay_e0314` (Vec, String and unbound target → E0314; “other types” follow the same HIR arm — no dedicated test seen under `crates/`, required in IMPL if Measure asks).
- Temporary parser bridge: `has_non_map_index_assign` (`crates/arita-syntax/src/lib.rs`) only preserves pre-282 precedence against form gates (E0001 / E0006 non-main) and returns the span for `@a..b`; HIR remains the authority. It is **removed** in slice 2 (ADR-283, Engineer 20:23).

## 1. Surface example

```text
// POS — insert / overwrite (cualquier contexto); RHS expr v0
fn main() -> Io<()> {
  let mut m: Map<Text, Int> = Map::new()
  m["a"] = 7                 // insert
  m["a"] = 9                 // overwrite (7 descartado)
  m["b"] = m.len()           // R2: self-referential — len evaluated before insert (positive)
  let c: Int = 2
  m["c"] = c + 1             // arithmetic on RHS (does not inherit put limit)
  match m["a"] { Some(x) => print(x), None => print(0) }   // 9
}

// NEG → E0202 (non-mut binding; same diag and text as m.put)
fn main() -> Io<()> {
  let m: Map<Text, Int> = Map::new()
  m["a"] = 1
  print(m.len())             // print mandatory in neg fixtures (no print → E0001 first)
}

// NEG → E0202 (loan compartida / loan viva)   let r = borrow m ; r["a"] = 1
// NEG → E0203 (tipo)                          m[1] = 2   ·   m["a"] = "x"
// NEG → E0006 (no grammar)                 m["a"] += 1   ·   m["a"][0] = 1   ·   m["a"].f = 1
// NEG → E0206 (invent unwrap)                 let o: Option<Int> = m["a"] ; o.unwrap()
```

## 2. Oracles

Required (to be measured in IMPL; **none** measured in this DOC). Every neg requires an **exact code** (never “any error”); neg fixtures carry `print`.

| Id | Expect |
|----|--------|
| `core09-map-assign-insert` | `m[k] = v` new key → `m[k]`/`get` = `Some(v)` → expected stdout |
| `core09-map-assign-overwrite` | second `m[k] = w` → `Some(w)`; `len` unchanged |
| `core09-map-assign-eq-put` | program accepted by both forms: `m[k] = v` ≡ `m.put(k, v)` → same stdout + same diag + `insert(` present in both emits (**not** exact emit, §0.1) |
| `core09-map-assign-fn-result` | `m[k] = v` inside `fn → Result` + `?` (278) → OK with unchanged semantics |
| `core09-map-assign-self-ref` | **POSITIVE** (compiles and runs): `m["b"] = m.len()` (and key/value that read `m`) → green build, no E0202 or rustc E0502; stdout fixed by Codegen/Measure (R2; no `put` twin: `put` yields grammatical E0006) |
| `neg-core09-map-assign-non-mut` | `let m` binding (non-mut) → exact **E0202**, same text as twin `m.put` |
| `neg-core09-map-assign-shared-loan` | write via `borrow m` / `&m` → exact **E0202** (= twin `put`; loans HOLD D1; measured, never Inconclusive) |
| `neg-core09-map-assign-live-loan` | `m[k] = v` with a live loan of `m` used afterward → exact **E0202** (= twin `put`) |
| `neg-core09-map-assign-type` | `Int` key / `Text` value → exact **E0203** (= twin `put`) |
| `neg-core09-map-assign-compound` | `m["a"] += 1` → exact **E0006** (`construct outside F1.1 (parse failure)`); green build = Rejected (nested / field-on-index = same E0006 rule) |
| `neg-core09-map-assign-unwrap` | bind to `Option` and `o.unwrap()` / `o.expect(…)` → exact **E0206** (`method not in F2 std whitelist`; = existing invent-unwrap negs); ≠ E0291 reopen. Chained form `m["a"].unwrap()` yields E0006 → the fixture does **not** chain |
| `core09-map-assign-emit-ban` | emit grep: `insert(` present; zero Rust `\w+\[[^\]]+\]\s*=[^=]`, zero `IndexMut`/`std::ops::Index`, zero `.unwrap()`/`.expect(`/`panic!` |
| `core09-map-assign-emit-clippy` | `cargo clippy -- -D warnings` green on the emit (R3; Engineer A1); clippy absent → Inconclusive (never PASS) |
| `core09-map-assign-put-emit-unchanged` | ADR-237 regression: `put` emit (`ejemplos/core01/07-map-put-get.arita`) identical to baseline — the `IndexAssign` node does not alter the `put` path |

`E0201` (moved) has no own oracle in this slice.

**Migration (option A, ADR-281 §0.3) — atomic change (fixture + runner + smoke + IMPL):** `neg-core06-map-index-mut` — **same id**, stays a neg; file `ejemplos/core06/map-index/neg/01-index-mut.arita` moves to a **non-`mut`** binding (`let m`), no `m.put` line, with `print` → expect exact **E0202**. It is **not** turned into a positive. The 9 E0314 ids (Vec/String/unknown-field) **stay E0314** in this slice.

## 3. Diags

No new code. Reuses current canonical text: **E0202** `borrow conflict` (non-mut · shared loan · live loan · via `&`) · **E0201** `use of moved value` · **E0203** `type mismatch` · **E0006** `construct outside F1.1 (parse failure)` (compound · nested · field-on-index) · **E0206** `method not in F2 std whitelist` · **E0314** (non-Map receiver (Vec until 283, String, other types) or no binding → E0314; §0.2). Prior versions with E0205 for non-mut/loans: “superseded (→ E0202, rev. 2026-09-26)”. **No E0345.** **Do not** reassign E0272 / E0340 / E0341 / E0342 / E0343.

## 4. What NOT to touch

| Forbidden | Reason |
|-----------|--------|
| `v[i] = x` Vec | slice 2 ADR-283 (HOLD until CLOSED 282) |
| Residual IndexMut (non-Map receiver (Vec until 283, String, other types) or no binding → E0314) · String.set | HOLD (E0314) |
| Compound / nested / field-on-index assignment grammar | no new grammar in 0.9 (E0006) |
| Receiver loan `&mut` · params `&mut Map` · widen `fn_param` | HOLD (ADR-281 D1) |
| Relax `put` args · change `put` emit | backlog / ADR-237 regression |
| Mutex / threads / idle / TLS/WS / crates.io / repair / I/O-new | Global HOLD |
| Surface unwrap/expect | emit-ban 241 HOLD |
| Invent PASS / Lex N/N | DOC only |

## 5. CLOSED criterion

Green Lex §2 (N/N set by the Engineer at the gate with the real count); green emit clippy (R3); `neg-core06-map-index-mut` migration applied (E0202, same id); 9 E0314 ids intact; ADR-281 slice 1 tick; §4 HOLDs intact. Next: slice 2 VEC-ASSIGN ([ADR-283](283-core-vec-assign-v0.en.md)) — GO IMPL after CLOSED 282.

## Checklist

- [x] Pins `m[k] = v` ≡ `put` + borrow negs + oracles (DOC GO-ready)
- [x] Engineer review 2026-09-26 19:54 — pins OK · option A · D1 HOLD · R2/R3
- [x] Engineer decisions 19:58 + A6 correction · Orchestrator 19:59 — `IndexAssign` node · RHS v0 expr · self-ref positive · exact diag = `put` (E0202/E0203) · compound E0006 · unwrap E0206 · emit `m.insert(k, v);` · relax `put` → backlog
- [x] Measure prep rev. (E0205 “superseded (→ E0202, rev. 2026-09-26)” · compound E0006 · emit ≠ `put` note)
- [x] Pre-gate cleanup 2026-09-26 (Measure 21:29 + Codegen): body without E0205 / without explicit-discard requirement / E0314 only Vec·String·unbound “superseded (→ non-Map receiver (Vec until 283, String, other types) or no binding → E0314, Architect rev. 2026-09-26)” · nested/field = E0006 decided · §0.2 E0314 HIR span + tests
- [x] **Architect decision 2026-09-26** (aligned with HIR `HirStmt::IndexAssign`, no IMPL change): residual E0314 = non-Map receiver (Vec until 283, String, other types) or no binding → E0314
- [x] GO IMPL (CUT `CORE-0.9-MAP-ASSIGN-20260926`)
- [ ] IMPL + Lex CLOSED

## Close

- **GO IMPL** activo · IMPL/Lex pendientes · nada medido en este DOC.
