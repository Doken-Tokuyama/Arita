# F3 logic ejemplos (positive Datalog island)

ADR-012 / CUT LOGIC-ENGINE-V0. Logic-only modules: `module` / optional empty `spec` / `fact` / `rule` / `query`.
No `fn` / `main` / `test`. Evaluated by `arita logic` via `arita-logic` (real fixpoint — not rustc, not always-SAT).

## Files

| Archivo | Qué cubre | Engine outcome |
|---------|-----------|----------------|
| `01-path-ok.arita` | `edge` + transitive `path` | **sat** (exit 0, prints `true`) |
| `02-path-fail.arita` | same rules; unreachable query `path(c, a)` | **E0301** (exit 1) |
| `03-ancestor-ok.arita` | `parent` + transitive `ancestor` (genealogy, not path-rename) | **sat** |
| `04-sibling-fail.arita` | same-generation `sibling` via shared `parent`; alice/charlie do not share | **E0301** |
| `05-edge-ok.arita` | bare `edge` fact query (no rules) | **sat** |
| `06-unknown-pred.arita` | query unknown predicate | **E0303** |
| `07-forbidden-fn.arita` | `fn` in a logic-only file | **E0304** |
| `08-arity-mismatch.arita` | arity conflict on `edge` | **E0303** |
| `09-twohop-ok.arita` | non-transitive binary join `hop2(X,Z) :- edge(X,Y), edge(Y,Z)` | **sat** |
| `10-cycle-reach-ok.arita` | directed cycle `edge(a,b)`/`edge(b,a)` + recursive `reach`; `reach(a,a)` | **sat** |
| `11-multi-query-fail.arita` | path sat + path fail in same file (anti partial-PASS) | **E0301** |
| `12-join-miss-fail.arita` | positive join miss (alice/bob different depts) | **E0301** |

## Measure oracles (`LOGIC_ORACLES` in `crates/arita-cli/src/measure.rs`)

| Oracle id | Path | `expect_sat` | Semantics |
|-----------|------|--------------|-----------|
| `f3-01-path-ok` | `ejemplos/f3/01-path-ok.arita` | `true` | **accepted** iff engine sat (all queries hold) |
| `f3-02-path-fail` | `ejemplos/f3/02-path-fail.arita` | `false` | **accepted** iff engine rejects with **E0301** (NEG-style) |
| `f3-03-ancestor-ok` | `ejemplos/f3/03-ancestor-ok.arita` | `true` | same as f3-01 |
| `f3-04-sibling-fail` | `ejemplos/f3/04-sibling-fail.arita` | `false` + code `E0301` | same as f3-02 |
| `f3-05-edge-ok` | `ejemplos/f3/05-edge-ok.arita` | `true` | bare fact query sat |
| `f3-06-unknown-pred` | `ejemplos/f3/06-unknown-pred.arita` | `false` + code `E0303` | unknown predicate |
| `f3-07-forbidden-fn` | `ejemplos/f3/07-forbidden-fn.arita` | `false` + code `E0304` | `fn` forbidden in logic island |
| `f3-08-arity-mismatch` | `ejemplos/f3/08-arity-mismatch.arita` | `false` + code `E0303` | arity conflict |
| `f3-09-twohop-ok` | `ejemplos/f3/09-twohop-ok.arita` | `true` | non-transitive `hop2` join sat |
| `f3-10-cycle-reach-ok` | `ejemplos/f3/10-cycle-reach-ok.arita` | `true` | cycle self-reach `reach(a,a)` sat |
| `f3-11-multi-query-fail` | `ejemplos/f3/11-multi-query-fail.arita` | `false` + code `E0301` | multi-query: any fail → whole module E0301 |
| `f3-12-join-miss-fail` | `ejemplos/f3/12-join-miss-fail.arita` | `false` + code `E0301` | join miss different depts → E0301 |

Field note: the struct uses `expect_sat` (not a separate `expect_fail`).  
`expect_sat: false` **is** the expect-fail / NEG-style flag (`code` must appear, typically `E0301`).

### Anti-theater (skip ≠ PASS)

- **Missing file** → measure verdict **inconclusive** (never accepted).
- **Skip / empty queries / not all_sat** → **rejected** (never accepted as pass).
- **Accidental sat** on an expect-fail oracle (`expect_sat: false`) → **rejected**.
- Real `arita_logic::eval_file` only — no theater stubs.

```bash
cargo run -p arita-cli -- logic ejemplos/f3/03-ancestor-ok.arita      # exit 0
cargo run -p arita-cli -- logic ejemplos/f3/04-sibling-fail.arita     # exit 1 + E0301
cargo run -p arita-cli -- logic ejemplos/f3/09-twohop-ok.arita        # exit 0
cargo run -p arita-cli -- logic ejemplos/f3/10-cycle-reach-ok.arita   # exit 0
cargo run -p arita-cli -- logic ejemplos/f3/11-multi-query-fail.arita # exit 1 + E0301
cargo run -p arita-cli -- logic ejemplos/f3/12-join-miss-fail.arita   # exit 1 + E0301
cargo test -p arita-cli f3_
```

Measure: `f3-01`…`f3-12` in `LOGIC_ORACLES` (ADR-099: 11 multi-query anti partial-PASS, 12 join-miss).
