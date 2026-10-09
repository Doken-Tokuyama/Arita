[Español](REPAIR-ORACLE.md) | English

Translation of `REPAIR-ORACLE.md`; the original is normative. / Traducción de `REPAIR-ORACLE.md`; el original es el normativo.

# ARITA Repair Oracle — compiler-guided autonomous correction

- **Status:** **v0.1.2 point 6: structured diagnostics, deterministic suggestions and `arita fix`** (rev. 1.2 — memory + budgets + envelope + stagnation)
- **Product:** the compiler is a **structured oracle**, not an error message pasted into the LLM
- **Related:** RFC rev. 3 · PRODUCT-VISION · evidence/scenarios · ADR-231
- **HOLD lifted** (GO <person> 2026-10-09 18:26, via the engineer); v0.1.2 point 6: structured diagnostics, deterministic suggestions and `arita fix` (previously: **HOLD IMPL** `arita repair` / CUT until restart GO from <person>)

## 0. Thesis

The agent proposes a **minimal change**; the compiler and validators produce **facts**; a **Repair Controller** decides:

`pass` | `repair` | `rollback` | `blocked` | `needs_human`

“Compiles” ≠ “correct”. An LLM↔compiler loop helps (e.g. RustAssistant ~74% on OSS compile errors), but it does **not** close the task without scenarios/evidence on the final artifact.

\[
\text{No autonomous repair} \Rightarrow \text{no fake success}
\]



## 0.1 Self-improvement (what it is / what it is not)

There are two “self-improvements”; ARITA starts with the **first**:

1. **External repair memory** — auditable, reversible; does not modify model weights.
2. **LLM finetuning** — expensive; only with thousands of golden pairs. Before that, it amplifies noise.

The system mainly improves the **compiler, the rules, and the Repair Controller**, not the opaque model.

## 0.2 Failure memory (immutable log)

Each failure/repair is stored in structured form (not as free-form “prompt→response”):

\[
\text{diagnosis} + \text{semantic context} +
\text{strategy} + \text{patch} + \text{measured outcome}
\]

Minimum fields: `repair_id`, language/compiler/toolchain versions, `policy_profile`, `task_kind`, diagnosis (`stage`/`code`/`category`/`semantic_rule`), `context_fingerprint` (HIR shape, types, effects, binding versions), `hypothesis`, `repair_template`, `patch_shape`, `validation_plan`, `outcome`, `quality` (`minimal_diff`, policy/dep delta, acceptance_preserved).

### Strategy retrieval

1. Normalize to a stable code (`ARI-OWN-002`…)  
2. Extract context (HIR/AIR, types, effects, profile, bindings)  
3. Retrieve compatible prior repairs  
4. Rank by success, minimality, regressions, cost  
5. Give the LLM **only 1–3** allowed strategies  
6. Sandbox + validate  
7. Record the outcome **even on failure**

Initial transparent ranking:

\[
\mathrm{score}(s) =
0.40 \cdot \mathrm{success\_rate}
+ 0.20 \cdot \mathrm{context\_similarity}
+ 0.20 \cdot \mathrm{minimality}
+ 0.10 \cdot \mathrm{test\_preservation}
- 0.10 \cdot \mathrm{regression\_rate}
\]

Symbolic keys first (no opaque embeddings that authorize repairs):

```text
diagnostic_code + semantic_rule + HIR node kind
+ language_version + types + profile + permitted repair class
```

### Template library (not free-form memory)

| Code | Rule | Valid templates |
|--------|-------|--------------------|
| `ARI-RES-014` | Nonexistent API | real binding / symbol / request dep |
| `ARI-TYP-021` | Domain type | parse/constructor / adapter / signature |
| `ARI-ERR-005` | Ignored `Result` | `?` / `match` / typed recover |
| `ARI-TOT-003` | Partial operation | `get()?` / refined index |
| `ARI-OWN-002` | borrow conflict | shorten scope / owned local / reorder / tx |
| `ARI-ASY-011` | guard × await | close scope / actor / channel |
| `ARI-EFF-004` | Missing capability | effect-free API / request permission / redesign |
| `ARI-POL-001` | unsafe/panic escape | **reject**; safe API or block |

### Learning promotion

| Level | Requirements | Use |
|-------|------------|-----|
| `observed` | passed `cargo check` | weak reference |
| `tested` | relevant tests | low priority |
| `trusted` | tests + properties + policy + minimal diff | auto-suggestible |
| `golden` | human review + regression + repetition | preferred template |
| `rejected` | compiled but broke tests/policy/semantics | **never** suggest |

Each `golden` → ARITA regression test + tighter rule/diagnostic + doc + deterministic repair if applicable.

## 1. Anti-loop (forbidden)

```text
AI generates → cargo check fails → AI tries random changes until it “goes away”
```

## 2. Correct loop

```text
Verifiable goal
   ↓
Plan and minimal change
   ↓
ARITA parse / typecheck
   ↓
Deterministic Rust emission
   ↓
Compiler, lints, tests and verifiers
   ↓
Failure classification (root cause)
   ↓
Bounded repair or rollback
   ↓
New evidence on an immutable revision
```

Each attempt **must** attach:

| Field | Role |
|-------|-----|
| Concrete task | what was intended |
| Base hash | `sha256(source + lock + policy + emitted-rust)` |
| Small reversible diff | 1 cause, not a mass rewrite |
| Structured diagnostics | ARI-* JSON |
| Cause hypothesis | before the diff |
| Target checks | verify plan |
| Attempt budget | e.g. max 3 / same cause |
| Final decision | pass / repair / rollback / blocked / needs_human |

## 3. Correction pyramid (cheap → expensive)

Repair first with the most deterministic feedback; climb a level **only** if the previous one passes.

| Level | Tool | What it finds | Reaction |
|------|-------------|--------------|----------|
| 0 | ARITA parser | grammar / AST | only the syntactic construct |
| 1 | Resolver | imports, symbols, nonexistent APIs | real catalog; **do not** invent a name |
| 2 | Type checker | types, `Result`, incomplete match | contract / conversion / typed handling |
| 3 | Ownership / effects | move, borrow, capability | restructure scope; no clone/permission without policy |
| 4 | Emit + `cargo check` | lowering, traits, async, real borrow | fix **IR/emitter**; do not hand-patch Rust |
| 5 | Clippy / policy | dangerous patterns | allowed repair or reject the design |
| 6 | Unit / integration / acceptance | observable regression | fix production; **do not** weaken expected |
| 7 | Properties / fuzz | edges / hostile input | invariants, validation, bounds |
| 8 | Kani / Verus / Creusot | bounded/deductive properties | model/code; **no** `assume` escape |
| 9 | Runtime / canary | operational | trace, reproduce, regression |

**Rule:** level-4 success does **not** paper over a level-6 failure.

### Queue priority

`P0` soundness/unsafe/policy → `P1` parse/resolve/types/ownership → `P2` build emit → `P3` acceptance → `P4` fuzz/formal → `P5` lints.

## 4. Structured diagnostic (example)

Do not send only the `rustc` text. Normalize to ARITA:

```json
{
  "revision": "sha256:base",
  "stage": "ownership",
  "diagnostic": {
    "code": "ARI-OWN-002",
    "category": "borrow_conflict",
    "severity": "error",
    "primary_span": {
      "file": "routes.arita",
      "start_line": 47,
      "start_column": 5,
      "end_line": 47,
      "end_column": 31
    },
    "message": "You cannot mutate `routes` while an active read borrow exists.",
    "cause": {
      "kind": "shared_borrow",
      "origin_span": {
        "file": "routes.arita",
        "start_line": 39,
        "start_column": 18
      }
    },
    "semantic_rule": "exclusive_mutation",
    "allowed_repairs": [
      "reduce_borrow_scope",
      "copy_required_fields_before_mutation",
      "reorder_read_then_write",
      "use_transactional_update"
    ],
    "forbidden_repairs": [
      "unsafe_escape",
      "suppress_diagnostic",
      "implicit_unbounded_clone",
      "change_test_expectation"
    ]
  }
}
```

The agent gets: **violated rule · allowed transforms · forbidden ones · location · evidence** — not “make it compile”.



## 4.1 Diagnostic envelope (agent channel)

Two channels: (1) a short, human-readable one with snippets; (2) **typed JSON/CBOR** — the agent does **not** parse terminal ANSI.

### Envelope

`schema_version: arita.diagnostics.v1` · `revision` (source/policy/lock/compiler hashes) · `stage` · `status` · `summary` (errors, warnings, blocked_by_policy, cascade_errors_suppressed) · `diagnostics[]` · `repair_context` · `next_action` (`repair_allowed` | `needs_dependency_approval` | `needs_human` | …).

### Individual diagnostic (required fields)

On top of §4: `id`, `blocking`, byte+line spans, `related[]`, `semantic` (rule, hir_node, types, control_flow), `cause_class`, `allowed_repair_classes` / `forbidden_repair_classes`, `examples[]` (canonical shape), `validation_required[]`.

Cascades: `cascade_errors_suppressed` avoids spending attempts on symptoms.

### Evidence with authorized claims

`schema_version: arita.evidence.v1` · checks with status/duration/bounds ·  
`claims_permitted` (e.g. `compiled`, `tested`, `bounded_verified_parser`) ·  
`claims_forbidden` (e.g. `fully_verified`) — the agent may **only** assert permitted claims.

## 5. Repair taxonomy (closed set)

| Class | Usual cause | Safe repair | Anti-pattern |
|-------|----------------|---------------|-------------|
| Unresolved symbol | Invented API / typo / missing dep | catalog/bindings; real symbol or request dep | invent a plausible name |
| Incompatible type | domain / signature | validated conversion; adapt API | `as`, blind stringify/parse |
| Untreated `Result` | ignored failure | `?` / `match` / typed recover | `unwrap` / `expect` / silent default |
| Missing `Option` | unguaranteed value | `match` / `ok_or` / justified fallback | assume `Some` |
| Non-exhaustive match | forgotten variant | explicit arms + neg tests | hiding `_ => default()` |
| Overflow / division | missing precondition | checked + error/bound | silent wrap |
| Borrow / move | aliasing / lifetime | shrink scope; redesign API | clone arbitrarily |
| Trait / bound | bad interop choice | compatible binding / declared adapter | generic bound “by luck” |
| Async / `Send` | guard across await | close the guard; actor/channel | `spawn_local` as a general patch |
| Failed test | logic or ambiguous requirement | fix production + regression | change expected without justification |
| Fuzz crash | unforeseen input | validate / bound / invariant | silently filter the corpus |
| Kani counterexample | false property / edge | reproduce as a test; fix | lower `unwind` / arbitrary `assume` |
| Dep / policy | outside allowlist | explicit request + review | silently edit the manifest |

## 6. Controller phases

### 6.1 Reproduce before editing

```text
revision = sha256(source + arita.lock + policy + emitted-rust)
diagnostic_set = verify(revision)
```

If it does not reproduce → environment/cache/flake/misalignment; **do not** edit.

### 6.2 Locate the root cause

Do not lump 100 errors. Typical cascades: parse blocks resolve; symbols block types; types yield false borrows; the emitter produces Rust diagnostics that do **not** exist in ARITA; a test may fail due to the specification, not the code.

### 6.3 Hypothesis before the diff

```json
{
  "diagnostic": "ARI-OWN-002",
  "hypothesis": "The shared borrow of `routes` lives longer than needed; only `route_id` is required.",
  "proposed_change": "Extract an owned `route_id` and end the borrow before `routes.insert`.",
  "expected_effect": "Remove the mutable/shared conflict without cloning the collection.",
  "affected_invariants": [
    "route_id identifies the route",
    "insert preserves ID uniqueness"
  ],
  "verification_plan": [
    "arita check routes.arita",
    "cargo check",
    "cargo test -p routes",
    "property test duplicate_ids"
  ]
}
```

The LLM proposes; the controller **accepts or rejects** per allowed repairs.

### 6.4 A single semantic diff

Limits:

- 1–3 production files / attempt
- One semantic module
- **Do not** change production code and locked acceptance criteria at the same time
- **No** policy / dependency / capability changes without approval
- **Do not** delete tests / reduce coverage to “fix” things
- Reject escapes: `unsafe`, `unwrap`, `expect`, `todo!`, `unimplemented!`, `#[allow]`, `cfg` that hides paths

### 6.5 Revalidate from the bottom

```text
ARITA parse → resolve → type → ownership/effects
  → emit Rust → cargo check → target tests
  (+ property / Kani if applicable)
```

Results linked to the **post-patch hash**.

### 6.6 Stop and escalate → human

Stop if: >N attempts on the same cause (e.g. 3) · the specification changes · permissions/deps widen · two semantically distinct repairs without evidence · business ambiguity · bounds/architecture degrade · touches FFI/crypto/auth/migration/deploy · forbidden repair.

Honest status: `blocked_by_ambiguity` — **never** “implemented”.

## 7. Structural ARITA feedback (examples)

### Hallucinated API (`ARI-RES-014`)

`http.fetch_json` nonexistent → list the real binding + typed candidates (`http.get`, `json.decode`) + suggested repair. **Forbidden** to guess another method.

### Partial access (`ARI-TOT-003`)

`users[index]` → require `users.get(index)?` or a refined index. Teaches the canonical shape (aligned with ADR-227).

### Ownerless task (`ARI-TASK-007`)

Loose `spawn` under `service-safe` → `await` / `group.spawn` / `supervisor.spawn`; detach forbidden.

## 8. Deterministic before LLM

| Pattern | Auto-repair possible | Condition |
|--------|---------------------|-----------|
| Unused import | remove | no public reexport |
| Format | `arita fmt` | always |
| Match with a new variant | stub that **blocks** compilation | never mark resolved |
| Literal → unambiguous type | typed constructor | no overflow |
| Obviously omitted `?` | suggest/apply | signature already admits the error |
| Unnecessary temporary borrow | shorten scope | AIR proves local equivalence |
| Clippy style | `--fix` in sandbox | reviewable diff; no new `allow` |

LLM only for **semantic** decisions.

## 9. Compiler as teacher

Knowledge base per code:

```text
diagnostic code
  → violated semantic rule
  → valid program shapes
  → minimal examples
  → policy constraints
  → mandatory tests/gates
```

Example `ARI-ERR-005` (unconsumed `Result`): valid shapes `?` / `match` / `recover`; invalid `unwrap` / `default` / `let _ =`; requires an error case in tests.

## 10. Post-patch controls

| Axis | Questions |
|-----|-----------|
| Task | Acceptance intact? Expected touched? Bypass? Public API changed? |
| Policies | unsafe/panic/unwrap/dep/capability/timeout/size? Hidden `allow`/`cfg`? |
| Evidence | On the final hash? Tests actually run? Kani bounds equal or higher? New assumptions listed? |

## 11. Repair Controller architecture

```text
        Task + specification
                  │
                  ▼
           Planner LLM
                  │
                  ▼
           Patch candidate
                  │
   ┌──────────────┴──────────────┐
   │ ARITA Repair Controller      │
   │ - revision store             │
   │ - policy / permission        │
   │ - diagnostic normalizer      │
   │ - root-cause grouper         │
   │ - repair-rule selector       │
   │ - diff budgeter              │
   │ - validation planner         │
   │ - rollback engine            │
   │ - evidence recorder          │
   └──────────────┬──────────────┘
        │         │         │
        ▼         ▼         ▼
  ARITA check  Cargo/Clippy  Tests/Kani/fuzz
        │         │         │
        └──── evidence + diagnostics ────┘
                       │
                       ▼
         pass / repair / rollback / blocked
```



## 11.1 Extended architecture (rev. 1.2)

```text
Task / SPEC / acceptance
        ↓
LLM planner → candidate patch
        ↓
┌─────────────────────────────────────────────┐
│ Repair Controller                            │
│ Revision store · Policy gate · Diff budget   │
│ Diagnostic normalizer → Root-cause grouper   │
│ → Strategy retrieval ← Repair-memory store   │
└─────────────────────────────────────────────┘
   ↓              ↓              ↓
ARITA check   Rust/Cargo     Tests/Fuzz/Kani
   └──────── Evidence aggregator ────────┘
                    ↓
     PASS / RETRY / ROLLBACK / BLOCKED
```

Components must **not**: self-certify (LLM), edit out of scope, overwrite the last good state, treat warnings as success, pass unstructured logs, authorize escapes, alter policy, declare done without tests, turn `tested`→`proved`, silently restart the loop.

### State machine

```text
IDLE → PLAN → PATCH_PROPOSED → POLICY_CHECK
  ├─ reject → BLOCKED
  └─ allow → SANDBOX_VALIDATE
       ├─ pass → ACCEPTED
       ├─ fail_repairable → DIAGNOSE → RETRIEVE_STRATEGY → PATCH_PROPOSED
       ├─ fail_nonrepairable → BLOCKED
       └─ environment_failure → RETRY_ENVIRONMENT
```

`ACCEPTED` requires **all** profile gates. Green `cargo check` ≠ `ACCEPTED` if acceptance is missing.

### Pseudocode (core)

```text
while may_continue:
  evidence = validate(candidate, required_checks)
  if all_required_pass → accept
  diagnosis = normalize_and_group(failures)
  if needs_human or is_stuck or risk_over_budget → block
  strategies = retrieve_allowed(diagnosis, context, policy, memory)
  proposal = llm_minimal_patch(..., strategies, diff_budget)
  if !policy_allows → record_rejected; continue
  candidate = apply_in_sandbox(proposal)
→ block_on_budget
```

What matters: `may_continue`, `policy_allows`, `all_required_checks_pass`, `is_stuck` — not the `while`.

## 12. Target CLI: `arita repair`

```bash
arita repair \
  --task TASK-142 \
  --profile service-safe \
  --max-attempts 3 \
  --scope crates/proxy-routes \
  --require acceptance,types,ownership,cargo,test \
  --deny unsafe,panic,dependency-change,policy-change
```

### Pass output (example)

```text
Attempt 3: Status: pass
Evidence: ARITA check PASS · emit 0 unsafe · cargo check PASS ·
  clippy 0 warnings · acceptance 7/7 · properties 10k PASS
Residual: URL parser fuzzed 5m; no formal proof of normalization.
```

### Blocked output (example)

```text
Status: BLOCKED
Reason: the requirement does not specify whether http:// is valid or HTTPS must be required.
No irreversible change was applied.
```

## 13. Golden rule

\[
\text{Accepted change} =
\text{explained cause}
\land \text{allowed repair}
\land \text{requirements preserved}
\land \text{new evidence on the final artifact}
\]



## 15. Budgets, progress, and stagnation

`max_iterations=10` is not enough. Budget is **multidimensional**:

| Budget | Initial example |
|-------------|-----------------|
| Iterations | 3 / root cause; 8 / task |
| Diff | ≤200 lines; ≤3 prod files / attempt |
| Tokens / model cost | fixed cap / task |
| Wall clock | 20 min local; 60 min CI |
| CPU/mem verifiers | 4c/8GiB; 10 min / checker |
| Test execution | 10k properties; fuzz 5 min PR / 1 h nightly |
| Risk | 0 without approval: network, secrets, migrations, FFI |
| Regression | do not increase net errors |
| Stagnation | ≤2 patches without reducing root cause |

On hitting a ceiling: deliver status + reason; **never** auto-raise the limit or silently restart.

### Progress vector

\[
P = (E_{\mathrm{root}}, E_{\mathrm{policy}}, E_{\mathrm{compile}}, E_{\mathrm{acceptance}}, E_{\mathrm{regression}}, D_{\mathrm{diff}})
\]

Lexicographic progress: lower root → policy does not rise → acceptance does not drop → no new regressions → diff within budget.

`A→B→A` cycle or same `root_cause_fingerprint` → `BLOCKED_STAGNATION` (hash: revision + root_cause + validation_summary + policy_delta).

### Exit states

| State | Meaning |
|--------|-------------|
| `accepted` | all profile gates on the final revision |
| `partial` | compiles but evidence missing; **not** “done” |
| `blocked_ambiguity` | insufficient requirement/semantics |
| `blocked_policy` | only repair violates security |
| `blocked_budget` | time/attempts/cost exhausted |
| `blocked_stagnation` | cycle / no progress |
| `blocked_environment` | toolchain/network/creds |
| `needs_human_review` | deps, permissions, FFI, migration, public API |
| `rollback` | regression; return to last healthy |

Human handoff: concrete question · attempts · last healthy · diagnosis · options A/B/C · do not touch acceptance/policy.

## 16. Implementation order (repair)

1. Stable JSON diagnostics (codes, cause, spans, repairs, checks)  
2. Snapshots + sandbox + rollback  
3. Controller with budget + stagnation  
4. Manual library of 20–30 templates  
5. Memory ranked by evidence  
6. LLM only for non-mechanical semantic decisions  
7. Golden promotion → rules/tests/deterministic  
8. Finetuning **only** with thousands of golden pairs  

\[
\text{Useful autonomy} =
\text{model} + \text{structured compiler} +
\text{validated memory} + \text{gates}
- \text{escapes} - \text{unlimited retries}
\]

## 14. DOC checklist

- [x] Structured oracle (no rustc paste)
- [x] Pyramid 0–9 + P0–P5
- [x] Diagnostic JSON + hypothesis
- [x] Taxonomy + anti-patterns
- [x] Diff budget + escalate
- [x] ARI-RES / TOT / TASK examples
- [x] Deterministic before LLM
- [x] Controller + CLI `arita repair`
- [x] Repair memory + ranking + templates + promotion
- [x] Envelope diagnostics.v1 + evidence claims
- [x] Budgets / progress / stagnation / exit states
- [ ] IMPL post-GO restart (ADR-231)
