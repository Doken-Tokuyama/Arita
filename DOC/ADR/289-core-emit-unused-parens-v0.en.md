Translation of `289-core-emit-unused-parens-v0.md`; the original is normative. / Traducción de `289-core-emit-unused-parens-v0.md`; el original es el normativo.

# ADR-289 — Core emit unused-parens v0: no redundant parentheses in emitted Rust (B-283-1 + `capacity()` + “zero rustc warnings on positives” criterion)

- **Estado:** **CLOSED (Ingeniero, GO CLOSED 2026-10-03; DOC/GATE-CORE10-EMIT-UNUSED-PARENS-20261003.md)** · `arita measure` 856/856 accepted (N = 856, k = 3) · historia: v0.1 APROBADO Y CONGELADO (Ingeniero 2026-10-02, sha `f10574bfe7085de81ddc47ce7bb391ff8c0ca7996370bc28cd3d299fcbe6fe9f`), Addendum 1 (§11, 2026-10-03), GO IMPL tras ADR-288 CLOSED; freeze `DOC/reviews/MEASURE-ADR289-EMIT-UNUSED-PARENS-FREEZE-20261003.sha` y addendum de freeze `DOC/reviews/MEASURE-ADR289-EMIT-UNUSED-PARENS-FREEZE-20261003.addendum.sha`.
- **CUT-ID:** `CORE-EMIT-UNUSED-PARENS-20261001` (provisional; lo confirma el Orquestador)
- **Fecha:** 2026-10-01
- **Autores:** ARITA Arquitecto (pins) · alcance y orden decididos por el Ingeniero 01-10 (S1b → PKG-MEMBER 287 → EMIT-CLIPPY-B282 288 → **este** → IndexMut → Mutex); pedido original de <person> (unused_parens en el Rust emitido)
- **Padre / contexto:** [ADR-282](282-core-map-assign-v0.md) (B-282-2, `strip_one_outer_parens`) · [ADR-288](288-core-emit-clippy-b282-v0.md) (B-282 medido; deja B-283-1 aquí) · [ADR-285](285-core-ref-mut-v0.md) y [ADR-286](286-core-0.10-errores-fase1.md) (freezes con `ejemplos/core09/ref-mut/evidence.json`) · inventario estático de Codegen `DOC/reviews/PREP_UNUSED_PARENS_INVENTARIO_20261001.md` (md5 `7d9aca74b16d4d37e11fc8edb124a183`, verificado al redactar; en adelante **INV**)
- **Cierra:** **B-283-1** · caso `let c: i64 = (v.capacity() as i64);` de `ejemplos/f2/184-shrink-to-fit-vec.arita` · criterio «cero warnings de rustc en el Rust emitido de los positivos» (alcance acotado a cero `unused_parens`, §11) (CLOSED 2026-10-03).
- **Códigos:** **ninguno** (warnings del Rust emitido, no diagnósticos).
- **No reabre:** ADR-282/283/284/285/286/287/288 · E0291 / E0340–E0343 · ADR-265 / 270 / 278 / 283. Los GATE 283/284/285 no se reescriben (addendum propio, §7).

## 1. Facts (from INV; static, no real count yet)

INV is a **static** inventory: there is no rustc/clippy warning count until the dynamic PREP (§6) produces it. Today the number of positives that warn is **unknown**, not zero. On `crates/arita-codegen/src/lib.rs` (md5 `57fc2a6c…` in INV):

- **(a) Binary:** one producer, `({l} {o} {r})` (L2204), and **23 consumption sites** where the result falls straight into a linting context (let init, assignment, `if`/`while` cond, `match`/`if let`/`while let` scrutinee, block arm and tail, `Ok`/`Err`/`Some` and `emit_user_call` args, and 6 `let __x = {a};` of `swap`/`get`/`swap_remove`/`remove`/`is_char_boundary`/`is_multiple_of`).
- **(b) Borrow:** `(*&mut x)` / `(*&x)`, 2 productores (L2210/L2212), mismos contextos.
- **(c) Casts parentizados:** 11 productores `(x.f() as i64)` (L3275–3565: `count_ones`…`trailing_ones`, `abs_diff`, `len`, `capacity`, `floor/ceil_char_boundary`).
- **(d) Argumentos directos:** 57 sites (`41` one-arg, `5` two-arg, `11` of the `arita_host_http` bridge) that lint only if the argument already arrives parenthesized; the 15 of form `&{a}` do **not** lint.
- **(e) Macros:** `println!("{}", {e})` (L2147) and `assert_eq!({l}, {r})` (L1830): **NOT VERIFIED** whether rustc lints (static read: probably not, `format_args!` and `assert_eq!`’s `match (&(a), &(b))`).
- **12 sitios que NO se tocan** for precedence (INV §2: `resize/try_reserve*/reserve/shrink_to(({a}).max(0) as usize)`, `checked_pow/shl/shr`, `wrapping_shl/shr` with `({a}) as u32`, `floor/ceil_char_boundary`), plus the `({ms} as u64)` (L2156/2377/2392/2404) and `{n} as usize` (L2923).
- Ya arreglado y que se conserva: `strip_one_outer_parens` en `let n: Int = x.len()` (L1820) y en `IndexAssign` (Vec L1889–1890, Map L1900–1901).
- Codegen test with expected string that **would change**: `lib.rs` L3772, L4026 (only if `println!` changes, see D4), L3835, L3936, L3937, L4121. Still valid: L4744, L4614, L5187–5196.
- `measure.rs`: today **no** oracle walks positives with rustc/clippy `-D warnings`; the only emit-clippy is `core09-map-assign-emit-clippy` (≈L9705–9813), and `run_ejemplo_oracle` (≈L4637–4660) compiles and runs without `-D warnings`. Universe: 398 positive `EJEMPLO_ORACLES` (538 `.arita` outside `*/neg/*`, 295 under `*/neg/*`).
- **Critical point:** `ejemplos/core09/ref-mut/evidence.json` declares hashes of **emitted** Rust (`lib_rs_sha256`, `main_rs_sha256`, `edge_lib_rs_sha256`, `edge_main_rs_sha256`) that oracle `core09-ref-mut-evidence` recomposes (`measure.rs` ≈L11563–11682); `evidence.json` is hashed in `MEASURE-ADR285-FREEZE-20260927.sha` L381 and `MEASURE-ADR286-FREEZE-20260927.sha` L383. That example emits `println!("{}", (m.len() as i64))` (`bin/main.arita:26`, `edge/bin/main.arita:26`).

## 2. Decisions (pins)

**D1 — Mechanism: emit context.** `ExprCtx { Operand, Bare, Head }` and `emit_expr_ctx(e, ctx)` are introduced. `emit_expr(e)` remains an alias with `Operand` (current behavior, byte for byte). Only `Binary` (L2204), `Borrow` (L2210/2212), and the 11 casts of (c) change: with `Operand` they emit as today (with parentheses); with `Bare` or `Head` they emit **without the outer paren**. Children of a `Binary` are **always** emitted with `Operand`. Each consumption site is migrated **explicitly**; an unmigrated site does not change. The general mechanism of string-patching via `strip_one_outer_parens` at each consumption is rejected (fragile: forgets D3’s guards); `strip_one_outer_parens` and its tests are kept.

**D2 — Migration scope.** `Bare` in: let init, assignment, function/method args (families (a) and (d)), `Ok/Err/Some(..)`, `emit_user_call` args, `let __x = {a};` (6), `match` arm, block and fn tail. `Head` in: `if`/`while` condition and `match`/`if let`/`while let` scrutinee (the 3 paths: stmt, `Expr::Match`, `emit_match_stmt_as_expr`). **Not migrated:** the 12 precedence sites of INV §2, the `({ms} as u64)` and `{n} as usize`, and the 15 `&{a}`.

**D3 — Mandatory guards (without them there is no GO IMPL).**
1. A `Binary`/cast that is an **operand** (either side) of another `Binary`, `Cast`, or method receiver is always emitted with `Operand` (with parentheses).
2. A cast that is the left operand of `<` or `<<` **keeps** the parenthesis (`(v.len() as i64) < 3`; without it it does not parse).
3. `Head` **keeps** the parenthesis if the expression contains an exterior struct literal (`contains_exterior_struct_lit` in rustc); also when needed for parse.
4. Double parentheses `((x + 1)).max(0)` are the result of the 12 unmigrated sites; they are not a target.
5. The operand of a **unary** (`-(a + b)`, `!(a && b)`), the left operand of **`as`** (`(a + b) as i64`), and of **indexing/range** keep parentheses when required for parse/precedence.
6. Values of `return (..)` / `break (..)`: the dynamic PREP decides whether rustc lints them; if they lint they move to `Bare`, otherwise they stay as-is.
Each guard has its fixture (§5, PU-2) and its cargo test (§4). Guard 5 also has its own cargo test for the unary.

**D4 — `println!` / `assert_eq!` (family (e)) and the ref-mut hashes.** Resolved by the Engineer (Q1): the criterion (zero `unused_parens` warnings in the emit) prevails over freeze convenience; **nothing is removed from scope to avoid a re-freeze, only for not linting.** (a) If the dynamic PREP shows rustc does **not** lint `println!`/`assert_eq!` arguments, they stay **`Operand`** and the four hashes of `ejemplos/core09/ref-mut/evidence.json` must stay **identical**, with `core09-ref-mut-evidence` green **without modifying `evidence.json`** (PU-4 close criterion). (b) If rustc **does** lint them, or if any ref-mut hash changes from another migration, they are **migrated** and **`evidence.json` is re-frozen with an addendum in this slice’s NEW freeze**, recording the old and new sha; freezes 285/286 (`MEASURE-ADR285-FREEZE-20260927.sha` L381, `MEASURE-ADR286-FREEZE-20260927.sha` L383) **are not rewritten**. No Engineer BLOCKER is pending for this point.

**D5 — `capacity()`.** The `capacity` cast (L3539) is migrated like the others of (c); `ejemplos/f2/184-shrink-to-fit-vec.arita` starts emitting without the outer paren on that cast.

**D6 — No semantic change.** The observable program (stdout, exit) of all positives is identical before and after. Any difference is a BLOCKER.

## 3. Criterion and measurement mechanism

Engineer criterion: “zero rustc warnings in the emitted Rust of the positives, without changing semantics”. Pins (Q2 resolved): `-D unused_parens -D unused_braces` are measured on **all** positives; scope widens to full `-D warnings` **only if** the dynamic PREP shows the corpus clean. Other lints that appear are recorded as **B-289-n (P2**: they are real warnings in emitted code) and **do not block** this slice. Mechanism, by positive class (the dynamic PREP must **classify each positive** into one of the two): (i) **`rustc`** on the emitted `.rs` (edition 2015 without `--edition`, like `arita build`, and 2021 for the Cargo path), `--emit=metadata`, for single-file positives without dependencies; (ii) **`cargo check`** in a scratch crate with `RUSTFLAGS="-D unused_parens -D unused_braces"` and a shared target dir, for positives that emit a Cargo crate with dependencies (async/tokio, `[deps]`, host-bridges, http bridge), which bare `rustc` does not compile. **No positive is silently excluded:** if it cannot be measured ⇒ **inconclusive and counts as not accepted**; missing `rustc`/`cargo` ⇒ inconclusive, never accepted; skip ≠ PASS.

## 4. Cargo tests (codegen, `crates/arita-codegen/src/lib.rs`; names and count fixed by IMPL)

Migrated (expected string changed, name kept): L3835 `let sum: i64 = 1 + 2;` · L3936 `while i < 2 {` · L3937 `i = i + 1;` · L4121 `a + b` as fn tail. L3772 and L4026 (`println!`) **do not change** if D4(a). Added: one test per D3 guard (left cast of `<`; struct literal under `Head`; `Binary` child; unary/`as`/indexing operand); one test per unmigrated INV §2 site (emitted string unchanged); one `capacity` test (D5). Intact: L4744, L4614, L5187–5196 and the two `b282_emit_clippy`. Close criterion: `cargo test --workspace` green, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, miri-workspace green if it was green at the prior CLOSED.

## 5. `measure` oracles (proposed; exact N fixed by the Engineer)

- **PU-1 (verde, corpus):** for **each** positive in `EJEMPLO_ORACLES` (398 today), emitted Rust is measured by its §3 class (`rustc` or `cargo check`) with `-D unused_parens -D unused_braces`, on both editions where applicable; **N positives reviewed == current table length** (oracle fails if the table grows and is not walked). Also covers positives that are **not** in `EJEMPLO_ORACLES` but are under `ejemplos/` (538 `.arita` outside `*/neg/*` vs 398 oracles): the dynamic PREP says how many and why they lack an oracle; those without expected stdout **are not measured as oracles but are listed**. One oracle by default (Q3 resolved); split only if **measured** PREP time exceeds **10 min on Lex**; if split, the “N reviewed == table length” invariant is checked over the **sum of the parts** and k is recalculated at GO IMPL.
- **PU-2 (green, guards and precedence non-regression):** new minimal fixtures under `ejemplos/core10/unused-parens/`: cast left of `<`, struct literal under `if`/`match`, `.max(0) as usize` (the 12 non-migrated sites), `checked_pow(.. as u32)`, unary operand (`-(a + b)`, `!(a && b)`), left operand of `as` and of indexing; each **compiles, does not warn, and prints exact stdout** (ids and output fixed by IMPL with the matching cargo test programs).
- **PU-3 (verde, `capacity`):** `ejemplos/f2/184-shrink-to-fit-vec.arita` emitted contains `let c: i64 = v.capacity() as i64;`, no warning, same output as today.
- **PU-4 (regression):** `core09-ref-mut-evidence` stays accepted **with no changes to `evidence.json`** (D4(a)); not a new oracle, an explicit close criterion.

Provisional k = 3 (PU-1…PU-3). **N is not fixed here:** N = prior N_CLOSED + k, at GO IMPL. Provisional with the current order: 850/851 (PKG-MEMBER) → 853 (288) → **856** (this). Rules: skip ≠ PASS; no oracle counts if rustc did not run. Corpus oracles must not modify the real repo (sha256 manifest before/after).

## 6. PREP (blocks GO IMPL)

1. **Dynamic Codegen PREP** (INV §5, with Lex free and after S1b CLOSED): for each positive, `rustc`/`cargo check` with `-W unused_parens -W unused_braces` (and `-W warnings` to count other lints) on the real emit, on both editions; **classify each positive as `rustc` or `cargo check`**; table positive → class → lint → context → line; aggregated by family (a)–(e). Must resolve: does `println!`/`assert_eq!` lint? do `return (..)`/`break (..)` lint (D3.6)? how many positives warn today? are there other-lint warnings (§3 scope, B-289-n)? how many of the 538 `.arita` have no oracle and why?
2. **Measure PREP:** that no existing oracle checks the parenthesis string of emitted Rust (INV: 0 assertions in `measure.rs`/`tests/`, by patterns), and which oracles recompose emitted-Rust hashes (today only `core09-ref-mut-evidence`; no other `evidence.json` with those keys).
3. Measure PU-1’s time on the real corpus before fixing it as an oracle; if it exceeds what is reasonable for the gate, it is split (Q3). Not estimated here.

## 7. Close y GATE

At CLOSED: own addendum `DOC/GATE-CORE10-EMIT-UNUSED-PARENS-<fecha>.md` that marks B-283-1 and `capacity()` fixed; GATE-283/284/285 and REF-MUT/VEC-ASSIGN/SCENARIO-MUT are not rewritten. If `evidence.json` is re-frozen (D4 b), the addendum records it with the old and new sha. The addendum also lists the consequence for migrated tests (L3835, L3936, L3937, L4121 and those the PREP adds).

## 8. Draft questions — resolved by the Engineer (01-10)

- **Q1 (D4):** <person>’s criterion rules; if `println!`/`assert_eq!` lint or a ref-mut hash changes: migrate and re-freeze `evidence.json` with an addendum in this slice’s new freeze; if they do not lint, identical `Operand` and hashes.
- **Q2 (§3):** `-D unused_parens -D unused_braces`; full `-D warnings` only if PREP shows a clean corpus; other lints, B-289-n (P2), do not block.
- **Q3 (PU-1):** single oracle; split only if measured time exceeds 10 min on Lex.

## 9. Slice and GO

Single slice `EMIT-UNUSED-PARENS` (size M: changes codegen emit, ~100 sites, 6 migrated tests, 3 oracles). Sequence: DOC (this ADR) → Engineer verification → dynamic PREP + Measure PREP (§6) → Q1–Q3 answer → Engineer **GO IMPL** with ADR-288 CLOSED → implementation (Codegen/Orchestrator) → Lex → CLOSED with real evidence. Inventing PASS / N/N / CLOSED / counts is forbidden.

## 10. Changelog

- 2026-10-01 — v0 DRAFT PINS (Architect). Based on INV (static, md5 `7d9aca74…`); no dynamic count or own execution; the mechanism and guards are design, not a measured result.
- 2026-10-01 — v0.1 DRAFT PINS (Architect): Engineer Q1–Q3 resolved; PREP/`rustc`/`cargo check` classes (§3, §6); guards 5 and 6 in D3; PU-1 covers positives without an oracle and sets a 10 min threshold; §7 with migrated tests. Pending Engineer approval.
- 2026-10-03 — Addendum 1 (Architect, Engineer decisions after the dynamic Codegen PREP): §11; criterion narrowed to zero unused_parens; B-289-1 P3; in/out scope; without rewriting the current text.
- 2026-10-03 — Note: the v0.1 changelog line (2026-10-01) that says “Pending Engineer approval” is superseded: the ADR is APPROVED AND FROZEN (Status, L3; Engineer 2026-10-02).
- 2026-10-03 — CLOSED (GATE `DOC/GATE-CORE10-EMIT-UNUSED-PARENS-20261003.md`); close in §12.

## 11. Addendum 1 (2026-10-03, Engineer decisions after the dynamic PREP)

Does not rewrite the current text (L1–L88); where it conflicts, this addendum prevails only as indicated.

- **Base:** Codegen dynamic PREP `DOC/reviews/PREP_ADR289_UNUSED_PARENS_DYNAMIC_20261002.md` (sha256 `fa1cd82865970ff5f6386edfbaafb54739772a3f130a8518a49fb0ff97490863`, md5 `27d2d5826ddd9c890e698679d19322d9`, 161 lines). Starting proposal: `DOC/reviews/ADR289-ADDENDUM-PROPOSAL-20261002.md` (sha256 `1c44c1ef2fcbbe2f45481ad51e09ba067a10846a4b937181afa0032bd3390fbe`).
- **Criterio acotado:** “zero rustc warnings” (title, “Closes” L8, §3 L50) is **BOUNDED to zero `unused_parens`** in emitted Rust of measured positives; it does **not** mean zero rustc warnings. Other lints (`unused_variables`, `dead_code`, `unused_mut`; 35 positives with other lints, PREP L47) move to **B-289-1 (P3)**. PREP data: 1791 `unused_parens` warnings in 93 positives on 2021 (L23); 278 positives at 0 today + 85 with only `unused_parens` = **363/398 at zero after the fix** (derived from L47).
- **Alcance DENTRO:** `Binary` in let/assignment/`if`/`while`/fn-tail; casts; `Borrow`; plus function/method args, `Ok(..)`, `match` scrutinee and `return`, with minimal regression on the PREP’s minimal cases (L100 `fn_arg_binary`, L105 `match_scrutinee`, L106–L107 `method_arg_*`, L108 `ok_wrapper`, L113 `return_binary`). **OUT:** macro args (`println!`, `assert_eq!`, `vec!`, …), which do not lint (PREP L117). This addendum prevails over D2 (L31, which had those contexts in `Bare`) where it contradicts; D3 guards 1–6 (L33–L40) still govern when a parenthesis is NOT redundant.
- **Evidence:** `ejemplos/core09/ref-mut/evidence.json` is not re-frozen; if the emitter changes Rust of a frozen evidence, Codegen reports it **beforehand** and it is resolved with an addendum (`main_rs_sha256` / `edge_main_rs_sha256` hashes depend on ref-mut `println!`, PREP L122–L132). PU-4 remains the `core09-ref-mut-evidence` regression.
- **k/N:** provisional k = 3 (PU-1..PU-3) + minimal regression of the added contexts (PU-4 remains a regression); provisional N **856** (ADR-288 CLOSED N = 853, +3); the Engineer recalculates at GO IMPL.
- **Ediciones medidas** (Architect proposal, no objection so far): the 398 positives on 2021; 2015 where it compiles (366; the 32 with `async`/`.await` from `cargo` are not measurable on 2015, PREP L22–L25). Fixes the mismatch with §3 (L50), where 2015 was measured on `rustc` positives and 2021 only on the Cargo path, while PREP measures all 398 on 2021.
- **GO IMPL:** la condición (1) del Estado (ADR-288 CLOSED) se cumple el 2026-10-03; siguen (2) PREP de Measure sin BLOCKER y (3) GO IMPL del Ingeniero. (El PREP dinámico de Codegen, parte de (2), ya está entregado: `PREP_ADR289_UNUSED_PARENS_DYNAMIC_20261002.md`.)
- **Cierre:** own GATE `DOC/GATE-CORE10-EMIT-UNUSED-PARENS-<date>.md` (unchanged vs §7).

## 12. Close (CLOSED 2026-10-03)

Datos tomados de `DOC/GATE-CORE10-EMIT-UNUSED-PARENS-20261003.md` (sha256 `1f00e51a26d27c3c1c8c00903c540fc0c5f87ba6c28c27c851b585d6c3a2a7f0`).

- **Veredicto:** GO CLOSED ADR-289 (Ingeniero Rust, 2026-10-03). Medida final **856/856 accepted**, 0 skip; k = 3 (PU-1..PU-3), N previo 853 → N = 856.
- **New oracles:** `core10-unused-parens-corpus` (PU-1: 398 corpus positives; 363 `rustc` class on 2015 and 2021 and 35 `cargo` class on 2021; 0 `unused_parens`) · `core10-unused-parens-guards` (PU-2: 2 fixtures under `ejemplos/core10/unused-parens/`, build ok, exact stdout, forms pinned in emit, clean `rustc -W unused_parens` on 2015 and 2021) · `core10-unused-parens-capacity` (PU-3: `ejemplos/f2/184-shrink-to-fit-vec.arita`, emit `let c: i64 = v.capacity() as i64;`). **PU-4 (regression):** existing `core09-ref-mut-evidence`, accepted, `evidence.json` intact; **no new id**. Prior 853 unchanged vs `MEASURE_ADR288_EMIT_CLIPPY_EXCLUSIVE_20261003.json`.
- **Evidence** (exclusive Lex run, 2026-10-03 05:47 → 09:00, `<scratch>/ing-adr289-final`): `cargo fmt --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` 0; `cargo build --release` 0; `cargo test --workspace -- --test-threads=1` 520 passed / 0 failed (the `arita-cli` test that launches real miri takes ~4500 s); `arita measure` exit 0 (ends 08:16), pure JSON stdout, 856 unique ids, 856 accepted, 0 skip; Miri inside measure and Veyra (green).
- **Veyra Proof** (`--profile quick`): **ACCEPTED** `20261003T061643Z` (rustfmt, cargo check, clippy, cargo test, cargo-audit, veyra-anti-theater; 0 problems, 0 skipped). The GATE cites no prior REJECTED run. Tools: rustc/cargo 1.97.1, clippy 0.1.97; `~/.cargo/bin` (clippy-driver) on PATH.
- **Alcance medido:** `unused_parens` 1791 → 0 (2015/2021 and workspaces); 93 of 398 positives change emitted Rust, parentheses only.
- **Measurement artifact:** `DOC/reviews/MEASURE_ADR289_UNUSED_PARENS_EXCLUSIVE_20261003.json` (sha256 `2252d84bcfd1…`). **Freeze:** `DOC/reviews/MEASURE-ADR289-EMIT-UNUSED-PARENS-FREEZE-20261003.sha` (1004 lines: 5 header + 999 verifiable with `shasum -a 256 -c`; sha256 `85ce0df5f33d…`) and its addendum `DOC/reviews/MEASURE-ADR289-EMIT-UNUSED-PARENS-FREEZE-20261003.addendum.sha` (GATE cites it without path; the path is that of the existing file).
- **Code shas:** `crates/arita-codegen/src/lib.rs` `9e654825…` → `c9b3e7d6ee38…`; `crates/arita-cli/src/measure.rs` `159a07ad…` → `4a04ab1e0920…`; `main.rs` `ed0820b4…` and `package.rs` `5fee2beb…` unchanged. Fixtures: `01-contexts` `2d162426…`, `02-precedence-sites` `8cf56a98…`. ADR-288 `0a962e12…` and ADR-289 `6fda6661…` (before this close edit).
- **Limitaciones declaradas:** other lints out of scope (B-289-1, P3). The 15 supplemental workspaces have no own oracle (B-289-2, P3); the 11 that change were verified by Codegen (cargo build 11/11; identical clippy except disappearance of `unused_parens`; 10 keep preexisting lints).

### Engineer decisions incorporated at close (03-10, via Orchestrator; see also §11)

- Oracle ids `core10-unused-parens-*` (`-corpus`, `-guards`, `-capacity`).
- k = 3 y N = 856 definitive; PU-4 = `core09-ref-mut-evidence` existing, no id.
- New contexts (function/method/`Ok(..)`/`match`/`return` arguments) are covered as **Codegen unit tests**, not as oracles (the GATE does not mention these unit tests; the freeze addendum does: au1..au10, 77 tests).
- PU-2 reformulated accordingly: fixtures under `ejemplos/core10/unused-parens/` (`01-contexts`, `02-precedence-sites`) for oracle-measured contexts; new contexts via unit tests.
- Editions measured: 2015 where `arita build` uses it (363 positives) and 2021 on the 35 of `cargo`; (corrected, per GATE: PU-1 also measures the 363 `rustc` class on 2021, i.e. 363 on 2015 and 2021 and 35 on 2021).
- Los 15 workspaces suplementarios → **B-289-2 (P3)**.
- PU-1 tolerated up to 15 min (Engineer decision; the GATE does not record PU-1’s time).

### Backlog abierto

- **B-289-1 (P3):** other rustc lints on positives (`unused_variables`, `dead_code`, `unused_mut`; 35 positives per PREP, L47).
- **B-289-2 (P3):** the 15 supplemental workspaces without own oracle (16 `unused_parens` on 2021 per PREP, L152; after close 0 per GATE).

### Next order

ADR-290 (IndexMut) → Mutex. The GATE indicates for ADR-290 a read-only Codegen PREP (Int overflow, read+set helper) and GO IMPL when the PREP closes Q4/D5. The export is regenerated with N = 856 when the Engineer asks.

Note: ADR sha after this close: recorded by the Engineer addendum.
