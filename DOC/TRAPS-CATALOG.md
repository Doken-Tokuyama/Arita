# ARITA — Trap catalog (measure oracles)

- **Estado:** **draft / v0** (+ E0214 vacuous len — CUT `TRAPS-LEN-THEATER-20260914`; + E0241 neg wire; + E0225 vacuous match — CUT `TRAPS-MATCH-VACUOUS-20260914`; + E0215 vacuous cmp — CUT `TRAPS-CMP-THEATER-20260915`; + E0242 borrow×await — CUT `TRAPS-BORROW-AWAIT-20260915`; + E0226 while false — CUT `TRAPS-WHILE-FALSE-20260915`; + E0227 if false — CUT `TRAPS-IF-FALSE-20260915`; + E0216 int div0 — CUT `TRAPS-INT-DIV0-20260915`; + E0217 int overflow — CUT `TRAPS-INT-OVERFLOW-20260915`; status DOC ADR-043 `CORPUS-STATUS-20260915` + ADR-046 `CORPUS-ARITH-20260915`)
- **Fecha:** 2026-09-14
- **SoT:** `<repo>` (Lex); companion to `arita measure` + `DOC/THREAT_MODEL.md`
- **Framing:** truth-gate / measure companion. Maps AI→Rust failure modes that existing **measure oracles already block**. Not an antivirus product; project name = **ARITA**.
- **Rule:** every **Covered now** row cites a wired oracle id + path under `ejemplos/` (or a required workspace oracle in `crates/arita-cli/src/measure.rs`). No unverified claims.
- **Roadmap:** starts Fase 4 item “Corpus trampas ARITA” — catalog v0 landed; ports from Investigación Rust-IA still pending.

## How to read

| Column | Meaning |
|--------|---------|
| Trap | Short name for the AI failure mode |
| Why AIs do it | Typical generative shortcut when targeting Rust-shaped code |
| ARITA defense | Stable `E0xxx` / engine code + measure oracle id |
| Path | `.arita` / contract / CLI fixture used by the oracle |

Measure rule (anti-theater): **accepted** for a negative oracle iff the pipeline fails with the **expected** code; accidental success or wrong code → **rejected**; missing file / missing toolchain → **inconclusive**. `skip ≠ PASS`. Never `inconclusive` → `accepted`.

---

## Covered now

Inventory source: `NEG_ORACLES` + CLI/deps/perf neg runners in `crates/arita-cli/src/measure.rs`, plus notable **expect-fail / reject** companions (F3 `expect_sat=false`, contract `expect_reject`, attest bad-hash, required clippy/miri). Positive E2E stdout oracles (hello, F2 `01`–`09`, etc.) are out of scope unless they are explicitly anti-theater gates.

### A. Anti-theater surface (E021x) — ADR-010

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 1 | `todo!` / `unimplemented!` stub | Leave unfinished bodies that “compile later” | **E0210** · `neg-e0210-todo` | `ejemplos/f2/neg/e0210-todo.arita` |
| 2 | Tautological `assert true` | Fake green tests without evidence | **E0211** · `neg-e0211-assert-true` | `ejemplos/f2/neg/e0211-assert-true.arita` |
| 3 | Empty `test` block | Claim coverage with a hollow test | **E0212** · `neg-e0212-empty-test` | `ejemplos/f2/neg/e0212-empty-test.arita` |
| 4 | Stub valued `fn` (empty body) | Sketch API shape without implementation | **E0213** · `neg-e0213-empty-fn` | `ejemplos/f2/neg/e0213-empty-fn.arita` |
| 4b | Vacuous `len` / `is_empty` theater | `assert v.len() >= 0`, `len()==len()`, `is_empty()||!is_empty()` | **E0214** · `neg-e0214-*` | `ejemplos/f2/neg/e0214-len-ge-zero.arita`, `e0214-len-eq-len.arita`, `e0214-is-empty-taut.arita` |
| 4c | Vacuous comparison assert | `assert n == n` / `n <= n` / lit==lit | **E0215** · `neg-e0215-*` | `ejemplos/f2/neg/e0215-eq-self.arita`, `e0215-le-self.arita` |
| 4d | Unchecked Int `/` `%` by zero | Lit `/0` or `div(x,0)` → rustc/panic theater | **E0216** · `neg-e0216-div0` | `ejemplos/f2/neg/e0216-div0.arita` (+ lit companions; ADR-044 **verified** Lex **84**+2) |
| 4e | Unchecked Int `+/-/*` overflow | Lit MAX+1; release silent wrap | **E0217** · `neg-e0217-add/sub/mul-overflow` | `ejemplos/f2/neg/e0217-*.arita`; CUT `E0217-ORACLES-SUBMUL-20260915` **DONE** |

### B. Ownership / borrow (E020x) — ADR-009

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 5 | Use after move | Copy Rust patterns without tracking moves (`String`/`Vec`) | **E0201** · `neg-e0201-use-after-move` | `ejemplos/f2/neg/e0201-use-after-move.arita` |
| 6 | Double `&mut` / borrow conflict | Parallel mut borrows from Python/JS mental models | **E0202** · `neg-e0202-double-mut` | `ejemplos/f2/neg/e0202-double-mut.arita` |

### C. Std whitelist (E0206) — ADR-026

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 7 | Invented / out-of-whitelist method | Hallucinate `Int.len()` or free-form Rust methods | **E0206** · `neg-e0206-bad-method` | `ejemplos/f2/neg/e0206-bad-method.arita` |

### D. Control flow / match (E022x) — ADR-014 / 015 / 016

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 8 | Non-Bool `if`/`while` condition | Use Int/truthiness like C/Python | **E0220** · `neg-e0220-bad-cond` | `ejemplos/f2.1/neg/e0220-bad-cond.arita` |
| 9 | Non-exhaustive Bool `match` | Partial arms; forget `false` / `_` | **E0221** · `neg-e0221-bool-nonex` | `ejemplos/f2.2/neg/e0221-bool-nonex.arita` |
| 10 | Int `match` without `_` | Finite lit arms only; miss open Int domain | **E0222** · `neg-e0222-int-no-wild` | `ejemplos/f2.2/neg/e0222-int-no-wild.arita` |
| 11 | Pattern / scrutinee type mismatch | Mix Bool patterns on Int (and similar) | **E0223** · `neg-e0223-pat-mismatch` | `ejemplos/f2.2/neg/e0223-pat-mismatch.arita` |
| 12 | `break`/`continue` outside `while` | Drop loop keywords into `if` / bare blocks | **E0224** · `neg-e0224-break-outside` | `ejemplos/f2.3/neg/e0224-break-outside.arita` |
| 12b | Vacuous `match` arms (same constant) | Exhaustive morph: all arms `{ lit }` identical | **E0225** · `neg-e0225-match-*-same` | `ejemplos/f2.2/neg/e0225-match-bool-same.arita`, `e0225-match-int-same.arita` |
| 12c | Vacuous `while false` | Unreachable loop body theater; print after still “passes” | **E0226** · `neg-e0226-while-false` | `ejemplos/f2.1/neg/e0226-while-false.arita` (ADR-040; **verified**) |
| 12d | Vacuous `if false` | Unreachable then; later print theater | **E0227** · `neg-e0227-if-false` | `ejemplos/f2.1/neg/e0227-if-false.arita` (ADR-041; **verified**) |

### E. Safe-only / async / deps / perf CLI

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 13 | `unsafe` / FFI in surface | Escape to Rust power tools for “performance” | **E0231** · `neg-e0231-unsafe` | `ejemplos/f2/neg/e0231-unsafe.arita` |
| 14 | `await` outside `async fn` | Call async helpers from sync `main`/`fn` | **E0240** · `neg-e0240-await-outside` | `ejemplos/async/neg/e0240-await-outside.arita` |
| 14b | Illegal `async` outside `async fn` | Put `async` on stmts/items that are not `async fn` | **E0241** · `neg-e0241-async-illegal` | `ejemplos/async/neg/e0241-async-illegal.arita` |
| 14c | Borrow held across `await` | Live `&`/`&mut` spans `await` (was HIR hole → rustc E0100) | **E0242** · `neg-e0242-borrow-across-await` | `ejemplos/async/neg/e0242-borrow-across-await.arita` (ADR-039; **verified**) |
| 15 | Unknown CLI `--profile` | Invent profile names; skip real release evidence | **E0250** · `perf-neg-e0250` | `ejemplos/perf/neg/e0250-bad-profile.arita` (CLI `--profile fantasma`) |
| 16 | Unknown `[deps]` crate | Pull arbitrary crates.io names | **E0260** · `deps-neg-e0260` | `ejemplos/deps/neg/e0260-unknown-crate/main.arita` (+ sibling `arita.toml`) |
| 17 | External crate path in surface | Write `tokio::…` as free Rust | **E0261** · `deps-neg-e0261` | `ejemplos/deps/neg/e0261-crate-path.arita` |

### F. Logic island expect-fail (E03xx) — ADR-012

Measure **accepted** iff the logic engine rejects with the listed code (`expect_sat = false`). Accidental SAT → **rejected**.

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 18 | Claim unsat path as proven | Assert reachability that facts do not support | **E0301** · `f3-02-path-fail` | `ejemplos/f3/02-path-fail.arita` |
| 19 | Invented sibling / wrong join | Hallucinate relational closure | **E0301** · `f3-04-sibling-fail` | `ejemplos/f3/04-sibling-fail.arita` |
| 20 | Unknown predicate | Freestyle predicate names | **E0303** · `f3-06-unknown-pred` | `ejemplos/f3/06-unknown-pred.arita` |
| 21 | Forbidden fn in logic island | Smuggle host-style calls into Datalog surface | **E0304** · `f3-07-forbidden-fn` | `ejemplos/f3/07-forbidden-fn.arita` |
| 22 | Predicate arity mismatch | Wrong arity from incomplete few-shot | **E0303** · `f3-08-arity-mismatch` | `ejemplos/f3/08-arity-mismatch.arita` |
| 23 | Partial-PASS multi-query | Claim module OK when one query sat and another failed | **E0301** · `f3-11-multi-query-fail` | `ejemplos/f3/11-multi-query-fail.arita`; ADR-099 **verified** Logic **12/12** |
| 24 | Join-miss as proven | Hallucinate join closure across wrong dept/relation | **E0301** · `f3-12-join-miss-fail` | `ejemplos/f3/12-join-miss-fail.arita`; ADR-099 **verified** Logic **12/12** |

### G. Contract / attest reject companions — ADR-017 / 018 / 019

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 23 | Contract ignores reject obligation | Ship a “contract” that only expects happy stdout | `expect_reject` **E0224** · `contract-neg-e0224` | `ejemplos/contracts/contract-neg-e0224.json` |
| 24 | Language `contract` without reject check | Same pattern in-source | `expect_reject` **E0224** · `lang-contract-neg-e0224` | `ejemplos/contracts/lang-neg-e0224.arita` |
| 25 | Pinned hash theater / wrong attestation | Claim attested source while bytes differ | attest mismatch · `contract-hello-bad-hash` (measure accepted **iff** reject) | `ejemplos/contracts/contract-hello-bad-hash.json` |

### H. Required workspace gates (skip ≠ PASS)

Not `.arita` traps; still anti-theater measure oracles — missing toolchain must never become overall `accepted`.

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 26 | Skip Clippy / treat missing as PASS | “Green” without `-D warnings` on workspace crates | `clippy-workspace` (missing → **inconclusive**) | `crates/arita-cli/src/measure.rs` → `run_clippy_oracle` |
| 27 | Skip Miri / treat missing as PASS | Claim UB-free without running Miri | `miri-workspace` (missing → **inconclusive**) | `crates/arita-cli/src/measure.rs` → `run_miri_oracle` |

### I. Release-profile evidence companion — ADR-028

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 28 | Fake “release” without opt-level evidence | Print release-looking stdout without real profile emit | `perf-02-release-optlevel` (requires `[profile.release] opt-level = 3` in generated `Cargo.toml`) | `ejemplos/perf/01-release-run.arita` (paired with release build) |

**Covered count (this catalog v0): 38** traps with wired measure defenses (+ E0217; + ADR-099 f3-11/12).

---

## Candidates next

Stubs only. **PENDING** — do not treat as measure coverage. No oracle invented here.

| Stub | Notes | Status |
|------|-------|--------|
| E0217 sub/mul measure oracles | ADR-045 addendum `E0217-ORACLES-SUBMUL-20260915` (no ADR-047); Lex **87**+2 | **DONE** (moved to Covered) |
| E0217 integer overflow | ADR-045 **verified** Lex **87**+2 (`TRAPS-INT-OVERFLOW` + `E0217-ORACLES-SUBMUL`); add/sub/mul | **DONE** (moved to Covered) |
| Mutex×await theater | Needs `Mutex` (+ async interaction) in surface — **PARK** until CUT surface | **PARK** (ADR-038) |
| Negative string repeat | ADR-075 / **E0280** `negative repeat count`; CUT `STD-REPEAT-20260918` **verified** Lex **172/172** (+ Vec ADR-076) | **DONE**/GO |
| Invalid clamp range | ADR-072 / **E0279** `invalid clamp range`; CUT `STD-CLAMP-20260918` **verified** Lex **162/162** | **DONE** |
| Int abs overflow MIN | ADR-069 / **E0278** `integer abs overflow`; CUT `STD-ABS-20260918` **verified** Lex **153/153** | **DONE** |
| Negative pow exponent | ADR-079 / **E0281**/E0283**; CUT `STD-POW-20260918` **verified** Lex **184/184** | **DONE** |
| Vacuous while-let Result | ADR-078 / **E0282** `vacuous while-let result`; CUT `TRAPS-WHILE-LET-RESULT-VACUOUS-20260918` **verified** Lex **180/180** | **DONE** |
| Vacuous while-let None | ADR-056 / **E0277** `vacuous while-let none`; CUT `TRAPS-WHILE-LET-VACUOUS-20260918` **IMPL GO** | **DONE** Lex **118/118** |
| Option swallow / ignore-None | ADR-051 / **E0274** `option none swallowed`; CUT `TRAPS-OPTION-SWALLOW-20260918` **IMPL GO** | **GO IMPL** |
| Partial-PASS multi-query / join-miss | ADR-099 / **E0301**; CUT `F3-UNSAT-FAIL-NEXT-20260918`; Logic `f3_` **12/12** | **DONE** |
| Result swallow / ignore-Err | ADR-048 / **E0272** `result error swallowed`; CUT `TRAPS-RESULT-SWALLOW-20260918` **verified** Lex **96/96** | **DONE** |
| Port corpus from Investigación Rust-IA | ROADMAP Fase 4 ports still open; oleada + arith **ticked** vía ADR-043/046 DOC; map Investigación names only after ARITA oracle; morphs/Mutex **PARK**; next = ADR-047 Result IMPL (#58); Mutex/swallow PARK (#59); HOLE no-control standby | **PENDING** |
| Timed perf thresholds / mutation oracles | Explicitly OUT of PERF-V0; would be new oracles, not inventory | **PENDING** (product decision first) |

---

## Non-goals (v0)

- Do **not** change crates or measure behavior for this DOC.
- Do **not** call this catalog antivirus / malware detection.
- Do **not** list positive stdout-only smoke tests as “traps” unless they are reject/expect-fail gates.
- Do **not** claim any code as covered without a measure oracle row (E0241 now wired via `neg-e0241-async-illegal`).

## Verify

```bash
# From repo root — negatives must fail with the listed code:
cargo run -p arita-cli -- parse ejemplos/f2/neg/e0210-todo.arita
cargo run -p arita-cli -- measure   # includes NEG_ORACLES + companions; skip ≠ PASS
```

## Related

- `DOC/THREAT_MODEL.md` — anti-theater threat model
- `DOC/ADR/010-e021x-negative-oracles.md` — E021x
- `ejemplos/README.md` — full oracle tables
- `crates/arita-cli/src/measure.rs` — SoT of wired ids
