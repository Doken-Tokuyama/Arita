Translation of `294-core-known-int-scope-v0.md`; the original is normative. / Traducción de `294-core-known-int-scope-v0.md`; el original es el normativo.

# ADR-294 — Core KNOWN-INT-SCOPE v0: `known_int` scope at branch, loop, and block boundaries (B-292-2, P1, enters v1)

- **Estado:** **DRAFT PINS v0.2 (Arquitecto, 2026-10-04; decisiones del Ingeniero incorporadas; respuestas Q1–Q5 incorporadas; pendiente Sello del Ingeniero).** Sin GO IMPL; pendiente la PREP de Measure (§7). No declara PASS, CLOSED ni N/N medido.
- **CUT-ID:** `CORE-0.10-KNOWN-INT-SCOPE-20261004`
- **Fecha:** 2026-10-04
- **Autores:** ARITA Arquitecto (pins) · decisiones del Ingeniero 2026-10-04 · hechos de Parser/HIR en `DOC/reviews/PREP_B292_2_20261004.md` (sha256 `b6cf58f84b488d82b3653e1e28b7a85db871ec2baa5004326515e9bb6c73d473`, 383 líneas; en adelante **PREP**) y 17 casos en `/tmp/arita_b292/cases/` (fuera del árbol)
- **Padre / contexto:** [ADR-292](292-core-int-arith-runtime-v0.md) (D5 / B-292-2; la etiqueta «hipótesis» queda **SUPERADA**: el falso positivo está **MEDIDO** por el PREP, casos fp1–fp4 y fp7; B-292-2 **sube de P2 a P1**) · [ADR-045](045-e0217-int-overflow.md) (E0217) · [ADR-044](044-e0216-int-div0.md) (E0216) · [ADR-290](290-core-index-mut-v0.md) (B-290-2) · [ADR-293](293-core-undeclared-call-v0.md) (orden: 293 → 294) · PREP Parser
- **Cierra (al CLOSED):** **B-292-2** (P1, HIR): falsos positivos de E0217/E0216 por `known_int` no sensible al flujo.
- **Código:** **ninguno nuevo.** E0217 (`integer overflow`) y E0216 (`integer division by zero`) conservan texto y significado; solo se eliminan falsos positivos.
- **No reabre:** el texto de E0216/E0217 · ADR-292 (CLOSED; se precisa por addendum) · `check_body` y el arm `Let` (reserva para S2 MUST-USE) · Parser y Codegen (no cambian) · los falsos negativos fn1–fn4 (B-294-2).

## 1. Decision and limits

**D1 — Option A (minimal).** Invalidate `known_int` at **branch boundaries** `if`/`match` (and `if let`/`while let`), **loop** boundaries, and **block scope**. No flow analysis: option B (`known` map per path, union by intersection, fixed point in loops, scope `pop`) is **rejected for v1** (PREP §7: +120–200 LOC, 15–20 tests, touches `check_body` and all arms).

**D2 — ONE single helper inside `check_stmt`.** Description in PREP terms (§7, option A):
1. `clobbered(body)`: walks nested bodies (template: `stmt_uses_ident` / `body_uses_ident`) and returns the names with `Assign` o `Let` at **any depth**.
2. Before **each sibling branch** (`then`/`else`, each `match` arm): restore the prior `known_int` **snapshot** from before the statement for those names (`Vec<(String, Option<i64>)>`).
3. Before a **loop body**: `known_int = None` for those names (“2nd iteration” effect).
4. **After the statement**: `known_int = None` for those names (conservative merge).
5. **`match` coverage (Q4, Engineer 2026-10-04: YES):** the helper covers `match` with pattern binding (Result/Option/enum) and **all** call sites of the `Match` arm, including the enum arm that calls `check_stmt` per statement (HIR ~L4127; sites L3925, L4006, L4084, and L4127). Restoring `known_int` neither replaces nor modifies the existing pattern-binding restore; IMPL fixes the exact order and at least one unit test `adr294_match_pattern_binding_*` locks it (§3).

**D3 — What is NOT touched.** `check_body` and the `Let` arm (including `bindings.insert`) are **not modified**: their sites are the S2 MUST-USE hooks (`Let` arm after `bindings.insert`, `Expr` arm, and `check_body` tail; PREP §8). Option A edits only the control-flow arms of `check_stmt` (`If`, `IfLet`, `While`, `WhileLet`, `Match`) and the helper.

**D4 — No new E-code.** E0217 remains valid **on straight-line code**: c5 (`let mut a = MAX; a + 1`) and c6 (`a = a + 1`) keep E0217. Any control statement that does **not** assign or `let`-bind the name (name not in `clobbered`) also keeps it.

**D5 — False negatives fn1–fn4 are NOT fixed.** They still lack E0217 and the overflow already **panic 101**s at runtime per ADR-292 (D1). They remain as HIR control tests (“still Ok”) and as backlog B-294-2.

**D6 — IMPL order and scope.** `arita-hir` only. IMPL order on Lex: **ADR-293 → ADR-294 → ADR-291 → S2 MUST-USE** (Engineer). No Codegen or Parser changes: not one byte of emitted Rust changes for programs that compile today.

## 2. Evidence (PREP + code)

- **Mecanismo** (PREP §3, verified by reading `crates/arita-hir/src/lib.rs`, sha256 `d0571e12419dae491ec7ce5dcfba99ad247c045793fc3c1f6c53abf1ee575807`, 9630 lines, unchanged since PREP): `bindings` is a single flat per-function `HashMap`; `check_body` (L4141–L4146) is `for s in body { check_stmt(s)? }` with no scope open/close; `If` (L3696–L3711) checks `then` and `else` on the **same** map; loops are checked **once**, in source order; an inner `let` does `bindings.insert` and is **not** removed on block exit; only `if let`/`while let`/`match` pattern bindings are restored (L3739–L3750, L3802–L3812, `match` arms). Straight **assignment** is correct (`Assign`, L3838–L3855: literal ⇒ `Some(n)`, other RHS ⇒ `None`). The defect is in **path merge**, **loop**, and **scope**.
- **Falso positivo MEDIDO** (PREP §4.1; `arita parse` with the already-built `target/debug/arita` binary, not rebuilt, not verified by source hash): fp1, fp2, fp3, fp4 ⇒ `E0217: integer overflow`; fp7 ⇒ `E0216: integer division by zero`; all five are valid programs with no execution path that overflows or divides by 0. Also fpw5 and fpw6 ⇒ `E0217` (**path-dependent**: with `n = 3` fpw5 overflows at runtime; fpw6 overflows after two iterations; see §4 and Q2).
- **Falsos negativos medidos** (PREP §4.2): fn1–fn4 yield `ok` from `arita parse` and overflow at runtime (panic 101 [L] per ADR-292).
- **Controls** (PREP §4.3): c1–c4 `ok` today; c5 and c6 `E0217` today. The 17 files under `<scratch>/arita_b292/cases/` are **identical, block for block**, to the PREP sources (checked today).
- **Corpus** (PREP §5, lexical, not run today): 867 `.arita`, 16 `ident = …` assignments in 9 files, **0** with integer-literal RHS; 0 nested `let x: Int = <lit>` shadows over an outer `x`; **0 affected positives, 0 false positives in the corpus**. v1 risk per PREP: **MEDIUM** (probability 0/867; severity: valid program rejected with no escape short of rewrite).
- **Todos los consumidores de `fold_i64`** are polluted by the same defect: 39 lines / 42 occurrences of `self.fold_i64` (re-verified today), including E0217 `+ - *` (L724–L739), E0217 MIN÷−1 (L3315, L3339) and E0216 (e.g. L2832). So the fix is in `known_int`, not in E0217.
- **Code references verified by reading:** `BindingState` L375–L380 (`known_int` L380); `fold_i64` L455–L463; E0217 L724–L739; `Let` L3594, `known_int` compute L3652–L3655 and `bindings.insert` L3666–L3680; `Expr` L3683; `If` L3696–L3711; `IfLet` L3713–L3757; `WhileLet` L3759–L3818; `While` L3820–L3830; `Assign` L3838–L3855; `Match` L3876–L4137; `check_body` L4141–L4146; `stmt_uses_ident` L4363; `body_uses_ident` L4419; inserts with `known_int: None` at L3739, L3802, L3912, L3993 and L5078. Call sites of `check_body`/`check_stmt` inside control arms: L3707, L3709, L3748, L3755, L3811, L3828, L3925, L4006, L4084 and L4127 (the enum `match` arm checks per statement with `check_stmt`).

## 3. Planned minimal change (to verify in IMPL)

- **`arita-hir`, `check_stmt`:** the D2 helper and its use in the `If`, `IfLet`, `While`, `WhileLet`, `Match` arms (the three `Match` scrutinee kinds: Result/Option, Bool/Int and enum). **PREP estimate, not measured:** +45–60 production lines.
- **HIR unit tests (outside Measure oracles):** ~10 `adr294_*` tests (PREP called them `adr292b_*`; **(corrected)** to this ADR’s id), ~150 LOC per PREP, with sources in `const` **outside** `#[test]` (B-286-5):
  - fp1, fp2, fp3, fp4, fp7, fpw5, fpw6 ⇒ `Ok` (fpw5 and fpw6 are **unit-only**, not corpus: Q2);
  - c1–c6 as controls (c1–c4 `Ok`, **unit-only**, not promoted to oracles: Q3; c5 and c6 `E0217`);
  - fn1–fn4 ⇒ “stays Ok” (controls for the known false negative);
  - **at least one** `adr294_match_pattern_binding_*` (Q4): `match` with pattern binding (Result/Option/enum) and enum arm with `check_stmt` per statement, checking that an `Assign` in one arm neither contaminates a sibling nor survives after the `match`.
  
  The exact split across test functions (17 sources, ~10 tests) is decided by IMPL. **0 existing tests modified** (PREP §7: the E0217 HIR tests `e0217_*` L6121–L6236 are straight-line; not HIR test builds `HirStmt::Assign`; `adr283_eq_set_negative_lit_and_known_int_e0319` L8579 is straight-line).
- **Measure:** register the 7 §4 oracles: fp1, fp2, fp3, fp4 and fp7 (new fixtures under `ejemplos/core10/known-int/`), c6 (**new** fixture under `ejemplos/core10/known-int/neg/`) and c5, which **REUSES** the existing fixture `ejemplos/f2/neg/e0217-runtime-max-plus.arita` without modifying or duplicating it (only wired in measure, same E0217 contract, exit 1); Measure/Engineer does it.
- **No changes:** `arita-syntax`, Codegen, `arita-cli` (except oracle registration), `package.rs`.
- **Post-gate validation (PREP §9.4, not executed):** `cargo test -p arita-hir adr294`, `cargo test -p arita-hir`, `cargo clippy -p arita-hir --all-targets -- -D warnings`, `cargo fmt --check`, and `arita parse` of the cases expecting Ok on fp1–fp4, fp7, fpw5, fpw6, c1–c4 and `E0217` on c5 and c6.

## 4. Oracles (k = 7)

**Ids and paths accepted by the Engineer (Q1, 2026-10-04):** positives under `ejemplos/core10/known-int/`, c6 under `ejemplos/core10/known-int/neg/`, c5 reuses an existing fixture (see below). The **file names** inside those directories are those of the PREP cases (`fp1-else-bleed.arita`, …) and IMPL confirms them. Sources are those of the PREP cases (`module b292`; the module name may change when creating the fixtures). **PREP only validated with `arita parse`: that these positives compile and run with rustc/cargo is [NOT MEASURED]** (risk in §6).

**Positives (must COMPILE and run; exit 0).** Exact stdout derived from the source (`print` writes one line per call), **[not executed]**:

1. `core10-known-int-fp1-else-bleed` — case `fp1-else-bleed`. `n = 3` ⇒ only the `then` runs. stdout: `ok`.
2. `core10-known-int-fp2-match-arm` — case `fp2-match-arm-bleed`. `flag = true` ⇒ only the `true` arm. stdout: `ok`.
3. `core10-known-int-fp3-loop-carried` — caso `fp3-loop-carried`. En la vuelta `i == 1`, `a` vale 0 ⇒ `r = 1`. stdout: `1` / `ok`.
4. `core10-known-int-fp4-shadow-leak` — caso `fp4-shadow-leak`. Imprime la sombra interior, luego `a + 1` con `a = 0`. stdout: `9223372036854775807` / `1` / `ok`.
5. `core10-known-int-fp7-e0216-else` — case `fp7-e0216-else-bleed`. `n = 3` ⇒ only the `then` (`d = 0`); the `else` with `x.div_euclid(d)` does not run. stdout: `ok`.

**Straight-line control negs (keep their error).** Each neg: exact text `E0217: integer overflow`, exit 1, empty stdout, no Rust emission:

6. `neg-core10-known-int-c5-straight` — **REUSES the existing fixture `ejemplos/f2/neg/e0217-runtime-max-plus.arita`** (`let a: Int = MAX; let x: Int = a + 1`; sha256 `34c6bf8f63673596b7b323721aa2406e11e330c2d370577e962887b3bf9a27b5`, verified today), **without modifying or duplicating it**; today it is not wired into measure (its own comment says so) and this ADR only wires it, with the same E0217 contract and exit 1. Equivalent to the PREP case `c5-true-positive` (the PREP source differs in `print(r)`; same program where it matters). Its sha is recorded in the new freeze (it already appears in earlier freezes, e.g. `MEASURE-ADR292-INT-ARITH-RUNTIME-FREEZE-20261003.sha:957`; none are rewritten).
7. `neg-core10-known-int-c6-assign-overflow` — caso `c6-assign-overflow-expr` (`a = a + 1` con `a = MAX`); fixture **NUEVO** en `ejemplos/core10/known-int/neg/`.

**k = 7 CONFIRMED (Engineer, 2026-10-04)**: the exact number of measurable oracles listed above (5 positives + 2 negs). Controls c1–c4 are **not** oracles (Q3: they are unit tests); fpw5 and fpw6 are **not** oracles (Q2: unit tests). **Queue N — Engineer figures (2026-10-04), quoted as such:** 866 → 871 (ADR-293, k = 5) → **878 (this ADR, k = 7)** → 882 (ADR-291, k = 4) → +k(S2); order 293 → 294 → 291 → S2; the Engineer recalculates at GO. Rules: skip ≠ PASS; no oracle counts if the build did not run. The freeze and addendum are written by the Engineer; no existing freeze is rewritten.

## 5. Backlog

- **B-292-2 (P2 → P1):** closes at this ADR’s CLOSED (documentary close with addendum on ADR-292 by the Engineer).
- **B-294-1 (P2, propuesta):** inherited `ty`/`mutable`/`moved` shadow via an inner `let`. Option A fixes `known_int` but an inner `let` still permanently replaces the outer’s whole `BindingState` (PREP §3.5 and §7). `moved` is a **read suspicion, not measured** [H].
- **B-294-2 (P3, propuesta):** fn1–fn4 — flow-sensitive E0217 (option B) to catch path-reachable overflow at compile time; today ADR-292’s panic 101 covers it.
- **B-292-1 and the rest of the backlog of ADR-292:** unchanged.

## 6. Risks

1. **Loss of detection (expected):** after a statement that assigns or `let`-binds a name in a branch, the name ceases to be foldable; E0217/E0216 stop firing in “definite” cases that only arose by path (fpw5, fpw6 no longer yield E0217; PREP §7). Consistent with ADR-045 (“same-scope”) and covered at runtime by ADR-292.
2. **Collision with S2 MUST-USE:** mitigated by D3 (do not touch `check_body` or `Let`); line-number drift is expected, not a context conflict (PREP §8).
3. **`Match` with several forms (Q4 resolved: YES):** the `Match` arm has several checking forms (Result/Option, Bool/Int, enum) with four separate call sites (L3925, L4006, L4084, L4127); the helper must cover them all, including pattern binding and the enum arm per statement; at least one test `adr294_match_pattern_binding_*` locks it.
4. **Compilation of the positives [H, not measured]:** PREP only ran `arita parse`. The Rust emitted for fp1–fp4 and fp7 could produce rustc warnings (e.g. `unused_assignments`) or trigger the default lint `arithmetic_overflow`/`unconditional_panic` on values propagated as constants; if Measure compiles without `-D warnings` only the latter matters. Measure PREP must resolve this before GO.
5. **The PREP binary:** `[M]` measurements use an existing `target/debug/arita` whose exact correspondence with the current HIR was not verified by hash (mtime is compatible).
6. **Corpus:** 0/867 affected (lexical, PREP §5); corpus regression risk is measured in IMPL with measure and `cargo test`.
7. **Closures / `spawn` with a body (Q5):** they do not exist today (`rg -i 'closure|lambda'` finds no matches in `arita-hir` or `arita.pest`; `spawn` takes a call, not a body). **Note for the ADR that introduces them:** it must extend `clobbered` to those bodies; otherwise an `Assign` inside them would not invalidate `known_int` and the defect this ADR fixes would reappear.

## 7. GO, order, and measurement

- Measure PREP without BLOCKER; implementation in `arita-hir` (+ Measure oracles and fixtures); Codegen and Parser unchanged.
- **GO IMPL** lo da el Ingeniero. **Orden:** ADR-293 → ADR-294 → ADR-291 → S2 MUST-USE; una carga pesada a la vez. **(corregido)** El PREP (§9.2) recomendaba el slice «tras ADR-291»; prevalece el orden del Ingeniero.
- Measurement with real evidence (exclusive measure, `cargo test`, fmt, clippy, Veyra); forbidden to invent PASS / N/N / CLOSED. Close with own GATE `DOC/GATE-CORE10-KNOWN-INT-SCOPE-<fecha>.md`.

## 8. Engineer questions — RESOLVED (2026-10-04)

- **Q1 — RESUELTA:** ids and paths accepted: positives `core10-known-int-fp1-else-bleed`, `-fp2-match-arm`, `-fp3-loop-carried`, `-fp4-shadow-leak` and `-fp7-e0216-else` under `ejemplos/core10/known-int/`; negs c5 and c6. **c5 REUSES** the existing fixture `ejemplos/f2/neg/e0217-runtime-max-plus.arita` without modifying or duplicating it (only wired in measure; same E0217 contract, exit 1; its sha is recorded in the freeze); **c6 NEW fixture** under `ejemplos/core10/known-int/neg/` (§4).
- **Q2 — RESUELTA:** fpw5 y fpw6 son **tests unitarios**, no corpus.
- **Q3 — RESUELTA:** c1–c4 are **NOT** promoted: unit tests.
- **Q4 — RESOLVED (YES):** the helper covers `match` with pattern binding (Result/Option/enum) and the enum arm that calls `check_stmt` per statement (HIR ~L4127); at least one unit test `adr294_match_pattern_binding_*` (D2.5, §3).
- **Q5 — RESUELTA:** no closures/`spawn` with a body today; risk note left for the ADR that introduces them (§6.7).
- No open questions.

## 9. Changelog

- 2026-10-04 — v0.1 DRAFT PINS (Architect): Engineer decisions 2026-10-04: option A (invalidate `known_int` at branch/loop/block boundaries, one single helper in `check_stmt`, do not touch `check_body` or `Let`); no new E-code; fn1–fn4 to backlog (B-294-2) and B-294-1 proposed; B-292-2 P1 (MEASURED false positive, ADR-292 D5 “hypothesis” label superseded); k = 7 proposed (5 positives + 2 negs), N = 878 provisional (871 + 7), Engineer recalculates; order 293 → 294 → 291 → S2; no GO IMPL.
- 2026-10-04 — v0.2 DRAFT PINS (Architect): Engineer answers Q1–Q5 incorporated: Q1 ids/paths accepted (positives under `ejemplos/core10/known-int/`; c5 REUSES `ejemplos/f2/neg/e0217-runtime-max-plus.arita` without modifying or duplicating, c6 new fixture under `ejemplos/core10/known-int/neg/`); Q2 fpw5/fpw6 unit-only; Q3 c1–c4 unit-only; Q4 the helper covers `match` with pattern binding and the enum arm per statement (D2.5; test `adr294_match_pattern_binding_*`); Q5 risk note for closures/`spawn` (§6.7); k = 7 confirmed; Queue N (Engineer figures): 866 → 871 (293) → 878 (294) → 882 (291) → +k(S2). Engineer Seal pending (not added here); no GO IMPL.

## Engineer Seal (2026-10-04)

Sealed on version v0.2 with sha256 `33c10f540994005633857b1899934ca9d5f2b86359c7815dc104407fb900d64c` (109 lines). This section is appended at the end; nothing above has been modified.

**Decision: v0.2 ACCEPTED as the pins contract for ADR-294. No GO IMPL yet.** GO IMPL is given in writing after ADR-293 CLOSED and Measure PREP without BLOCKER (order 293 → 294 → 291 → S2; one heavy load at a time on Lex).

1. **Scope (option A) confirmed.** One single helper inside `check_stmt`, used only in the `If`, `IfLet`, `While`, `WhileLet`, and `Match` arms. `check_body`, the `Let` arm, and `Assign` are not touched. No new E-code. Option B remains rejected for v1.
2. **Oracles k = 7 confirmed**: five positives (fp1, fp2, fp3, fp4, fp7) under `ejemplos/core10/known-int/` and two negs with exact E0217 and exit 1 (c5 reuses `ejemplos/f2/neg/e0217-runtime-max-plus.arita` without modifying or duplicating it; c6 is a new fixture under `ejemplos/core10/known-int/neg/`). Queue figures: 866 → 871 (293) → 878 (this ADR) → 882 (291) → +k(S2).
3. **Acceptance condition for the positives (risk §6.4).** Before counting a positive as an oracle, Measure must compile and run the emitted Rust with rustc and fix the real stdout; hand-derived stdout from §4 is not evidence. If rustc rejects or warns (e.g. `arithmetic_overflow` or `unconditional_panic` on values propagated as constants), that is a BLOCKER that returns to the Architect. The contract is not relaxed and the case is not removed from the set without my decision in writing.
4. **`match` coverage.** The unit test `adr294_match_pattern_binding_*` is mandatory and must cover the four call sites of the `Match` arm (Result/Option, Bool/Int, and the enum arm that calls `check_stmt` per statement). An `Assign` in one arm must neither contaminate a sibling nor survive after the `match`.
5. **Unit tests without touching anything existing.** fpw5, fpw6, c1–c4, and fn1–fn4 remain unit tests only. Zero existing tests modified; sources go in `const` outside `#[test]`. The patch must pass `cargo fmt --check` and `cargo clippy -- -D warnings` before being applied to the real tree.
6. **Parser.** The `/tmp` validation diff is redone on this v0.2 (k = 7). Only the final diff that Parser delivers with its sha256 counts, validated in the 293 → 294 → 291 chain on a clean copy.
7. **Close.** The gate requires fmt 0, clippy `-D warnings` 0, release build, `cargo test --workspace -- --test-threads=1`, measure N/N with no skip, Veyra ACCEPTED, and own GATE `DOC/GATE-CORE10-KNOWN-INT-SCOPE-<fecha>.md`. I write the freeze and addendum; no prior freeze or seal is rewritten. The documentary close of B-292-2 is done with an addendum in ADR-292 by me, without editing its Seal.
8. **Backlog confirmed.** B-294-1 (P3: inherited shadow of `ty`/`mutable`/`moved`; `moved` is an unmeasured suspicion) and B-294-2 (P3: false negatives fn1–fn4, covered at runtime by ADR-292 panic 101). B-292-2 rises to P1 and closes with this ADR.
9. **Note for the future.** If closures or `spawn` with a body are introduced, their ADR must extend `clobbered` to those bodies (§6.7).
