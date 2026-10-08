[Español](PACK-F3-FEWSHOT.md) | English

# PACK F3 — Few-shots for AIs (logic island)

- **Estado:** **final** (GO Ingeniero Rust, 2026-09-13)
- **Fecha:** 2026-09-13
- **CUT surface DOC:** `LOGIC-ISLAND-DOC-20260913` (`DOC/ADR/012-logic-island-v0.md`)
- **Engine:** `LOGIC-ENGINE-V0` / `crates/arita-logic` (**landed**)
- **Barra:** solo programas reales bajo `ejemplos/f3/`. skip ≠ PASS. Sin theater. **Cero IDNI**.
- **ADR-099:** `f3-11`/`f3-12` anti partial-PASS **verified** Logic `f3_` **12/12** (E0301; evidencia `F3_UNSAT_FAIL_NEXT_EVIDENCE.md`).
- **Separado** de `PACK-F1.1-FEWSHOT.md` y `PACK-F2-FEWSHOT.md`. Default IA = F1.1/F2; cargar este PACK **solo** si la tarea pide lógica F3.

## Usage

Paste the **system prompt** + few-shots when the task asks for `spec`/`fact`/`rule`/`query`.
The `.arita` blocks are copies of real oracles.

```bash
# query OK (satisfiable)
cargo run -p arita-cli -- logic ejemplos/f3/01-path-ok.arita
# exit 0

# query fail → rejected E0301
cargo run -p arita-cli -- logic ejemplos/f3/02-path-fail.arita
# exit ≠ 0; stderr contains E0301
```

## System prompt (canonical, English for the model)

```text
You write ARITA F3 logic-only modules (isla lógica propia). Not Rust. Not Tau/TML APIs.

v0 file shape (separate from F1/F2 — NO fn/main/test/print in the same file):
  module <ident>
  spec <Name> { }          // optional container; may be empty
  fact pred(args...)
  rule Head :- BodyAtom, ...
  query Atom

Rules:
- Logic XOR systems: never mix fn/main/test/print with spec/fact/rule/query in one file (ADR-012 pin).
- Positive Datalog-style v0: facts + rules + queries. No IDNI/Tau runtime APIs.
- query success → ok; query fail / unsat-ish → rejected (E0301). Never invent PASS. skip ≠ PASS. Never inconclusive→accepted.
- Engine: arita logic (arita-logic crate). arita-syntax F1/F2 stays untouched.

Canonical oracles:
  ejemplos/f3/01-path-ok.arita           → logic OK (path a→c)
  ejemplos/f3/02-path-fail.arita         → E0301 query failed (path c→a)
  ejemplos/f3/11-multi-query-fail.arita  → E0301 (any query fail → whole module fail; no partial-PASS)
  ejemplos/f3/12-join-miss-fail.arita    → E0301 (join miss / no closure)

When asked for F3, output a complete logic-only .arita matching these patterns.
If the task is F1.1 or F2, use PACK-F1.1 / PACK-F2 instead.
```

## E03xx codes (summary)

| Code | English (canonical) |
|------|---------------------|
| **E0301** | `query failed` |
| **E0302** | `unsatisfiable / contradiction` |
| **E0303** | `unknown predicate or arity` |
| **E0304** | `logic construct not allowed in F3 v0` |

(See ADR-012 § E03xx; emitted by `arita-logic`.)

## Few-shot 1 — path OK (`ejemplos/f3/01-path-ok.arita`)

**User:** ARITA F3: edges a→b, b→c; derive path; query path(a,c) should succeed.

**Assistant:**

```arita
module path_ok

spec Reachability { }

fact edge(a, b)
fact edge(b, c)

rule path(X, Y) :- edge(X, Y)
rule path(X, Z) :- edge(X, Y), path(Y, Z)

query path(a, c)
```

Path: `ejemplos/f3/01-path-ok.arita`.  
Oracle: `cargo run -p arita-cli -- logic ejemplos/f3/01-path-ok.arita` → exit 0.

## Few-shot 2 — path fail (`ejemplos/f3/02-path-fail.arita`)

**User:** Same graph; query path(c,a) must fail (no reverse edges).

**Assistant:**

```arita
module path_fail

spec Reachability { }

fact edge(a, b)
fact edge(b, c)

rule path(X, Y) :- edge(X, Y)
rule path(X, Z) :- edge(X, Y), path(Y, Z)

query path(c, a)
```

Path: `ejemplos/f3/02-path-fail.arita`.  
Oracle: `arita logic` → **rejected** with **E0301** (`query failed`). This is a real negative oracle, not a PASS.

## Few-shot 3 — anti partial-PASS (ADR-099)

**Pin:** if **any** query fails in the same module run → whole module **E0301**. Never claim PASS because another query sat.

| Oracle | Path | Expect |
|--------|------|--------|
| `f3-11-multi-query-fail` | `ejemplos/f3/11-multi-query-fail.arita` | rejected **E0301** |
| `f3-12-join-miss-fail` | `ejemplos/f3/12-join-miss-fail.arita` | rejected **E0301** |

Measure: `cargo test -p arita-cli f3_` → **12/12** (evidence `F3_UNSAT_FAIL_NEXT_EVIDENCE.md`).

## Counter-examples (do NOT emit)

| Bad idea | Why |
|----------|-----|
| `fn main` / `print` / `test` in same file as `fact` | violates separate logic modules (ADR-012) |
| Tau/TML / IDNI APIs as runtime | ADR-002 / licenses — forbidden |
| Claim PASS when query fails | theater; must be E0301 rejected |
| Partial-PASS (one query OK, another fail) | theater; ADR-099 → whole module E0301 |
| Empty queries / skip → accepted | skip ≠ PASS; inconclusive ≠ accepted |
| Invent predicates without facts/rules and expect SAT | likely E0303 |

## Checklist GO (Rust Engineer) — closed

- [x] System prompt F3 OK (separate from F1/F2; no IDNI)
- [x] Few-shots = real `ejemplos/f3/01`–`02` (+ ADR-099 `11`/`12` anti partial-PASS)
- [x] E03xx table OK
- [x] Counter-examples / no fake PASS OK
- [x] GO → status **final** + Docs index

## Links

- `DOC/ADR/012-logic-island-v0.md`, `DOC/ADR/002-isla-logica-propia.md`
- `DOC/08-LICENSES-IDNI.md`, `DOC/THREAT_MODEL.md`
- `ejemplos/f3/01-path-ok.arita`, `ejemplos/f3/02-path-fail.arita`, `ejemplos/f3/11-multi-query-fail.arita`, `ejemplos/f3/12-join-miss-fail.arita`
- `DOC/ADR/099-f3-unsat-fail-next.md`, `F3_UNSAT_FAIL_NEXT_EVIDENCE.md`
- `DOC/PACK-F1.1-FEWSHOT.md`, `DOC/PACK-F2-FEWSHOT.md` (other surfaces)
