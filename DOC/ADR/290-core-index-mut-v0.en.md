Translation of `290-core-index-mut-v0.md`; the original is normative. / Traducción de `290-core-index-mut-v0.md`; el original es el normativo.

# ADR-290 — Safe IndexMut: compound indexed assignment (core 0.10, slice A)

- **Estado:** **CLOSED (Ingeniero, GO CLOSED 2026-10-03; DOC/GATE-CORE10-INDEX-MUT-20261003.md)** · `arita measure` 862/862 accepted (N = 862, k = 6) · historia: v0.3 DRAFT PINS (Arquitecto 2026-10-03, sha `7ea657e65343aff8766b33378a98ef832f3cda55c495287a5d3b3db9f941900f`); v0.3 revisada y APROBADA Y CONGELADA por el Ingeniero el 2026-10-03 (línea «Sello», pins 1–3 aceptados; GO IMPL con k = 6, N = 862, orden Parser → HIR → Codegen, Codegen condicionado a P7); antes, v0.2 (sha `5524d902166c718fd109f44af6e27d5a0b5ec686021f449c450e47eac09fa847`) aprobada con cambios sobre v0.1 (sha `926483b4…`); freeze `DOC/reviews/MEASURE-ADR290-INDEX-MUT-FREEZE-20261003.sha` y addendum de freeze `DOC/reviews/MEASURE-ADR290-INDEX-MUT-FREEZE-20261003.addendum.sha`.
- **Sello:** **v0.3 APROBADO Y CONGELADO (Ingeniero, 2026-10-03; sha previo al sello `7ea657e65343aff8766b33378a98ef832f3cda55c495287a5d3b3db9f941900f`) · GO IMPL con k = 6, N = 862, orden Parser → HIR → Codegen.** El GO de Parser y HIR es inmediato; el paso de Codegen exige antes la medición ligera P7 (`i64::checked_add/sub/mul` como puntero a fn sin lint de rustc/clippy en 2015 y 2021; si avisa, alternativa B del PREP §3.3 con aviso previo). Condiciones de IMPL: (a) P10 (literal máximo de i64 y `known_int` sin plegar a E0217) se verifica ANTES de diseñar la fixture de overflow de IM-2; si no se puede escribir el límite, IM-2 usa `*=` con operandos grandes, sin cambiar semántica; (b) sin E0333 por operador; (c) el emisor de E0006 para operador no soportado es el Parser; HIR solo para compuesto sobre Map/String con el texto exacto actual; (d) ningún cambio a la expresión binaria `a + b` (ADR-045): se registra como B-290-2 (P2); (e) migración IM-6 solo con freeze nuevo + addendum + fixture nuevo, sin reescribir 05-compound ni freezes cerrados; (f) k y N se re-miden al CUT, sin hacer ADR nuevo si IMPL propone un oráculo extra: se reporta al Ingeniero antes.
- **CUT-ID:** `CORE-0.10-INDEX-MUT-20261002`
- **Fecha:** 2026-10-02
- **Autores:** ARITA Arquitecto (pins) · orden de cola decidido por el Ingeniero (287 → 288 → 289 → **este** → Mutex)
- **Padre / contexto:** [ADR-261](261-core-index-sugar-v0.md) · [ADR-266](266-core-map-index-v0.md) · [ADR-282](282-core-map-assign-v0.md) · [ADR-283](283-core-vec-assign-v0.md) · [ADR-284](284-core-scenario-mut-v0.md) · [ADR-285](285-core-ref-mut-v0.md) · [ADR-289](289-core-emit-unused-parens-v0.md) (criterio de emit limpio)
- **Cierra:** el caso compuesto `v[i] op= x` sobre `Vec<Int>`/`List<Int>` (slice A) (CLOSED 2026-10-03). Map compuesto, anidado y campo quedan fuera (OUT / Q1); B-290-1 sigue abierto.
- **Códigos:** **E0333 nuevo** (D2; solo tipo de elemento no soportado; un operador compuesto no soportado ⇒ E0006). E0334–E0339 libres, reservados a slices B/C de este ADR.
- **No reabre:** E0291 · E0340–E0343 · ADR-265 / 270 / 278 / 283.

## Contexto

- ADR-261/266: read `v[i]`/`m[k]` = `get` → `Option`; IndexMut stayed OUT.
- ADR-282 (`m[k] = v`, Map<Text|String,Int>, ≡ put), ADR-283 (`v[i] = x`, Vec/List, ≡ `v.set(i,x)?`, only in `fn -> Result<_, Int>`), ADR-284 (scenario-mut), ADR-285 (ref-mut): CLOSED. Codegen emits helpers (`__arita_vec_set(&mut v,i,x)?`, `m.insert(k,v)`), never Rust `IndexMut`/`Index`; emit-bans require 0 IndexMut/Index in emitted Rust.
- Today still E0006 (outside F1.1 grammar): compound `v[0] += 1` (ejemplos/core09/vec-assign/neg/05-compound.arita:7; map-assign/neg/04-compound.arita:8), nested `v[0][1] = 1` (vec-assign/neg/06-nested.arita), field `v[0].f = 1` (vec-assign/neg/07-field.arita; TRAPS T09-13). HIR test: crates/arita-hir/src/lib.rs ~L8682-8687.
- Verified fact: the surface has NO plain compound assignment (`x += 1`): assign_stmt in arita.pest L216 (`=` only); index_assign_stmt at L218; rule `assign_stmt = { ident ~ "=" ~ assign_rhs }`, HIR `Assign {name, value}`. Therefore `v[i] += x` inherits no base: this ADR adds the compound operator ONLY on indexed places.

## Decision (slice A)

D1. Grammar: `index_assign_stmt` also admits `op_assign` ∈ {`+=`, `-=`, `*=`} (same operand set of ident and literal/ident as index as ADR-283). Plain `x += 1` stays out (E0006) → B-290-1.

D2. v0 type scope: Vec<Int>/List<Int> only. Other element types ⇒ E0333 (new: “compound index-assign: unsupported element type in v0”; **only** for unsupported element TYPE, i.e. not Int). An unsupported compound OPERATOR (`/=`, `%=`, …) ⇒ **E0006** (as today: not in D1 grammar, fails at parse), never E0333.

D3. Vec semantics: `v[i] op= x` ≡ read the element at i and `v.set(i, cur op x)?`. Out-of-range index or i<0 ⇒ the same Err as `set` (propagated with `?`); only in `fn -> Result<_, Int>` (outside: E0344, same as ADR-283). Negative literal ⇒ E0319 (same as ADR-283).

D4. Evaluation order: (1) index, (2) rhs (left→right), (3) read cur, (4) `cur op x`, (5) write. rhs is evaluated before taking `&mut v`; since `v[j]` is `Option` (ADR-261), an rhs `v[j]` is E0203 and `v.get(j)` is not Int: no aliasing with the mutable loan.

D5. **CLOSED — D5-ii (Architect, 2026-10-03; Q4 closed with the Codegen PREP `DOC/reviews/PREP_ADR290_CODEGEN_20261003.md`).** Compound `v[i] op= x` (`+=`, `-=`, `*=` on Vec<Int>/List<Int>) is emitted with `checked_add`/`checked_sub`/`checked_mul`; `None` (overflow) ⇒ `Err` propagated with `?`, **same in debug and release** (`checked_*` do not depend on `overflow-checks`; PREP §2.2) and **with no new panic**. Negative (non-literal) index, out of range, and overflow ⇒ `Err` with `v` **intact** (write is the last step, D4.5); only in `fn -> Result<_, Int>` (D3). Failure order: out of range before overflow (must read `cur` to operate). **Explicit declaration:** `v[i] += x` is **STRICTER** than the Int expression `a + b`, which today emits `(a + b)` on `i64` without checked (panic in debug, wrap in release; PREP §1.4 and §2.1, ADR-045). The binary expression **does NOT change** in this slice and this ADR does not fix the ADR-045 hole; if rhs `x` is itself `a + b`, that inner sum keeps panic/wrap (D5 covers only the compound operation, not “the whole statement”). **Pin 1 (overflow Err value):** `Err(0)`, the same value as `Vec.set` OOB (`DOC/ADR/283-core-vec-assign-v0.md:23` “early-return `Err(0)`” and `:175` “`Err(0): Int` is SoT (270)”; PREP §2.3). **Declared:** the cause (OOB vs overflow) **is not distinguished in the Err value**; distinguishing it would need a separate ADR.

D6. Borrow rules reused unchanged: non-`mut` binding ⇒ E0202; moved ⇒ E0201; live/shared loan ⇒ E0202; rhs type ≠ Int ⇒ E0203; `x[i] op= y` with x Map/String/other collection ⇒ E0006, as today (same code, emitted by HIR when rejecting the receiver; ejemplos/core09/map-assign/neg/04-compound stays E0006); E0314 only for non-collection ident (e.g. Int). HIR must reproduce the **exact** text `E0006: construct outside F1.1 (parse failure)` (same format and span as today; HIR test `crates/arita-hir/src/lib.rs:8687`) for compound on Map, because with D1 grammar the parser no longer rejects it; oracle `neg-core09-map-assign-compound` must stay green (PREP §4.2.3, P5).

D7. Emit: no IndexMut/Index/AddAssign on index in emitted Rust; prelude helper `__arita_vec_update(v: &mut [i64], i: i64, x: i64, op: fn(i64, i64) -> Option<i64>) -> Result<(), i64>` (name and shape: PREP §3.1, alternative A: `usize::try_from(i)`, single `get_mut` lookup, `op(*slot, x)`, write at the end), dumped only if the module uses some Vec compound (like `__arita_vec_set`). Call site: `{ let __arita_vi = <i>; let __arita_vx = <x>; __arita_vec_update(&mut <v>, __arita_vi, __arita_vx, i64::checked_add)?; }` (`checked_sub`/`checked_mul` for `-=`/`*=`). Evaluation order index → rhs → read → operation → write, without double evaluation (D4; PREP §3.2). Same emit-bans as 282/283 (0 IndexMut/Index; no `+`/`-`/`*` between `slot` and `x`, nor `unwrap`/`expect`/`panic!`/`as usize`). The form with `i64::checked_add` as fn pointer **must be measured** (rustc + clippy on 2015 and 2021) before GO IMPL (PREP §3.2, P7); if it warns, return to the Architect (PREP §3.3 alternative B). Emitted code passes `-D unused_parens -D unused_braces` and clippy per ADR-289’s criterion.

D8. Map does NOT enter slice A. Slice B (future ADR/extension): `m[k] op= y` requires `fn -> Result` context and key miss ⇒ propagated Err (option b); NEVER silent 0 identity (Q1 RESOLVED, Engineer 02-10).

## OUT / HOLD

- `v[i][j] = x`, `v[i].campo = x`, `m[k].push(x)`, `&mut v[i]`, `&mut m[k]` as an argument: still E0006/HOLD (need subelement borrow + their own ADR + GO).
- String.set (indexes by chars), Mutex, D1 (params `&mut Map/Vec` + receiver loan): HOLD.
- E0314 split: HOLD (backlog ROADMAP L67-68).
- B-290-1 (P3): plain `x += 1` (compound assignment on ident), after the Mutex PREP.
- Slice B: Map compound (Q1 b).

## Codes

New E0333 (D2; **only** unsupported element type, e.g. Vec<Text>/Vec<Bool>). Unsupported compound operator ⇒ E0006 (D2), not E0333. E0334–E0339 free, reserved for slices B/C of this ADR. E0291, E0340–E0343 and ADR-265/270/278/283 are not reopened.

Meaning of codes used (per real ADR/HIR): **E0344** `index assign outside result fn` (context; `DOC/ADR/283-core-vec-assign-v0.md:14` and `:161`; `crates/arita-hir/src/lib.rs:597`) · **E0319** `negative set index` (negative literal index; `DOC/ADR/270-lang-contract-set.md:25`; HIR `lib.rs:564`) · **E0203** `type mismatch` (rhs type, or fn `E` ≠ Int; ADR-283 `:25`; HIR `lib.rs:531`) · **E0006** `construct outside F1.1 (parse failure)` (ADR-283 `:35`; HIR `lib.rs:8687`) · **E0333** (new in this ADR; not yet in HIR or another ADR).

**Pin 3 — precedence inside the same statement `v[i] op= x`:** E0344 > E0319 > E0333 > E0203. It is ADR-283’s rule (E0344 > E0319 > E0203, `:28`) with E0333 inserted between E0319 and E0203; it does **not** clash with “first offender in source order wins”, which still governs **between** statements (ADR-283 `:28` and `:207`; code precedence only governs inside one statement).

## Proposed oracles (provisional k = 6)

- IM-1 `v[i] += x` / `-=` / `*=` on Vec<Int> en `fn -> Result`: correct result (pos).
- IM-2 OOB index, negative, and **overflow** (`+=`/`-=`/`*=` at i64 bounds) at runtime ⇒ propagated `Err(0)`, `v` intact, no panic, same in debug and release (D5-ii) (pos).
- IM-3 emit-ban: 0 IndexMut/Index/AddAssign-index en el Rust emitido + `rustc`/`cargo check` limpio.
- IM-4 neg E0333: Vec<Text> `+=` en `fn -> Result` (tipo de elemento no soportado); un operador no soportado (`/=`, `%=`, …) ⇒ E0006, no E0333 (D2).
- IM-5 neg: non-mut ⇒ E0202; Text rhs ⇒ E0203; main ⇒ E0344; negative literal ⇒ E0319; `x[i] op= y` with x Map/String/other collection ⇒ E0006, as today (same code, emitted by HIR when rejecting the receiver; ejemplos/core09/map-assign/neg/04-compound stays E0006); E0314 only for non-collection ident (e.g. Int). Precedence E0344 > E0319 > E0333 > E0203 with one neg per pair (Pin 3).
- IM-6 migration: `vec-assign/neg/05-compound` moves from **negative (E0006) to POSITIVE** (it is `Vec<Int>` in `fn -> Result<Int, Int>`: `ejemplos/core09/vec-assign/neg/05-compound.arita:4-8`) ⇒ **new freeze + addendum + new fixture** (e.g. a `05-compound-ok.arita`; name fixed by IMPL; 05 is not rewritten and is withdrawn from `neg-core09-vec-assign-compound` only with Engineer addendum; PREP P8); negs 06-nested and 07-field stay E0006 unchanged. Migrating ejemplos/core09/vec-assign/neg/05-compound.arita changes a frozen fixture’s sha: done with a NEW freeze + addendum (DOC/reviews/…addendum.sha); the existing freeze is never rewritten. Same for map-assign/neg/04-compound if it changes (it does not in slice A: stays E0006).

Provisional N = prior N_CLOSED + k, computed by the Engineer at GO IMPL (queue order: 287 → 288 → 289 → 290 → Mutex). Provisional: 856 (ADR-289 CLOSED) + k = 6 ⇒ **862**; the Engineer recalculates it.

## Open questions (Engineer)

Q1. **RESOLVED (Engineer 02-10):** Map does NOT enter slice A. Slice B (future ADR/extension): `m[k] op= y` requires `fn -> Result` context and key miss ⇒ propagated Err (option b); NEVER silent 0 identity. Original question: Map compound `m[k] += x`: put returns no Result, so a key miss must be total without `?`. Options: (a) miss ⇒ identity (0) only for `+=`/`-=`, `*=` ⇒ E0333; (b) require Result context and Err on miss; (c) keep E0006. Architect recommendation: slice A Vec only, Map in slice B after decision.

Q2. **RESOLVED (Engineer 02-10):** Int-only in v0. Original question: Admit Float/other numerics in slice B or keep Int-only.

Q3. **RESOLVED (Engineer 02-10):** plain `x += 1` = B-290-1 (P3), after the Mutex PREP, not before. Original question: plain `x += 1` as a separate ADR (B-290-1) before or after?

Q5. **RESOLVED (Engineer 02-10):** separate oracles IM-1..IM-6 (k=6), none >10 min on Lex. Original question: IM-1..IM-6 in one oracle or several (≤10 min on Lex per run criterion, as ADR-289).

## Proceso

DOC pins → Engineer verification → GO IMPL. The “after ADR-289 CLOSED” condition is **MET** (ADR-289 CLOSED 2026-10-03). **Still NO GO IMPL** (Engineer gives it after verifying v0.3 and P7 measurement). Implementation order: **Parser → HIR → Codegen** (AST gains `op`; codegen is not testable without the field; PREP §4.1, P6). Provisional N 862 (856 + k = 6), Engineer recalculates. Do not edit ROADMAP.

## Changelog

- 2026-10-03 — v0.3 DRAFT PINS (Architect): Q4/D5 closed → D5-ii (`checked_*`, `None` ⇒ `Err`, same in debug and release; `v[i] op= x` stricter than `a + b`, which does not change); Pin 1: unique `Err(0)`, cause undistinguished; Pin 2: unsupported operator ⇒ E0006, E0333 only for element type (D2, Codes, IM-4); Pin 3: precedence E0344 > E0319 > E0333 > E0203; D6: HIR reproduces exact E0006 text for Map; D7: helper `__arita_vec_update`; IM-2/IM-4/IM-5/IM-6 adjusted (05-compound becomes positive: new freeze + addendum + new fixture); order Parser → HIR → Codegen; “after ADR-289 CLOSED” met; provisional N 862; no GO IMPL. Base: Codegen PREP `DOC/reviews/PREP_ADR290_CODEGEN_20261003.md` (sha256 `25d0783112f8e8f68ba4019eba943914045cb65b736ec28a597e59309e842127`). (The file had no prior changelog section.)

- 2026-10-03 — Seal (Engineer): v0.3 reviewed and APPROVED; GO IMPL with k = 6, N = 862, order Parser → HIR → Codegen, Codegen conditioned on P7; pins 1–3 accepted as-is; B-290-2 (P2): semantics of `a + b` on Int (panic in debug, wrap in release) to pin before v1.
- 2026-10-03 — CLOSED (GATE `DOC/GATE-CORE10-INDEX-MUT-20261003.md`); close in the “Close” section (end of file; this ADR’s sections are not numbered beyond this note).

## Close (CLOSED 2026-10-03)

Datos tomados de `DOC/GATE-CORE10-INDEX-MUT-20261003.md` (sha256 `2a653cff898e18d191d4b79b611207849432b1eaff617c435c523d5db5d4cc7d`).

- **Veredicto:** GO CLOSED ADR-290 slice A (Ingeniero Rust, 2026-10-03). Medida final **862/862 accepted**, 0 skip; k = 6 (IM-1..IM-6), N previo 856 → N = 862. CUT-ID del GATE `CORE-0.10-INDEX-MUT-20261002` (el mismo del ADR; sin cambio).
- **Oráculos** (ids decided by the Engineer): `core10-index-mut-compound-ok` (IM-1: `+=`, `-=`, `*=` on Vec<Int> in `fn -> Result`, build ok and exact stdout) · `core10-index-mut-err-propagation` (IM-2: OOB index, negative and `+=`/`-=`/`*=` overflow ⇒ propagated `Err(0)`, `v` intact, no panic, in debug and release) · `core10-index-mut-emit-ban` (IM-3: helper `__arita_vec_update` + `i64::checked_*` calls; 0 IndexMut/Index/*Assign/unwrap/expect/panic!; clean `unused_parens` and clippy `-D warnings` on 2015 and 2021) · `core10-index-mut-neg-unsupported` (IM-4: `v[0] /= 2`, `v[0] %= 2` and annotated `Vec<Text>` ⇒ **exact E0006**) · `core10-index-mut-neg-rules` (IM-5: 10 negs with their exact code: E0202, E0203, E0344, E0319, E0006 Map/String, E0314 Int, and the reachable precedence pairs E0344>E0319, E0344>E0203, E0319>E0203) · `core10-index-mut-migrated-05` (IM-6).
- **Evidencia** (exclusive Lex run, 2026-10-03 10:09 → 12:30, `/tmp/ing-adr290-final`): `cargo fmt --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` 0; `cargo build --release` 0; `cargo test --workspace -- --test-threads=1` 562 passed / 0 failed; `arita measure` exit 0 (ends 11:45), pure JSON stdout, 862 unique ids, 862 accepted, 0 skip, prior 856 unchanged vs `MEASURE_ADR289_UNUSED_PARENS_EXCLUSIVE_20261003.json` and the 6 new accepted; Miri inside measure and Veyra (green).
- **Veyra Proof** (`--profile quick`): **ACCEPTED** `20261003T094540Z` (rustfmt, cargo check, clippy, cargo test, cargo-audit, veyra-anti-theater; 0 problems, 0 skipped). The GATE cites no prior REJECTED run. Tools: rustc/cargo 1.97.1, clippy 0.1.97; `~/.cargo/bin` (clippy-driver) on PATH.
- **Artefacto de medición:** `DOC/reviews/MEASURE_ADR290_INDEX_MUT_EXCLUSIVE_20261003.json` (sha256 `97ae00c8776f…`). **Freeze:** `DOC/reviews/MEASURE-ADR290-INDEX-MUT-FREEZE-20261003.sha` (1029 lines: 6 header + 1023 verifiable with `shasum -a 256 -c`; sha256 `2eae386b83b0…`) and its addendum `DOC/reviews/MEASURE-ADR290-INDEX-MUT-FREEZE-20261003.addendum.sha` (GATE cites it without path; the path is that of the existing file).
- **Shas de código** (identical before and after the run): `crates/arita-codegen/src/lib.rs` `c9b3e7d6…` → `e8df9366dc74…`; `crates/arita-hir/src/lib.rs` `7983cecc…` → `d0571e12419d…`; `crates/arita-syntax/src/lib.rs` `16b4191e…` → `187cff0e4bb0…`; `crates/arita-syntax/src/arita.pest` `569b15d1…` → `7c28aa3c627a…`; `crates/arita-cli/src/measure.rs` `4a04ab1e…` → `c4dc13a73339…`; `main.rs` `ed0820b4…` and `package.rs` `5fee2beb…` unchanged. ADR-290 `c3adf806…` (sealed v0.3, before this close edit).

### Engineer decisions incorporated at close (03-10; see also D5–D6, Codes, and IM-4..IM-6)

- **D5-ii:** `v[i] op= x` uses `checked_add`/`checked_sub`/`checked_mul`; overflow ⇒ `Err(0)`, same in debug and release. **Stricter than `a + b`**, which does not change in this slice (ADR-045).
- **Err(0):** the same value as `Vec.set` OOB; the cause (OOB vs overflow) is not distinguished in the Err value.
- **D6:** compound on Map/String/Bytes ⇒ **E0006**, with the ADR prefix + ` @start..end` (HIR emits it); other non-collections ⇒ **E0314** (`n[0] += 1` moves from E0006 to E0314). An unsupported operator (`/=`, `%=`) ⇒ E0006 emitted by the Parser.
- **IM-4 (prevalece sobre el texto de v0.3, que hablaba de un neg E0333):** IM-4 measures exact E0006; **E0333 is not reachable from source** (the grammar is Int-only and the Parser search offers no real path to a non-Int Vec/List; `Vec<Text>` fails in the parser with E0006), stays as defense-in-depth proven only by HIR tests, without faking an E0333 neg. For the same reason the Pin 3 precedence pairs with E0333 are not reachable from source; precedence E0344 > E0319 > E0333 > E0203 still holds (GATE, “Decisions and semantics”).
- **IM-6 / 05-compound:** `ejemplos/core09/vec-assign/neg/05-compound.arita` (frozen, unmodified) moves from E0006 to valid; withdrawn from `neg-core09-vec-assign-compound` (3 → 2 cases) via freeze addendum; new positive `core10/index-mut/05-compound-ok.arita` (exists under `ejemplos/core10/index-mut/`). `ejemplos/core09/vec-assign/evidence.json` (neg_compound: E0006) is documentarily obsolete; not rewritten.

### Backlog abierto

- **B-290-2 (P2):** `a + b` Int without checked: today `(a + b)` on `i64` (panic in debug, wrap in release; ADR-045); must be pinned before v1.
- **B-290-1 (P3):** simple `x += 1`. **B-289-1 / B-289-2 (P3)** (other rustc lints; the 15 supplemental workspaces without their own oracle). Those the GATE declares (“Open”).

### Next order

ADR-291 MUTEX-REJECT (real Mutex to v1.1). The GATE also says Docs aligns figures to N = 862.

Note: ADR sha after this close: recorded by the Engineer addendum.
