Translation of `291-core-mutex-reject-v0.md`; the original is normative. / Traducción de `291-core-mutex-reject-v0.md`; el original es el normativo.

# ADR-291 — Core Mutex-reject v0: `Mutex`/`Arc`/`.lock()` outside the surface (Core 0.10)

- **Estado:** **DRAFT PINS v0.2 (Arquitecto, 2026-10-04; decisiones del Ingeniero incorporadas; pendiente GO; v0.1 pedido tras PREP de B-286-7 → superado por ADR-293).** ADR-290 está CLOSED (GATE `DOC/GATE-CORE10-INDEX-MUT-20261003.md`) y ADR-292 CLOSED (GATE `DOC/GATE-CORE10-INT-ARITH-RUNTIME-20261003.md`); la verificación y el GO de este ADR los da el Ingeniero. Este texto solo fija una propuesta comprobable; no declara PASS, N/N ni CLOSED. **Orden de la cola (Ingeniero, 2026-10-04): ADR-293 → ADR-294 → ADR-291 → S2 MUST-USE.**
- **CUT-ID:** `CORE-0.10-MUTEX-REJECT-20261003` (provisional; lo confirma el Orquestador)
- **Fecha:** 2026-10-03
- **Autores:** **ARITA Arquitecto**; alcance y orden pendientes de confirmación del Ingeniero
- **Padre / contexto:** [ADR-229](229-lang-contract-mutex.md) (Mutex PARK/E0312), [ADR-286](286-core-0.10-errores-fase1.md) (B-286-7 y B-286-12), [ADR-039](039-e0242-borrow-across-await.md) (E0242), [ADR-290](290-core-index-mut-v0.md) (CLOSED), [ADR-293](293-core-undeclared-call-v0.md) (B-286-7 y la migración del neg `neg-core03-compose-mutex-hold`, E0347; va ANTES que este ADR), Parser IMPL r2 (not in the public export / no incluido en el export público) (parche del Parser validado en copia; §2.5)
- **Cierra (al CLOSED):** el hueco de diagnóstico propio para la superficie Mutex-reject. **La migración del neg `neg-core03-compose-mutex-hold` pertenece a ADR-293 (E0347)**, no a este ADR.
- **Código propuesto:** **E0346** — mensaje EN canónico: **`mutex concurrency is not available in this surface`**.
- **No reabre:** E0242 `borrow held across await`, E0312 en su contrato histórico, E0340–E0345, ADR-229/286/290, ni el código de emisión. E0313 no se reutiliza.

## 1. Proposed decision and bounds

**D1 — Nature.** In v0, `Mutex`, `Arc<Mutex<..>>`, and the `.lock()` method **do not exist on the ARITA surface**: they are rejected with E0346, before emitting Rust. The exit is actionable and stable; the accidental E0100 of a nonexistent function is not accepted as a Mutex diagnosis (that free call, e.g. `mutex_new(0)`, belongs to ADR-293, E0347).

The canonical E0346 text is exactly:

```text
E0346: mutex concurrency is not available in this surface
```

The diagnosis is anchored on the type identifier (`Mutex`/`Arc`) or on the method selector `lock`, with the smallest available span. The exact way to obtain the `MethodCall` span remains Q1 (§8).

**D2 — Closed v0 surface.** Only the following enter:

1. `Mutex` as a type or construction name, including `Mutex<T>` when the parser can recognize it.
2. `Arc`, and the compound form `Arc<Mutex<T>>` when the parser can recognize it.
3. `.lock()` on a receiver.

`RwLock` stays **out**: the Parser PREP does not treat it and there is no evidence of a current rule or contract for it.

**D3 — Out of scope.** Functional Mutex, functional `Guard`, `unlock`, guard-RAII, `thread_local`, async locks, sharing across tasks, `spawn` with arguments, poisoning policy, emitted `Arc`, and any Codegen change. Real Mutex is recommended for v1.1, not this slice.

**D4 — Future alternatives.** The Parser PREP leaves two possible forms for a later v1: (a) own rejection with an ARITA code; (b) opaque `Named` types (`Mutex`/`Guard`) with an E0346 neg. This ADR recommends **(a)** for v0: it does not open a partial semantics nor create a guard whose lifetime cannot yet be bounded. Alternative (b) remains a v1.1 design, not a pin of this ADR.

**D6 — NO (B-286-7 lives in ADR-293).** This ADR neither treats nor closes B-286-7 (call to undeclared function typed `Int`). `mutex_new(0)` and any undeclared free constructor are rejected with **E0347** by ADR-293, **not** with E0346; this ADR migrates no existing neg.

## 2. Verified facts: what reaches the parser and HIR today

### 2.1 Generic types

- `ty_named = { ident }` en `crates/arita-syntax/src/arita.pest:74` no admite `<...>`.
- Generic rules are closed: `ty_vec_int` L63, `ty_list_int` L65, `ty_map_text_int` L66, `ty_result` L69, and `ty_option` L72; `let_stmt` uses the closed list at L112–115.
- Therefore `let m: Mutex<Int> = ...` and `Arc<Mutex<Int>>` die at parse as generic E0006 (`construct outside F1.1 (parse failure)`) before HIR: `DOC/reviews/PREP_MUTEX_PARSER_20261003.md:46–57`.
- A `Mutex` parameter or return without `<...>` does go through `ty_named`: `arita.pest:84`, `:276–277` **(corrected: formerly `:274–277`)**; lowering produces `Type::Named` (`crates/arita-syntax/src/lib.rs` and HIR).

### 2.2 `.lock()` y constructores asociados

- `method_call_other = { ident ~ "." ~ ident ~ "(" ... }` en `arita.pest:123`; `m.lock()` baja a `Expr::MethodCall` (`DOC/reviews/PREP_MUTEX_PARSER_20261003.md:64–72`).
- There is no chaining: `m.lock().unwrap()`, `*m.lock()`, and `m.lock().len()` are not expressible today (`PREP_MUTEX_PARSER_20261003.md`).
- `Mutex.new(0)`/`Arc.new(0)` have the associated-constructor shape that HIR tries to resolve by name (`PREP_MUTEX_PARSER_20261003.md:71, 96–106`). Today they end in E0206 for an unknown method.
- `Type::Named` exists in AST/HIR, but there is no validation that the name exists: `PREP_MUTEX_PARSER_20261003.md:58–62, 96–106`.
- An undeclared free call such as `mutex_new(0)` reaches HIR and returns `Int` via B-286-7 behavior (today `crates/arita-hir/src/lib.rs:1305–1311`; **(corrected)**: the citation may drift). That path is ADR-293’s, not this ADR’s.

### 2.3 Await, loans, and guard

- E0242 today covers only two `HirExpr::Await` arms (`crates/arita-hir/src/lib.rs:881–907` **(corrected: was `:852–878`)**); its contract is `borrow held across await`, ADR-039 L23–31.
- Current HIR loans come from `&`/`&mut` or the temporary receiver; a guard returned by `lock` would be a `Named` value, not a `Loan` (`PREP_MUTEX_PARSER_20261003.md:142–1`).
- There are no HIR push/pop scopes for `if`, `while`, or `match` bodies; loans live until the end of the function (`PREP_MUTEX_PARSER_20261003.md:74–87`). That is why guard-RAII and hold-across-await are later slices.
- ADR-039 L59 keeps Mutex×await in PARK; ADR-286 B-286-12 keeps guard-RAII and `thread_local` as P3 backlog (`DOC/ADR/286-core-0.10-errores-fase1.md`).

### 2.4 Current accident of the existing neg

`ejemplos/core03/client-compose/neg/02-mutex-hold.arita:1–8` contiene:

```arita
// CUT CORE-0.3-CLIENT-COMPOSE-20260920 — neg Mutex HOLD must reject
// Oracle: neg-core03-compose-mutex-hold. Do NOT unpark Mutex.
module core03_neg_mutex

async fn main() -> Io<()> {
  let _m: Int = mutex_new(0)
  print("should-not")
}
```

The oracle evidence says E0100, not an own Mutex diagnosis: `DOC/GATE-CORE03-CLIENT-COMPOSE-20260920.md:8–13`; Measure detail is in `DOC/reviews/MEASURE_ADR274_SC*.md` and related freezes that contain the fixture sha.

**Migration of `neg-core03-compose-mutex-hold` belongs to ADR-293 (E0347)**, not this ADR (Engineer decision, 2026-10-04): the fixture (`mutex_new(0)`) contains neither `Mutex`, `Arc`, nor `.lock()`, so it is outside the closed D2 surface. This ADR does not migrate it.

### 2.5 Parser IMPL reference: `ADR-291-PARSER-IMPL-DRAFT-20261004-r2.md`

Source: `DOC/reviews/ADR-291-PARSER-IMPL-DRAFT-20261004-r2.md` (sha256 `fa35c7a89920961dd38d3c1b98c99393891bf9cea6d4689e609caa3e36fc5e8a`, 232 lines; hereafter **r2**). Status on a clean Lex copy with dry-run; line citations from r2, read today; code lines re-verified today on the tree (hir `d0571e12…`, syntax `187cff0e…`, pest `7c28aa3c…`, unchanged).

- **Interception point: `arita-syntax`, not HIR** (r2 L105–L111). `Expr::MethodCall` (`crates/arita-syntax/src/lib.rs` L241) and `Call` (L287) store no span; HIR `CheckError::Coded` has no span field; an E0346 anchored on the identifier can only be emitted where pest `Pair`s exist: lowering or `classify_pest_error` (L518). `ty_named = { ident }` (`arita.pest` L74) makes the pair span exactly the identifier. `lower_method_call_other` (L927) is invoked from L1262, L1608, L1993, L2550 and L3065; `lower_host_call` (L1899) is delegated by `lower_host_call_try` (L2983). Sites that build `Type::Named`: L742, L791, L2140, L2187 and L2469. `arita.pest`, HIR and Codegen do not change.
- **What is detected** (r2 L114–L121): bare `Mutex`/`Arc` as a type without `<>` (let, param, ret, Result/Option argument) in a new `lower_ty_named`; generic `Mutex<..>`/`Arc<..>` and `x.lock().m()`/`*x.lock()` via the pest failure path (lexical scan of the failure line, after E0005 and before E0007); `Mutex.new(..)`/`Arc.new(..)` in `lower_method_call_other` (anchor: the receiver); `<recv>.lock(..)` of any receiver (anchor: the `lock` selector); `host.lock(..)` in `lower_host_call`.
- **Mensaje:** `E0346: mutex concurrency is not available in this surface`; `ParseError::Coded`’s `Display` appends ` @start..end` (syntax L343–L355; r2 L125), so real stderr is e.g. `E0346: mutex concurrency is not available in this surface @60..65`.
- **Span de `MethodCall` (Q4):** none after lowering (r2 L127–L128); spans live only in the `Pair`s (`recv`, `method_pair`) inside `lower_method_call_other`. So HIR cannot anchor this code.
- **Precedencia** (r2 L130–L136 and L45): E0346 is parse-phase and beats any HIR error (E0007, **ADR-293 E0347**, E0203, E0206, E0343/E0344, E0340); observed on a copy: `Mutex.new(0)` with `undeclared(1)` before or after yields E0346 in both orders. Bare `mutex_new(0)` never yields E0346 (with ADR-293, E0347).
- **Validation on a copy** (r2 L18–L27, not a GATE measurement): `cargo fmt --check` and `clippy -D warnings` rc 0; `cargo test -p arita-syntax adr291` 7 passed and `-p arita-hir adr291` 5 passed; full suites 85 (syntax) and 179 (hir) passed, 0 failed; with ADR-293 applied, same results. Sweep of 867 `.arita` base vs base+ADR-291: 0 changes (r2 L40). Final diffs (r2 L49–L50): `adr291_parser_final.diff` (189 l., sha256 `9c8fc38b756646e136dc089365dd948a345e886f0555a83f8f97ad103b664c37`) and `adr291_tests_final.diff` (269 l., sha256 `f5e3117d0351b6d8cd5443ea6240e163d3b3e87c85396ab2427fb945e513ab22`), both under `<scratch>/arita_adr291/`.
- **Sources and spans of the 4 oracles** (r2 L29–L37 and L160–L165): `<scratch>/arita_adr291/oracles/01-type.arita` (`@60..65`), `02-arc.arita` (`@36..39`), `03-constructor.arita` (`@66..71`), `04-lock.arita` (`@85..89`); predicted spans = observed on the copy. Recalculated today on those sources: all four match (in `04-lock` the `.lock(` selector is at 85..89; the first `lock` in the file is the module name). They are new sources; **not** the frozen `mutex_new(0)` fixture.

**Findings (r2 vs code vs this ADR):**
1. **HIR → syntax.** This ADR’s v0.1/v0 placed constructor and lock in HIR (§4, §5); r2 L211 contradicts that and L105–L111 proves it by code. Corrected in §4 and §5 of this v0.2.
2. **Stale citations in this ADR** (r2 L207): hir `852–878` → today **L881–L907** (`Await` arm, E0242 at L887 and L899); `arita.pest:274–277` → today **L276–L277** (`fn_ret_ok`/`fn_ret_bad`). **(corrected)** in §2.1 and §2.3. Citations to `PREP_MUTEX_PARSER_20261003.md:NN` in this ADR apply to the original (275 lines), not to `-r2` (491 lines) (r2 L208).
3. **Order and N in r2** (L69 and L232): “ADR-292 → ADR-293 → ADR-291”, with N figures, omits ADR-294. The current order is 293 → 294 → 291 → S2 (Engineer, 2026-10-04) and N is recalculated by the Engineer at GO: not cited here.
4. **r2 L2549 → L2550.** r2 §3.1 cites L2549 as one of the calls to `lower_method_call_other`; the call is at L2550 (L2549 is `Rule::method_call_other => {`). Minor.
5. **Diff sizes.** r2 §3 (L100–L101) gives 188/245 lines; r2 R8 (L49–L50) gives 189/269 after R3. R8 prevails.
6. **`RwLock` (Q7).** r2’s source with `fn f(a: RwLock)` fails with E0006 for parameter shape, not for RwLock; r2 replaces it with `let r: RwLock = 0` (L15). Does not change Q7.
7. **ADR-293 is not rustfmt-clean** (r2 L46): the `adr293_hir_final.diff` patch leaves 13 lines `imports: vec![],  // ADR-293 test literal fix` with two spaces before `//` (verified today in the diff) and fails `cargo fmt --check`; r2 supplies `<scratch>/arita_adr291/adr293_fmtfix.diff` (sha256 `9f7e0d07324dd2dae9d8bbb47fbee6d31deed567d5d2317fea795c5e4fa4a50c`). Affects ADR-293 IMPL; this ADR does not edit it.
8. **r2 L139** speaks of “~20 test literals” of ADR-293; ADR-293 §3 and its sweep give 13 test literals.
9. **Message with suffix.** The exact E0346 text remains as prefix; real stderr adds ` @a..b`. The Measure oracle must compare by prefix or by the computed span (r2 L204).

## 3. Code E0346

**D5 — choice.** E0346 is proposed because `rg` in `crates/`, `DOC/`, and `ejemplos/` finds no E0346–E0360 (`DOC/reviews/PREP_MUTEX_PARSER_20261003.md:212–231`). **(corrected, 2026-10-04)** E0347 remains assigned to ADR-293 (call to undeclared function); E0346 stays free and is this ADR’s code.

- E0312 is not reused: ADR-229 L20–24 defines it as `Mutex not available in this profile/surface` and ties it to `neg-e0312-mutex`; its checklist stays open at L46–48. Reusing it would keep a contradiction when Mutex has a future profile.
- E0313 is not reused either: ADR-229 L32–35 reserves it for hold-across-await, and the PREP documents that E0313 is already used by timeout/delay/cancel (`PREP_MUTEX_PARSER_20261003.md:227–231`).
- E0340–E0345 are used or reserved; E0333–E0339 are reserved by ADR-290 (`DOC/ADR/290-core-index-mut-v0.md:9–10, 46–50`).

Proposed canonical EN message: `mutex concurrency is not available in this surface`. It is not mixed with E0242: E0242 diagnoses a live loan that crosses `await`, while E0346 rejects the surface before a guard exists.

## 4. Minimum rejection point

**Architect recommendation (v0.2, Q1 resolved: Parser choice, without widening what it accepts):** all forms are rejected in **`arita-syntax`** (parse phase), with the same E0346 anchored on the identifier (§2.5). **(corrected)** Prior text sent `Named`/`MethodCall` forms to HIR; r2 proves by code that HIR cannot anchor the span.

1. Syntax lowering: `Mutex`/`Arc` as a type without `<>`; `Mutex.new`/`Arc.new` (anchor: the receiver); `.lock()` of any receiver and `host.lock()` (anchor: the `lock` selector).
2. Pest failure path (`classify_pest_error`): `Mutex<...>`/`Arc<...>` and `.lock().x`/`*x.lock()`, by lexical scan of the failing line, before generic E0006.
3. HIR and Codegen do not change: with rejection at parse, no Rust is emitted; a `Named("Mutex")` in a parameter does not reach HIR (r2 L145).

Alternative not recommended for v0: leave opaque `Named` and measure only one E0346 case. That would let other forms reach E0206/E0100 and would not meet D2’s closed rejection.

## 5. Proposed oracles

**k = 4 FIXED** (Engineer, 2026-10-04). **N is recalculated by the Engineer at GO**; no measurement is declared in this ADR.

- `neg-core10-mutex-type`: type form `Mutex<Int>` in `let`; expected E0346, exit 1, empty stdout, identifier span. Proposed new source `ejemplos/core10/mutex-reject/01-type.arita`.
- `neg-core10-mutex-arc`: form `Arc<Mutex<Int>>` in a parameter; expected E0346, exit 1, empty stdout, span of the first reserved identifier. Proposed new source `ejemplos/core10/mutex-reject/02-arc.arita` (`878609be7249096e`, `@36..39`).
- `neg-core10-mutex-constructor`: `Mutex.new(0)`; expected E0346 in syntax lowering (parse phase), before E0206. Proposed new source `ejemplos/core10/mutex-reject/03-constructor.arita`.
- `neg-core10-mutex-lock`: `n.lock()` with any receiver; expected E0346 on the `lock` selector (parse phase), before E0206. Proposed new source `ejemplos/core10/mutex-reject/04-lock.arita` (`497e04cbdda93a64`, `@85..89`).

The four sources are **new** (PROPOSAL from r2 L157–L167; `ejemplos/core10/mutex-reject/` does not exist today and is not created in this ADR) and **not** the frozen fixture `mutex_new(0)`. Each neg must check the exact prefix `E0346: mutex concurrency is not available in this surface` (real stderr adds ` @a..b`, §2.5), exit 1, empty stdout, and no emission. No functional positive is proposed in v0. Ids, paths, and shas are proposal until the Engineer freeze.

**Historical neg migration:** migration of `neg-core03-compose-mutex-hold` belongs to **ADR-293 (E0347)**; this ADR migrates no existing neg. The 4 rejection oracles above remain.

## 6. GO, order, and measurement

- ADR-290 and ADR-292 CLOSED are a met precondition (`DOC/ADR/290-core-index-mut-v0.md:3–4, 37–39`, `DOC/GATE-CORE10-INDEX-MUT-20261003.md`, `DOC/GATE-CORE10-INT-ARITH-RUNTIME-20261003.md`); ADR-293 and ADR-294 go first (order 293 → 294 → 291 → S2, Engineer 2026-10-04).
- GO IMPL stays blocked until explicit Engineer GO.
- The Parser PREP (static viability) and the r2 IMPL draft exist, validated on a copy (§2.5; no GATE measurement); any future Measure PREP must close without BLOCKER before GO. Parser IMPL applies the final r2 diffs (r2 L53) when ADR-293 is applied.
- Proposed order: Parser/lowering → HIR → tests/Measure. Codegen does not change if rejection stays before emission.
- The frozen fixture and closed freezes are not touched in this DRAFT.

## 7. Out of scope / residual

- `RwLock`: out; not treated in the PREP.
- Functional Mutex, shared `Arc<Mutex<T>>`, guards, `unlock`, `thread_local`, async locks, and a guard that crosses `await`: later slices.
- **B-291-1 (P2, propuesta):** real Mutex, `Arc<Mutex<T>>`, guard-RAII, poisoning and sharing policy for v1.1.
- **B-286-7 (P1, existente):** call to nonexistent function typed as `Int`; **lives in ADR-293 (E0347)** (`DOC/ADR/286-core-0.10-errores-fase1.md:258`); this ADR does not close it (D6).
- **B-286-12 (P3, existente):** signatures with guard-RAII and `thread_local` (`DOC/ADR/286-core-0.10-errores-fase1.md:263`). Guard/borrow integration with E0242 stays open and is not fixed in v0.

## 8. Engineer questions — RESOLVED (2026-10-04)

- **Q1 — RESUELTA:** `Mutex<T>`/`Arc<...>` interception (parser before E0006 or lowering layer with lookahead) is chosen by the Parser, **without widening what the parser accepts**.
- **Q2 — RESUELTA:** `Mutex.new` and `Arc.new` fall **INSIDE** the rejected set (E0346).
- **Q3 — RESUELTA:** `.lock()` is rejected **lexically for any receiver**; the `lock` selector is **reserved** (a future non-Mutex API cannot use `.lock()` without an ADR that frees it). Documented here.
- **Q4 — RESUELTA:** the exact `MethodCall` span is documented by the Parser, **without reopening B-286-11**.
- **Q5 — RESUELTA:** **k = 4 fixed**; N is recalculated by the Engineer at GO.
- **Q6 — RESUELTA:** this ADR **has no own migration**; the historic neg’s belongs to ADR-293 (E0347).
- **Q7 — RESUELTA:** `RwLock` stays **out**.
- No open questions.

## 9. Changelog

- 2026-10-03 — v0 DRAFT PINS (Architect): Mutex-reject proposal; free E0346; closed Mutex/Arc/lock scope; provisional k 4 (provisional N withdrawn in v0.2); core03 neg migration (withdrawn in v0.2: moves to ADR-293); no GO IMPL, no code or Codegen changes.
- 2026-10-04 — v0.2 DRAFT PINS (Architect): Engineer decisions: the neg `neg-core03-compose-mutex-hold` (`mutex_new(0)`) contains neither Mutex/Arc/`.lock()` and **does not migrate to E0346**: it migrates to E0347 via ADR-293 (call to undeclared function); “Single migration” and all N figures withdrawn (N is recalculated by the Engineer at GO); k = 4 fixed; D6 NO (B-286-7 lives in ADR-293); Q1–Q7 RESOLVED (Q1 Parser choice without widening what it accepts; Q2 `Mutex.new`/`Arc.new` inside; Q3 lexical `.lock()` for any receiver, reserved selector; Q4 span documented by Parser without reopening B-286-11; Q5 k = 4; Q6 no own migration; Q7 RwLock out); order 293 → 294 → 291 → S2; v0.1 requested after B-286-7 PREP, superseded by ADR-293. No GO IMPL, no code changes.
- 2026-10-04 — v0.2 (extension, Architect): **this v0.2 equals the no-migration v0.1 the Orchestrator asked for after the B-286-7 PREP.** Incorporated the Parser IMPL reference `ADR-291-PARSER-IMPL-DRAFT-20261004-r2.md` (§2.5): interception in `arita-syntax` (not HIR), span by `Pair`, new sources and spans of the 4 oracles (proposal), E0346 beats E0347 by phase, r2 vs code vs ADR findings (9); corrected citations (`arita.pest` L276–277, HIR `Await` L881–907). No own cargo, no OWN ROADMAP, no seals; no GO IMPL.

## Engineer Seal (2026-10-04)

Sealed on version v0.2 with sha256 `774267483b798c0a5fe7f7615927bae31c25503a448d07adf37c5494f15ade15` (168 lines). This section is appended at the end; nothing above has been modified.

**Decision: v0.2 ACCEPTED as the pins contract for ADR-291.** GO IMPL is given in writing after ADR-294 CLOSED (order 293 → 294 → 291 → S2).

1. **Scope confirmed.** E0346 `mutex concurrency is not available in this surface` in `arita-syntax` (parse phase), no changes to production HIR, Codegen, or `arita.pest`; closed D2 surface (`Mutex`, `Arc`, `.lock()` of any receiver, with the `lock` selector reserved). `RwLock` out. No own migration: that of `neg-core03-compose-mutex-hold` already closed with ADR-293 (E0347).
2. **Oracles k = 4 and N.** The four negs of §5 with new sources under `ejemplos/core10/mutex-reject/`. Queue figure: 878 → **882**, and S2 adds k(S2) = 7 afterward (889). §5 ids and paths remain accepted; exact sources, and thus `@a..b` spans, are fixed by the real run, not this proposal.
3. **Diagnosis comparison.** The oracle requires the exact prefix `E0346: mutex concurrency is not available in this surface`, single E-code, exit 1, empty stdout, no emitted Rust, and a control that builds with 0 diagnoses. The stderr ` @a..b` suffix is mandatory and must be a non-empty span inside the source; it is not fixed in the id.
4. **Precedence.** E0346 (parse) beats E0347 (HIR) in both orders, measured by Parser on the 293 → 294 → 291 chain. A unit test must lock it.
5. **Reserved-selector risk.** The 867 `.arita` sweep does not change with 291 (0 changes from 291); Measure must confirm in its PREP that no existing oracle, fixture, or example uses `.lock(` or the identifiers `Mutex`/`Arc`. Any hit is a BLOCKER.
6. **Patches.** Only the Parser final diffs with sha256 count (`adr291_parser_final.diff` `9c8fc38b…` and `adr291_tests_final.diff` `f5e3117d…`), applied to the real tree by the Engineer after the 294 GATE; `cargo fmt --check` and clippy `-D warnings` must pass in the run. Zero existing tests modified.
7. **Close.** Same gate as 293/294: fmt 0, clippy 0, release build, `cargo test --workspace -- --test-threads=1`, measure N/N with no skip, Veyra ACCEPTED, own GATE `DOC/GATE-CORE10-MUTEX-REJECT-<fecha>.md`, freeze and addendum new ones written by me. No prior freeze or seal is rewritten; this ADR is not edited after the Seal.
8. **Backlog.** B-291-1 (P2, real Mutex in v1.1). Functional Mutex stays outside v1.
