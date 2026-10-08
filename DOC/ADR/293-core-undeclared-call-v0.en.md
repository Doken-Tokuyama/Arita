Translation of `293-core-undeclared-call-v0.md`; the original is normative. / Traducción de `293-core-undeclared-call-v0.md`; el original es el normativo.

# ADR-293 — Core UNDECLARED-CALL v0: call to undeclared function (B-286-7, P1)

- **Estado:** **DRAFT PINS v0.1 (Arquitecto, 2026-10-04; decisiones del Ingeniero incorporadas; sin GO IMPL).** Pendiente la PREP de Measure (§7). No declara PASS, CLOSED ni N/N medido.
- **CUT-ID:** `CORE-0.10-UNDECLARED-CALL-20261003`
- **Fecha:** 2026-10-04
- **Autores:** ARITA Arquitecto (pins) · decisiones del Ingeniero 2026-10-04 · hechos de Parser en `DOC/reviews/PREP_B286_7_20261003.md` (en adelante **PREP**) y `DOC/reviews/ADR-293-PARSER-SWEEP-20261004.md` (en adelante **BARRIDO**)
- **Padre / contexto:** [ADR-286](286-core-0.10-errores-fase1.md) (B-286-7, B-286-11) · [ADR-291](291-core-mutex-reject-v0.md) (neg `mutex_new`; DRAFT, ver D7) · [ADR-292](292-core-int-arith-runtime-v0.md) (CLOSED, N = 866 según `DOC/GATE-CORE10-INT-ARITH-RUNTIME-20261003.md`) · PREP B-286-7 · BARRIDO del Parser
- **Cierra (al CLOSED):** **B-286-7** (P1) y la migración del neg `neg-core03-compose-mutex-hold` desde el fallo accidental E0100 a E0347.
- **Código:** **E0347** (libre en `crates/`, ADR y `ejemplos/`, §2) — mensaje EN canónico: ``E0347: call to undeclared function `NAME` `` (`NAME` = nombre de la llamada). **Sin span.**
- **No reabre:** B-286-11 (spans en `Call`/`HirCall`; sigue abierto, P2) · E0206/E0313/E0320/E0321 y su orden en el brazo `Call` · E0241/E0203 de `spawn`/`join` · E0007 · E0340–E0345 · ADR-291 (su texto) · ADR-292 · ADR-289 · Codegen.

## 1. Decision and bounds

**D1 — Code and message.** A free call to a function that is not declared in the module, not imported by `use`, and not a builtin ⇒ **E0347** with the exact text ``E0347: call to undeclared function `NAME` ``. No span: `Call`/`HirCall` carry no span and B-286-11 stays open (`DOC/ADR/286-core-0.10-errores-fase1.md` L262); the diagnosis does not reopen it.

**D2 — Check site.** In HIR, in the `HirExpr::Call` arm of `eval_expr` (`crates/arita-hir/src/lib.rs` L743–L884), inside ADR-244’s `match c.callee`, **in the `_ => {}` hole (L865) that remains after** E0320 (`busy_spin`/`hang_forever`, L772), E0313 (`timeout`/`delay`/`cancel*`, L778–L846), E0206 (`http_*`, L853–L858) and E0321 (`reqwest`, L860–L864), and **BEFORE evaluating the arguments** (`for arg in &c.args`, L875–L877) and the `async_fns` check (L867). Earlier arms and the early return of `spawn`/`join` (L744–L765) thus keep their priority.

**D3 — Rule.** Reject if the callee meets **no** exemption: (1) contains `::` (`Vec::new`, `List::new`, `Map::new`, `host::*`); (2) is `print`; (3) is `DEFERRED_SHAPE_MARKER` (`crates/arita-syntax/src/lib.rs` L1745); (4) is in `BUILTIN_FN_NAMES` (HIR L5146); (5) is in `fn_rets`, i.e. declared in the module (L5217); (6) is imported by `use` (D4).

**D4 — `imports` is MANDATORY.** New field `imports: Vec<String>` on `HirModule` (L61), populated in `lower_ast` (L5323) from `module.uses[].item`; `CheckCtx` receives it as a `HashSet` and `check` (L5177) assigns it at the three sites where it builds a `CheckCtx` (two loops over `hir.functions` and the `tests` one). The SWEEP proves the need: **without** the imports exemption **16** programs change; **with** it **exactly 1** changes (`ejemplos/core03/client-compose/neg/02-mutex-hold.arita`, ok → E0347), **0 breaks**, over **867** `.arita`. Touching `HirModule` forces editing **13** test literals (§3).

**D5 — Codegen does not change.** The call is rejected in HIR before emit: not one byte of emitted Rust changes, and the migrated neg no longer reaches rustc/cargo.

**D6 — Single migration.** `neg-core03-compose-mutex-hold` (frozen id and fixture, sha256 `4ea5e12123c7ae22e1b4af62827f9e1a45a8def29d0a84aa44c373b7da222d79`) moves from accidental E0100 to **E0347** (`mutex_new` call), with a new freeze and addendum written by the Engineer (§5).

**D7 — Interaction with ADR-291.** `mutex_new(0)` is a free call: it contains neither `Mutex`, `Arc`, nor `.lock()`, so it does **not** belong to ADR-291’s E0346 scope (its own §2.2 recognizes this: “`mutex_new` is not a new form of the closed D2 scope”). With ADR-293 before ADR-291 (Lex queue order, ADR-292 GATE), the neg migrates to **E0347 and NOT to E0346**; ADR-291 (v0.1, pending) loses its “Single migration” and keeps its own oracles (type, `Arc`, constructor, `lock`) on new sources. ADR-291’s correction (§5, §2.4, k and N) is done by its author; this ADR does not edit it. ADR-291’s N figures are recalculated by the Engineer: they are not fixed here.

**D8 — Outside scope.** (a) B-286-11 (spans on `Call`); (b) functions from imported modules (existence and signature are not resolved: the CLI does that with E0404/E0405/E0406/E0331); (c) unbound variables (unbound `Path` ⇒ `Int`, PREP L60 and L224); (d) any Codegen change.

## 2. Evidence (PREP + SWEEP + code)

- **Recorrido actual de `mutex_new(0)`** (PREP L23–L44): the parser accepts `user_call`; HIR has no `mutex_new` arm (falls into `_ => {}`) and `type_of_expr` types it `Int` via the `fn_rets` fallback (today L1305–L1311); Codegen emits the call; cargo fails late and the CLI wraps it as E0100 (`async/tokio`). The `run_core03_compose_mutex_hold_oracle` checks only the `E0100` substring (`crates/arita-cli/src/measure.rs` L18368–L18411; registry at L18542). **(corrected)**: ADR-286 B-286-7 row (L258) and ADR-291 cite HIR L1275–L1282 / L1276–L1281, and PREP cites measure L17704–L17745; current positions are those in this paragraph.
- **Código libre** (PREP L62): `rg` over `crates/`, `DOC/ADR`, `ejemplos/` finds no E0347 definitions; it only appears in `DOC/reviews/` notes (PREP B-286-7, PREP B292_2, PREP S2 must-use, and an **optional suggestion** not adopted for an escaped guard in `PREP_MUTEX_PARSER_20261003.md` L229). E0346 is an ADR-291 proposal; E0333–E0339 are reserved by ADR-290.
- **Corpus (PREP L12–L22):** 860 `.arita` in the PREP; 150 unique free call names, of which 12 undeclared; 68 of the 73 undeclared calls are from `ejemplos/f3/` (Datalog logic island, do not go through parser/HIR); the remaining 5 are `reqwest_get` ×4 (E0321, unchanged) and `mutex_new` ×1.
- **BARRIDO (ejecución real de parse + `lower_ast` + HIR check, en copia `/tmp/arita_adr293`):** 867 `.arita` (the PREP’s 860 plus 7 from `core10/int-overflow`). Baseline: 530 `ok` / 337 with diagnosis; with patch: 529 / 338. **1 file changes** (`02-mutex-hold`: `ok` → ``E0347: call to undeclared function `mutex_new` ``); **0 breakages** (the 516 non-`neg` `ok` stay `ok`); the 4 `reqwest` negs stay on E0321; the 12 from `f3/` unchanged. **Ablation** (without the import exemption): 16 changes instead of 1; the extra 15 are files that call fns imported via `use`.
- **Confirmaciones del BARRIDO:** `assert f(1) == 2` in `test` with undeclared `f` does go through `eval_expr` and yields E0347 (with `f` declared, `ok`); `g(h(1))` yields E0347 on `g` (callee before evaluating arguments); `cargo test -p arita-hir -p arita-syntax` on the copy: 174 + 78 passed, 0 failed, original and patched; instrumented, 0 guard impacts across the 174 HIR tests.
- **Parche de referencia:** `/tmp/arita_adr293/adr293_hir_final.diff` (192 lines, sha256 `15a9ef142973ec58941500b02cc7256bdd9d0fe9eb6341ef88cc871b5991ac63`; identical to `adr293_hir.diff`). Feasibility evidence on a copy, **not** the implementation: the real tree was not touched.
- **Referencias de código verificadas por lectura** (`crates/arita-hir/src/lib.rs`, sha256 `d0571e12419dae491ec7ce5dcfba99ad247c045793fc3c1f6c53abf1ee575807`, 9630 lines): `HirModule` L61; `CheckCtx.fn_rets` L417 and `CheckCtx::new` L434; `Call` arm L743–L884; E0320 L772, E0313 L778–L846, E0206 `http_*` L853–L858, E0321 `reqwest` L860–L864, `_ => {}` L865, `async_fns` L867, argument evaluation L875–L877; generic `type_of_expr` L1305–L1311; `BUILTIN_FN_NAMES` L5146; `check` L5177; `fn_rets` L5217; `lower_ast` L5323. `HirModule` test literals: 20 lines with `HirModule {` = 1 struct + 1 signature and 1 `lower_ast` literal + 4 helper signatures + **13 test literals** (L5641, L5704, L7413, L7587, L7737, L7783, L7812, L7835, L7902, L7950, L7988, L8035, L8088); matches the SWEEP.

## 3. Planned minimal change (to verify in IMPL)

- **`arita-hir`:** `HirModule.imports`; `CheckCtx.imports`; the D2/D3 exemption and error arm; fill-in in `lower_ast`; assignment at the 3 `check` sites; **13 test literals** with `imports: vec![]` (PREP said 20: it also counted signatures and the struct; **(corrected)** with the SWEEP and §2 reading).
- **Tests cargo adicionales** (do not count toward N; PREP L216 and SWEEP confirmations): E0347 with exempt imported fn (`imports` populated); `print`/`Vec::new`/`host::*` exempt; precedence vs E0007, `spawn(ghost())` (E0241) and `reqwest_*` (E0321); `assert f(1) == 2` in `test` (E0347).
- **Measure:** migrate the `neg-core03-compose-mutex-hold` oracle (today requires the `E0100` substring) and register the 5 §4 oracles; Measure/Engineer does it, not this ADR.
- **Sin cambios:** `arita-syntax`, Codegen, `arita-cli` (except the `measure.rs` oracle), `package.rs`.

## 4. Proposed oracles (k = 5)

Ids and sources are those of PREP §4.4 (~6-line sources, `[L]`: not executed); oracle ids and fixture paths are **fixed by the Engineer Seal** (Q1, below). Each neg: stderr with the exact text ``E0347: call to undeclared function `ghost` ``, exit 1, empty stdout, and no Rust emission.

1. **`neg-core10-undeclared-call-stmt`** (`ejemplos/core10/undeclared-call/neg/01-stmt.arita`): `main` with `ghost(1)` as a statement followed by `print("after")`.
2. **`neg-core10-undeclared-call-let`** (`ejemplos/core10/undeclared-call/neg/02-let.arita`): `let n: Int = ghost(1)` y `print(n)`.
3. **`neg-core10-undeclared-call-print-arg`** (`ejemplos/core10/undeclared-call/neg/03-print-arg.arita`): `print(ghost(1))`.
4. **`neg-core10-undeclared-call-in-result-fn`** (`ejemplos/core10/undeclared-call/neg/04-in-result-fn.arita`): the call `let r: Int = ghost(n)` inside `fn f(n: Int) -> Result<Int, Int>`, not in `main`.
5. **`core10-declared-call-ok`** (`ejemplos/core10/undeclared-call/05-declared-ok.arita`, outside `neg/`; positive control, no `pos-` prefix per the Seal; **(corrected)** PREP called it `pos-core10-declared-call-control`, same shape as (2) with `fn ghost(n: Int) -> Int { n }` declared): builds green; stdout `1` [derived from the source, not executed].

**k = 5. N:** numbered against N = 866 (ADR-292 CLOSED) ⇒ **871 (866 + 5)**, the value ADR-292’s GATE records for this queue; **the Engineer recalculates it at GO**. The **migration** of `neg-core03-compose-mutex-hold` keeps its id and does not add to N. Rules: skip ≠ PASS; no oracle counts if the build did not run. Existing negs for `reqwest` (E0321), `http_*` (E0206), and `busy_spin` (E0320) serve as precedence guards unchanged.

## 5. Migration and backlog

**Migration of `neg-core03-compose-mutex-hold`.** Fixture `ejemplos/core03/client-compose/neg/02-mutex-hold.arita` (sha256 `4ea5e12123c7ae22e1b4af62827f9e1a45a8def29d0a84aa44c373b7da222d79`, verified today): E0100 ⇒ E0347. Requires a **new freeze + addendum written by the Engineer**; **no existing freeze is rewritten**. ADR-291 (§2.4) lists four freezes that contain that sha: `MEASURE-ADR284-FREEZE-20260927.sha:175`, `MEASURE-ADR285-FREEZE-20260927.sha:175`, `MEASURE-ADR286-FREEZE-20260927.sha:177`, and `MEASURE-ADR286-S1B-FREEZE-20261002.sha:189` (verified). **(corrected)** `rg` of that sha in `DOC/reviews/*.sha` also finds `INGENIERO-ADR283-FREEZE-20260927.sha:160`, `MEASURE-ADR287-PKG-MEMBER-FREEZE-20261002.sha:202`, `MEASURE-ADR288-EMIT-CLIPPY-FREEZE-20261003.sha:205`, `MEASURE-ADR289-EMIT-UNUSED-PARENS-FREEZE-20261003.sha:210`, `MEASURE-ADR290-INDEX-MUT-FREEZE-20261003.sha:217`, and `MEASURE-ADR292-INT-ARITH-RUNTIME-FREEZE-20261003.sha:223`: **ten** in total; none are rewritten. The `measure.rs` oracle is changed to E0347 only with real evidence.

**Backlog.**
- **B-286-7 (P1):** closes at this ADR’s CLOSED (documentary close with addendum on ADR-286 by the Engineer).
- **B-286-11 (P2):** spans on `Call`/`HirCall`; **stays open**.
- **B-293-1 (P3, propuesta):** unbound variables still silently type as `Int` (PREP L60 and L224; not re-verified by reading here).
- **B-293-2 (P3, propuesta, [H]):** calls to imported fns still type as `Int` via the `fn_rets` fallback on the loose-file-with-`use` path (PREP L104); D4 only exempts them, does not resolve them.
- **B-293-3 (P3):** in v0 every callee with `::` is exempt (including a nonexistent `modulo::f`); existence is resolved by the CLI (E0404/E0405/E0406/E0331). Narrow the `::` exemption to a closed list in an associated-methods ADR.

## 6. Riesgos

1. **Imports:** without the exemption, the sweep yields 16 changes instead of 1 (D4); touching `HirModule` touches 13 test literals.
2. **Neg ownership with ADR-291:** a fixture can expect only one code (D7); if the order were inverted, ADR-291 would need a `mutex_*` name check before this rule.
3. **Measure coverage:** the SWEEP uses approximate `use` in “mode B” (without checking `pub`, approximates `package.rs` L500); it does not cover `.arita` sources composed with `format!` in tests nor fixtures Measure copies into `/tmp` (PREP L124–L130). §2 figures are from a copy, not a GATE.
4. **Future builtins:** a new free builtin, closures, or fn-as-value must enter the exemption list in the same ADR that introduces them (ADR-286 §0.1d (c) rule, PREP L223).
5. **Logic island:** `ejemplos/f3/` (Datalog, `arita logic`) does not go through parser/HIR (`E0006` in `spec`); the rule does not affect it [PREP L16].

## 7. GO, order, and measurement

- Measure PREP without BLOCKER; implementation in `arita-hir` (+ Measure oracles and fixtures); Codegen and Parser unchanged.
- **GO IMPL** lo da el Ingeniero. **Orden:** ADR-293 va ANTES que ADR-291 (cola de Lex; **(corregido)** el GATE de ADR-292 decía ADR-293 → ADR-291 → S2 MUST-USE; el Sello fija ADR-293 → ADR-294 → ADR-291 → S2); una carga pesada a la vez.
- Measurement with real evidence (exclusive measure, `cargo test`, fmt, clippy, Veyra); inventing PASS / N/N / CLOSED is forbidden. Close with own GATE `DOC/GATE-CORE10-UNDECLARED-CALL-<fecha>.md`.

## 8. Engineer questions — RESOLVED (Seal, 2026-10-04)

- **Q1 — RESUELTA (Sello):** fixtures at `ejemplos/core10/undeclared-call/neg/01-stmt.arita`, `02-let.arita`, `03-print-arg.arita` and `04-in-result-fn.arita`, and control at `ejemplos/core10/undeclared-call/05-declared-ok.arita` (outside `neg/`); oracle ids `neg-core10-undeclared-call-{stmt,let,print-arg,in-result-fn}` and, for the control, `core10-declared-call-ok` (no `pos-` prefix).
- **Q2 — RESUELTA (Sello):** in v0 every callee with `::` is exempt (including a nonexistent `modulo::f`); existence is resolved by the CLI (E0404/E0405/E0406/E0331). Opens **B-293-3 (P3)** (§5).
- **Q3 — RESUELTA (Sello):** acceptable in v0: exemption by item name is deliberate; path distinction stays with B-293-2.
- **Q4 — RESUELTA (Sello):** the Architect corrects ADR-291 now (without waiting for this ADR’s GO): removes its “Single migration” and fixes k = 4; its N is recalculated by the Engineer at GO IMPL with the queue’s real N. **Done:** ADR-291 v0.2 (no “Single migration”, fixed k = 4, no N figures).
- No open questions.

## 9. Changelog

- 2026-10-04 — v0.1 DRAFT PINS (Architect): Engineer decisions 2026-10-04: E0347 with message ``call to undeclared function `NAME` `` no span; check site in the `eval_expr` hole after E0320/E0313/E0206/E0321 and before the arguments; `HirModule.imports` mandatory (sweep: 16 → 1 change, 867 `.arita`, 0 breaks); k = 5 (4 negs + control); migration of `neg-core03-compose-mutex-hold` to E0347 (ten freezes that contain it, none rewritten); interaction with ADR-291 (the neg does not migrate to E0346); N = 871 provisional, the Engineer recalculates; no GO IMPL.
- 2026-10-04 — v0.1 (backlog, Architect): added **B-293-3 (P3)** (narrow the `::` exemption to a closed list in an associated-methods ADR) and §8 Q2 marked RESOLVED by the Engineer (every callee with `::` exempt in v0). No changes to D1–D8, k, or N; the “Engineer Seal” is not touched.
- 2026-10-04 — v0.1 (reconciliation with the Seal, Architect): oracle ids (`neg-core10-undeclared-call-*`; control `core10-declared-call-ok`, not `pos-core10-declared-call-control`) and fixture paths (`ejemplos/core10/undeclared-call/neg/{01-stmt,02-let,03-print-arg,04-in-result-fn}.arita`; control `05-declared-ok.arita` outside `neg/`) aligned with the sealed answers; §8 Q1–Q4 marked RESOLVED; B-293-3 wording same as the Seal’s; queue order in §7 → 293 → 294 → 291 → S2. Only above the Seal; the “Engineer Seal” is not touched. No changes to D1–D8, k, or N.

## Engineer Seal (2026-10-04 12:45)
Seal of v0.1 with sha256 `db6d8b5338b8faf910b8a31b92475716d87e7a7d340efa747387d202fb640469`: pins D1–D8 and k = 5 approved; **GO IMPL pending** (Lex queue: ADR-293 is next, before ADR-294, ADR-291, and S2). Answers to §8:
- **Q1:** fixtures at `ejemplos/core10/undeclared-call/neg/01-stmt.arita`, `02-let.arita`, `03-print-arg.arita`, `04-in-result-fn.arita` and control at `ejemplos/core10/undeclared-call/05-declared-ok.arita`; oracle ids `neg-core10-undeclared-call-{stmt,let,print-arg,in-result-fn}` and, for the control, `core10-declared-call-ok` (recent positives pattern, no `pos-` prefix).
- **Q2:** in v0 every callee with `::` is exempt (including a nonexistent `modulo::f`); existence is resolved by the CLI (E0404/E0405/E0406/E0331). Opens B-293-3 (P3): narrow the `::` exemption to a closed list in an associated-methods ADR.
- **Q3:** acceptable in v0: exemption by item name is deliberate; path distinction stays with B-293-2.
- **Q4:** the Architect corrects ADR-291 now (without waiting for this ADR’s GO): removes its “Single migration” and fixes k = 4; I recalculate its N at GO IMPL with the queue’s real N.
