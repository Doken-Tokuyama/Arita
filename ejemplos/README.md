# Ejemplos ARITA (F1.1 + F2 + F2.1 + F2.2 + F2.3 + F3 + async + perf + deps + host + bootstrap + contracts + attest + result)

**Core 0.2:** ver [`core02/`](core02/) (README dual IA/humano; pins ADR-233).

Programas mínimos del surface congelado en `DOC/ADR/005-f1.1-surface-freeze.md`
y el slice F2 (`let Int` / `String` / `Bool` / `Vec<Int>` / Int binary / multi-fn + Call / `print` Path|LitInt|LitStr|LitBool|len|user_call / `test` + `assert`) de ADR-006 / ADR-007 / CUT `F2-AST-20260913`.

## Surface permitido

```arita
module <ident>

fn name(x: Int, …) -> Int {
  x + x   // Expr binary | path (no print)
}

fn main() -> Io<()> {
  print("<lit-str>")
  // F2:
  let [mut] name: Int = <int-lit | binary | ident>
  let [mut] name: String = "<lit-str>"
  let [mut] name: Bool = true | false
  let [mut] name: Vec<Int> = Vec::new()
  name.push(<int-lit | ident>)
  print(<ident> | <int-lit> | <bool_lit> | <ident>.len() | name(args…))
  // binary (let init): (ident|int_lit) (+|-|*|/|%) (ident|int_lit)
}

test <ident> {
  assert <binary|call|int|ident> == <binary|call|int|ident>
}
```

## Cómo compilar

Desde la raíz del repo (`<repo>`):

```bash
cargo run -p arita-cli -- build ejemplos/01-hello.arita
./target/arita-out/hello
```

Esperado: binario en `target/arita-out/<module>` y salida del programa según los `print`.

## Lista

| Archivo | Qué cubre | stdout esperado |
|---------|-----------|-----------------|
| `01-hello.arita` | smoke mínimo | `hello` |
| `02-hello-lines.arita` | varios `print` | `line one` / `line two` / `line three` |
| `03-greet.arita` | otro `module` name | `hola ARITA` |
| `04-multi-print.arita` | multi-print extra | `one` / `two` / `three` |
| `05-with-comments.arita` | comentarios `//` | `alpha` / `beta` |

### F2 (`ejemplos/f2/`)

| Archivo | Qué cubre | stdout esperado |
|---------|-----------|-----------------|
| `f2/01-let-int.arita` | `let`/`let mut` Int + `print(Path)` | `42` / `7` |
| `f2/02-vec-len.arita` | `Vec::new` / `push` / `print(len)` | `2` |
| `f2/03-string.arita` | `let` String + `print(Path)` | `hi` |
| `f2/04-int-arith.arita` | Int binary `a + b` + print | `5` |
| `f2/05-bool.arita` | `let` Bool + `print(Path)` | `true` |
| `f2/06-fn-call.arita` | helper `fn` + `print(user_call)` | `42` |
| `f2/07-assert.arita` | `test` + `assert` (cfg(test) only) | `ok` (`arita test` → PASS) |
| `f2/08-string-len.arita` | `String.len()` bytes → Int (ADR-026) | `2` |
| `f2/09-is-empty.arita` | `Vec`/`String`.`is_empty()` → Bool | `true` / `false` / `false` |


### Bootstrap (`ejemplos/bootstrap/`) — ADR-037 / ADR-042

| Archivo | Qué cubre | stdout esperado |
|---------|-----------|-----------------|
| `bootstrap/01-fact.arita` | dual-oracle factorial-lite `fact5` (unrolled 1‥5) | `120` |
| `bootstrap/02-sum.arita` | dual-oracle sum-lite `sum5` (unrolled 1‥5) — ADR-042 | `15` |

Golden SoT: `fact5` / `GOLDEN_FACT5` (ADR-037); `sum5` / `GOLDEN_SUM5` (ADR-042). Measure ids: `bootstrap-01`, `bootstrap-02`.

### Result (`ejemplos/result/`) — ADR-047 / ADR-048

| Archivo | Qué cubre | stdout / diagnóstico |
|---------|-----------|----------------------|
| `result/01-ok.arita` | `Result<Int,String>` Ok path + exhaustive match | `42` |
| `result/02-err.arita` | Err path + exhaustive match | `fail` |
| `result/03-swallow-ok.arita` | Err(e)+print(e) — no E0272 | `1` |
| `result/neg/e0270-nonexhaustive.arita` | match solo `Ok` → **E0270** | `non-exhaustive result match` |
| `result/neg/e0272-err-swallow.arita` | Err(e)=>`{ 0 }` → **E0272** | `result error swallowed` |
| `result/neg/e0272-err-underscore.arita` | Err(_)=>`{ 0 }` → **E0272** | `result error swallowed` |

Measure ids: `result-01`, `result-02`, `result-swallow-ok`, `neg-e0270`, `neg-e0272`, `neg-e0272-underscore`. OUT v0: `?` / unwrap / map / Option / Mutex.

### F2 neg (`ejemplos/f2/neg/`) — must FAIL parse/build

Ownership **E020x** (HIR check v0, ADR-006 / ADR-009) + anti-theater **E021x**. See `ejemplos/f2/neg/README.md`.

| Archivo | Código | Mensaje canónico |
|---------|--------|------------------|
| `f2/neg/e0201-use-after-move.arita` | E0201 | `use of moved value` |
| `f2/neg/e0202-double-mut.arita` | E0202 | `borrow conflict` |
| `f2/neg/e0210-todo.arita` | E0210 | `todo/unimplemented not allowed` |
| `f2/neg/e0211-assert-true.arita` | E0211 | `assert requires evidence` |
| `f2/neg/e0212-empty-test.arita` | E0212 | `empty test not allowed` |
| `f2/neg/e0213-empty-fn.arita` | E0213 | `stub function body not allowed` |
| `f2/neg/e0231-unsafe.arita` | E0231 | `unsafe / FFI not allowed in ARITA` |
| `f2/neg/e0206-bad-method.arita` | E0206 | `method not in F2 std whitelist` |

```bash
cargo run -p arita-cli -- parse ejemplos/f2/neg/e0201-use-after-move.arita  # FAIL E0201
cargo run -p arita-cli -- parse ejemplos/f2/neg/e0202-double-mut.arita      # FAIL E0202
cargo run -p arita-cli -- parse ejemplos/f2/neg/e0210-todo.arita            # FAIL E0210
```

```bash
cargo run -p arita-cli -- build ejemplos/f2/01-let-int.arita
./target/arita-out/let_int

cargo run -p arita-cli -- build ejemplos/f2/02-vec-len.arita
./target/arita-out/vec_len

cargo run -p arita-cli -- build ejemplos/f2/03-string.arita
./target/arita-out/str_hi

cargo run -p arita-cli -- build ejemplos/f2/04-int-arith.arita
./target/arita-out/arith

cargo run -p arita-cli -- build ejemplos/f2/05-bool.arita
./target/arita-out/flag

cargo run -p arita-cli -- build ejemplos/f2/06-fn-call.arita
./target/arita-out/fn_call

cargo run -p arita-cli -- build ejemplos/f2/07-assert.arita
./target/arita-out/assert_demo

cargo run -p arita-cli -- test ejemplos/f2/07-assert.arita
```

Negativos / `E0xxx` y oráculos positivos → suite `arita measure` (cubre estos ejemplos + clippy-workspace).

Also: `examples/hello.arita` (canónico legacy path) ≡ smoke hello.

### F2.1 (`ejemplos/f2.1/`) — CUT `F2.1-CTRL-20260913`

| Archivo | Qué cubre | stdout esperado |
|---------|-----------|-----------------|
| `f2.1/01-if-true.arita` | `if true { print }` | `yes` |
| `f2.1/02-if-else.arita` | `if` Path-Bool + `else` | `no` |
| `f2.1/03-while-count.arita` | `while i < 3` + mut assign | `0` / `1` / `2` |
| `f2.1/04-cmp-if.arita` | Int cmp in `if` (`n < 5`) | `yes` |
| `f2.1/05-cmp-let.arita` | Int cmp as Bool let (`a == 7`) | `eq` |

### F2.1 neg — E0220 / E0226 / E0227

| Archivo | Código | Mensaje |
|---------|--------|---------|
| `f2.1/neg/e0220-bad-cond.arita` | E0220 | `if/while condition type ≠ Bool` |
| `f2.1/neg/e0226-while-false.arita` | E0226 | `vacuous while false` (ADR-040; measure-wired) |
| `f2.1/neg/e0226-while-false-empty.arita` | E0226 | companion empty-body (not measure-wired) |
| `f2.1/neg/e0227-if-false.arita` | E0227 | `vacuous if false` (ADR-041; measure-wired) |
| `f2.1/neg/e0227-if-false-empty.arita` | E0227 | companion empty-body (not measure-wired) |
| `f2.1/neg/e0227-if-false-assert.arita` | E0227 | companion assert morph (not measure-wired) |


### F2.2 (`ejemplos/f2.2/`) — CUT `F2.2-MATCH-20260913`

| Archivo | Oracle id | Qué cubre | stdout esperado |
|---------|-----------|-----------|-----------------|
| `f2.2/01-match-bool.arita` | `f22-01-match-bool` | `match` Bool true/false | `yes` |
| `f2.2/02-match-int.arita` | `f22-02-match-int` | `match` Int lit + `_` | `a` |
| `f2.2/03-match-bool-wild.arita` | `f22-03-match-bool-wild` | Bool exhaustivo vía `_` | `other` |

### F2.2 neg — E0221 / E0222 / E0223 / E0225

| Archivo | Código | Mensaje |
|---------|--------|---------|
| `f2.2/neg/e0221-bool-nonex.arita` | E0221 | `non-exhaustive Bool match` |
| `f2.2/neg/e0222-int-no-wild.arita` | E0222 | `Int match requires `_` arm` |
| `f2.2/neg/e0223-pat-mismatch.arita` | E0223 | pattern/scrutinee type mismatch |
| `f2.2/neg/e0225-match-bool-same.arita` | E0225 | `vacuous match arms` (Bool same const) |
| `f2.2/neg/e0225-match-int-same.arita` | E0225 | `vacuous match arms` (Int same const) |

See `ejemplos/f2.2/neg/README.md`. ADR-015 / ADR-031; PACK `DOC/PACK-F2.2-FEWSHOT.md`.



### F2.3 (`ejemplos/f2.3/`) — break / continue (ADR-016)

| Archivo | Qué cubre | stdout esperado |
|---------|-----------|-----------------|
| `f2.3/01-while-break.arita` | `while` + `break` after printing 0,1 | `0` / `1` |
| `f2.3/02-while-continue.arita` | `while` + `continue` skip odds | `0` / `2` / `4` |

### F2.3 neg (`ejemplos/f2.3/neg/`)

| Archivo | Código | Mensaje canónico |
|---------|--------|------------------|
| `f2.3/neg/e0224-break-outside.arita` | E0224 | `break/continue outside while` |

### F3 (`ejemplos/f3/`) — logic island (ADR-012)

| Archivo | Oracle id | Engine | Measure (`expect_sat`) |
|---------|-----------|--------|------------------------|
| `f3/01-path-ok.arita` | `f3-01-path-ok` | sat | `true` → accepted iff sat |
| `f3/02-path-fail.arita` | `f3-02-path-fail` | E0301 | `false` → accepted iff E0301 |
| `f3/03-ancestor-ok.arita` | `f3-03-ancestor-ok` | sat | `true` |
| `f3/04-sibling-fail.arita` | `f3-04-sibling-fail` | E0301 | `false` + E0301 |
| `f3/05-edge-ok.arita` | `f3-05-edge-ok` | sat | `true` (bare fact) |
| `f3/06-unknown-pred.arita` | `f3-06-unknown-pred` | E0303 | `false` + E0303 |
| `f3/07-forbidden-fn.arita` | `f3-07-forbidden-fn` | E0304 | `false` + E0304 |
| `f3/08-arity-mismatch.arita` | `f3-08-arity-mismatch` | E0303 | `false` + E0303 |
| `f3/09-twohop-ok.arita` | `f3-09-twohop-ok` | sat | `true` (non-transitive `hop2` join) |
| `f3/10-cycle-reach-ok.arita` | `f3-10-cycle-reach-ok` | sat | `true` (cycle `reach(a,a)`) |
| `f3/11-multi-query-fail.arita` | `f3-11-multi-query-fail` | E0301 | `false` + E0301 (anti partial-PASS) |
| `f3/12-join-miss-fail.arita` | `f3-12-join-miss-fail` | E0301 | `false` + E0301 (join miss) |

See `ejemplos/f3/README.md`. Missing file → inconclusive; skip ≠ PASS; accidental sat on expect-fail → rejected. `arita logic`, not rustc.


### Contracts (`ejemplos/contracts/`) — ADR-017 / ADR-018 / ADR-019 (`LANG-CONTRACT-20260913`)

JSON contracts for `arita contract <path.json>` and measure `ContractOracle`. Language-level `contract { }` in `.arita` (ADR-019). Exactly one of `expect_stdout` / `expect_reject`. Optional JSON `source_sha256` (raw-file SHA-256 hex). Invalid schema or missing source → inconclusive (never PASS). **OUT:** signatures, notarization, nested contracts, `target: fn`, surface attest fields.

| Archivo | Oracle id | Kind |
|---------|-----------|------|
| `contracts/contract-hello.json` | `contract-hello` | expect_stdout `hello` ← `01-hello.arita` |
| `contracts/contract-f23-break.json` | `contract-f23-break` | expect_stdout `0` / `1` ← `f2.3/01-while-break` |
| `contracts/contract-neg-e0224.json` | `contract-neg-e0224` | expect_reject `E0224` |
| `contracts/contract-hello-attested.json` | `contract-hello-attested` | stdout + pinned `source_sha256` |
| `contracts/contract-hello-bad-hash.json` | `contract-hello-bad-hash` | wrong sha → reject; measure accepted iff reject |
| `contracts/lang-hello.arita` | `lang-contract-hello` | language `expect_stdout "hello"` |
| `contracts/lang-neg-e0224.arita` | `lang-contract-neg-e0224` | language `expect_reject "E0224"` |

```bash
cargo run -p arita-cli -- contract ejemplos/contracts/contract-hello.json
cargo run -p arita-cli -- contract ejemplos/contracts/lang-hello.arita
cargo run -p arita-cli -- contract ejemplos/contracts/lang-neg-e0224.arita
cargo run -p arita-cli -- attest verify ejemplos/contracts/contract-hello-attested.json
```

Measure: existing **41** + LANG-CONTRACT **+2** → **~43**. See `ejemplos/contracts/README.md`.


### Async (`ejemplos/async/`) — ADR-027 ASYNC-V0

| Archivo | Oracle id | Expect |
|---------|-----------|--------|
| `async/01-await-greet.arita` | `async-01-await-greet` | stdout `hi` then `done` |
| `async/02-async-print.arita` | `async-02-async-print` | stdout `ok` |
| `async/neg/e0240-await-outside.arita` | `neg-e0240-await-outside` | E0240 `await outside async function` |
| `async/neg/e0241-async-illegal.arita` | `neg-e0241-async-illegal` | E0241 `async feature not allowed here` |

```bash
cargo run -p arita-cli -- parse ejemplos/async/neg/e0240-await-outside.arita  # FAIL E0240
cargo run -p arita-cli -- parse ejemplos/async/neg/e0241-async-illegal.arita  # FAIL E0241
```

PACK: `DOC/PACK-ASYNC-FEWSHOT.md` (load only if the task asks for async).


### Perf (`ejemplos/perf/`) — ADR-028 PERF-V0

CLI-only `--profile debug|release` (default debug). Release emit writes `[profile.release] opt-level = 3` (LTO off). Timed thresholds OUT.

| Archivo / check | Oracle id | Expect |
|-----------------|-----------|--------|
| `perf/01-release-run.arita` | `perf-01-release-run` | stdout `release-ok` under `--profile release` |
| generated `Cargo.toml` | `perf-02-release-optlevel` | `[profile.release] opt-level = 3` |

## Deps v0 (ADR-029)

| Path | Measure id | Expect |
|------|------------|--------|
| `deps/01-tokio-bridge.arita` + `deps/arita.toml` | `deps-01-tokio-bridge` | stdout `hi` / `deps-ok`; Cargo.toml tokio `=1.53.1` |
| `deps/neg/e0260-unknown-crate/` | `deps-neg-e0260` | E0260 |
| `deps/neg/e0261-crate-path.arita` | `deps-neg-e0261` | E0261 |

| CLI `--profile fantasma` + `perf/neg/e0250-bad-profile.arita` | `perf-neg-e0250` | E0250 `unknown build profile` |

```bash
cargo run -p arita-cli -- build --profile release ejemplos/perf/01-release-run.arita
cargo run -p arita-cli -- build --profile fantasma ejemplos/perf/neg/e0250-bad-profile.arita
```


### Host-border (`ejemplos/host/`) — ADR-035

| Archivo | Qué cubre | stdout esperado |
|---------|-----------|-----------------|
| `host/01-bridge-hello.arita` + `arita.toml` | `[host-bridges]` + `host.mark()` | `host-ok` |
