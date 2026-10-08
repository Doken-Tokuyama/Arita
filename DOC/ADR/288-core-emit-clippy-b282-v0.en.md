Translation of `288-core-emit-clippy-b282-v0.md`; the original is normative. / Traducción de `288-core-emit-clippy-b282-v0.md`; el original es el normativo.

# ADR-288 — Core emit-clippy B-282 v0: `measure` oracles for B-282-1 (`unnecessary_to_owned`) and B-282-2 (`unused_parens` on `let n: Int = m.len()`)

- **Estado:** **CLOSED (Ingeniero, GO CLOSED 2026-10-03; DOC/GATE-CORE10-EMIT-CLIPPY-B282-20261003.md)** · `arita measure` 853/853 accepted (N = 853, k = 2) · historia: APROBADO Y CONGELADO (Ingeniero 2026-10-02, sha del ADR congelado `252d5dd4e6acb89c1a0cdb576e4ce3ac869f354264c9ce014067e2c04d035302` (corregido: es el sha tras el sello; el v0.1 previo al sello fue `bbf27929432ab1da3b8b5b7438404d42393d7a814c45321880fc3c8b32696e1b`)), GO IMPL tras PKG-MEMBER CLOSED; freeze `DOC/reviews/MEASURE-ADR288-EMIT-CLIPPY-FREEZE-20261003.sha` y addendum de freeze `DOC/reviews/MEASURE-ADR288-EMIT-CLIPPY-FREEZE-20261003.addendum.sha`.
- **CUT-ID:** `CORE-EMIT-CLIPPY-B282-20261001` (provisional; lo confirma el Orquestador)
- **Fecha:** 2026-10-01
- **Autores:** ARITA Arquitecto (pins) · orden y alcance decididos por el Ingeniero 01-10 (S1b → PKG-MEMBER → unused_parens/B-282 → IndexMut → Mutex)
- **Padre / contexto:** [ADR-282](282-core-map-assign-v0.md) (R3, L76/L132) · [ADR-283](283-core-vec-assign-v0.md) (FASE 2, fix de codegen) · `DOC/GATE-CORE09-MAP-ASSIGN-20260926.md` L40–42 · `DOC/reviews/NOTE_B282_STATUS_20260927.md` · `DOC/reviews/PREP_B282_ORACLES_PROPOSAL_20261001.md` (Measure, md5 `e70d197a635299bc3582568d307ec43f`) · T09-22 (`TRAPS_CORE09_SURFACE`)
- **Cierra:** **B-282-1** y **B-282-2** (CLOSED 2026-10-03; el fix de codegen ya existía y ahora está medido por los oráculos `measure` de este slice).
- **Códigos:** **ninguno** (son warnings de clippy/rustc sobre el Rust emitido, no diagnósticos E-code).
- **No reabre:** ADR-282 / 283 / 284 / 285 · E0291 / E0340–E0343 · ADR-265 / 270 / 278 · ADR-286 / 287.
- **Fuera de alcance:** **B-283-1** (general `unused_parens` in `if`/`while`/assignments/`let` with binaries, including test `lib.rs:4121` `contains("(a + b)")`), the `let c: i64 = (v.capacity() as i64);` case of `ejemplos/f2/184-shrink-to-fit-vec.arita` and the “zero rustc warnings in emitted Rust of positives” criterion → **ADR-289** (real codegen change, test 4121 migrated on its land, rustc/clippy measurement on positives, own Codegen/Measure PREP). Current order: S1b → PKG-MEMBER (287) → EMIT-CLIPPY-B282 (288) → unused_parens (289) → IndexMut → Mutex.

## 1. Facts (not hypotheses)

Data taken from `PREP_B282_ORACLES_PROPOSAL_20261001.md` and the code; no own Architect execution.

| Id | Original symptom | Fix already in codegen |
|---|---|---|
| **B-282-1** | `m.get("lit")` emitted `.get(&"lit".to_string())` ⇒ clippy `unnecessary_to_owned` | emits `m.get("lit").cloned()` only if the argument is `LitStr` and the receiver is a local `Map` binding (`crates/arita-codegen/src/lib.rs` ~L2671–2689) |
| **B-282-2** | `let n: Int = m.len()` emitted `let n: i64 = (m.len() as i64);` ⇒ rustc `unused_parens` | emits `let n: i64 = m.len() as i64;` only if the init is `MethodCall len` (~L1817–1822, `strip_one_outer_parens`) |

Estado de la cobertura hoy:
- Tests cargo (no cuentan en N): `b282_emit_clippy::b282_1_map_get_string_literal_key_is_clippy_clean` y `…b282_2_let_int_len_has_no_outer_parens_and_is_clippy_clean` (`lib.rs` L5669–5735), con `clippy-driver` en ediciones 2015 y 2021, `-D warnings -D clippy::all`.
- In `measure.rs` **no B-282 oracle exists** (`rg b282` empty). Today there is a single emit-clippy oracle: `core09-map-assign-emit-clippy` (`run_core09_map_assign_emit_clippy_oracle`, `measure.rs` ~L9705–9813), which deliberately avoided the two B-282 cases.
- Documentary inconsistency: `GATE-CORE09-REF-MUT-20260926.md` L33 already marks B-282-1/2 CLOSED; `GATE-CORE09-VEC-ASSIGN` L42 and `GATE-CORE09-SCENARIO-MUT` L28 still list them as backlog.

Conclusion: **this slice does not change codegen**. It adds the `measure` oracles and their fixtures, and leaves the two B-282s measured the same way as the rest of the gate.

## 2. Decisions (pins)

**D1 — Two oracles, one per fix (k = 2).** Ids: `core09-b282-1-map-get-lit-emit-clippy` and `core09-b282-2-let-len-emit-clippy`. No neg (no diagnosis to pin). A combined id is rejected: a failure must show which fix regressed.

**D2 — New minimal fixtures** (created by IMPL, not this ADR; core06 fixtures are not reused because they do not exercise the literal in `get`): `ejemplos/core10/emit-clippy/01-map-get-lit.arita` and `ejemplos/core10/emit-clippy/02-let-int-len.arita`, with the programs of §4. They enter the slice freeze when created.

**D3 — What each oracle measures (all three checks must hold):**
1. **Emit:** emitted Rust contains the good form and not the bad one (B-282-1: contains `m.get("a").cloned()` and does not contain `.get(&"a".to_string())`; B-282-2: contains `let n: i64 = m.len() as i64;` and `let k: i64 = v.len() as i64;`, and does not contain `(m.len() as i64)` or `(v.len() as i64)`).
2. **Clippy in two configurations (D4):** edition 2015 with `clippy-driver` and edition 2021 with `cargo clippy -- -D warnings` in a scratch crate; both clean.
3. **Execution:** the emitted binary prints exactly the expected stdout: `7` (B-282-1) and `1\n1` (B-282-2).

**D4 — Editions and tool (both mandatory).** Each oracle measures **two** configurations and both must be clean for `accepted`: (a) **edition 2015** (the real default `arita build` path: in synchronous debug the CLI invokes `rustc <file>.rs -o <bin>` without `--edition`, `crates/arita-cli/src/main.rs` L450–455 / `crates/arita-codegen/src/lib.rs` L154) measured with **`clippy-driver`** (`--edition 2015 -D warnings -D clippy::all`), like the `b282_emit_clippy` tests; (b) **edition 2021** (Cargo path: release, async, `[deps]`, host-bridges or `--target`, `main.rs` L422; emitted `Cargo.toml` carries `edition = "2021"`, `lib.rs` L1266) measured with **`cargo clippy -- -D warnings`** in a scratch crate, like `core09-map-assign-emit-clippy` (`measure.rs` ~L9705–9813). If `clippy-driver` **or** `cargo clippy` is missing ⇒ **inconclusive**, never accepted (D5). **IMPL may not reduce to a single edition/tool without returning to the Architect.**

**D5 — No clippy ⇒ inconclusive**, never accepted nor silent skip (same as the existing oracle: `measure.rs` ~L9772–9782); “no clippy” includes missing `clippy-driver` **or** `cargo clippy` (D4). Clippy present with warnings in either configuration ⇒ rejected. skip ≠ PASS.

**D6 — Existing cargo tests.** The two `b282_emit_clippy` tests in `lib.rs` stay as-is (not renamed or relaxed). Test `lib.rs:4121` is B-283-1 (confirmed by the Engineer 01-10; migration moved to ADR-289); this slice does not touch 4121 or the two `b282_emit_clippy`.

**D7 — GATE.** GATE-283/284/285 and their historical texts are not rewritten. At CLOSED an own **addendum** of this slice is created (`DOC/GATE-CORE10-EMIT-CLIPPY-B282-<fecha>.md`) that marks B-282-1/2 fixed and measured, and lists the three §1 inconsistencies (REF-MUT already CLOSED; VEC-ASSIGN and SCENARIO-MUT with historical backlog text).

## 3. Oracles and N

| Id | Fixture | Expect | stdout |
|---|---|---|---|
| `core09-b282-1-map-get-lit-emit-clippy` | `ejemplos/core10/emit-clippy/01-map-get-lit.arita` | accepted | `7` |
| `core09-b282-2-let-len-emit-clippy` | `ejemplos/core10/emit-clippy/02-let-int-len.arita` | accepted | `1\n1` |

Se registran en `run_measure_with`, junto al registro Core09 (`measure.rs` ~L16192).

k = 2. **N is not fixed in this ADR:** N is always prior N_CLOSED + k, computed by the Engineer at GO IMPL. Provisional with the current order: after PKG-MEMBER (851) ⇒ **853**; if closed directly after S1b (844) ⇒ 846. Rules: skip ≠ PASS; no oracle counts if either of D4’s two clippy configurations (2015 with `clippy-driver`, 2021 with `cargo clippy`) did not run.

## 4. Fixture programs

Fixture 01 (`01-map-get-lit.arita`), programa del test `b282_1` de `lib.rs`:

```arita
module t
fn main() -> Io<()> {
  let mut m: Map<Text, Int> = Map::new()
  m["a"] = 7
  let o: Option<Int> = m.get("a")
  match o {
    Some(x) => { print(x) }
    None => { print(0) }
  }
}
```

Fixture 02 (`02-let-int-len.arita`), programa del test `b282_2` de `lib.rs`:

```arita
module t
fn main() -> Io<()> {
  let mut m: Map<Text, Int> = Map::new()
  m["a"] = 7
  let mut v: Vec<Int> = Vec::new()
  let n: Int = m.len()
  v.push(n)
  let k: Int = v.len()
  print(n)
  print(k)
}
```

(The module and final names are fixed by IMPL; they must keep the constructions `m.get("a")` with a literal over a local `Map` and `let n: Int = m.len()` / `let k: Int = v.len()`, and the stdout of §3.)

## 5. Cargo tests and close criteria

- `cargo test -p arita-codegen` green (includes the two unchanged `b282_emit_clippy`).
- `cargo test -p arita-cli` verde (smoke de `measure.rs` actualizado si cuenta ids).
- `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` without regression; miri-workspace green if it was green at the prior CLOSED.
- `arita measure` N/N with the N of §3, with no skip or inconclusive; **`clippy-driver` and `cargo clippy` must be available on Lex** (both D4 configurations) to count.

## 6. Measure PREP (blocks GO IMPL)

Read-only scan: (1) confirm no existing green oracle depends on the old form (`.get(&"…".to_string())` or `(x.len() as i64)` in the emit); (2) inventory of positive `.arita` with `let n: Int = <x>.len()` and with `get("lit")` over a local Map (23 examples with `let n: Int = v.len()` per the Measure PREP) and confirm all stay accepted with the fix already present (current situation: no codegen change); (3) confirm the new ids’ names do not collide with existing ids.

## 7. Open questions (Engineer)

**Resolved by the Engineer 01-10:** Q1 (4121 = B-283-1, moves to ADR-289; D6 intact); Q2 (both editions, 2015 with `clippy-driver` + 2021 with `cargo clippy`; see D4); Q3 (D7’s addendum suffices; GATE-283/284/285 and REF-MUT/VEC-ASSIGN/SCENARIO-MUT are not touched). No open questions.

## 8. Slice and GO

Single slice `EMIT-CLIPPY-B282` (size S, only `measure.rs` + fixtures + evidence). Sequence: DOC (this ADR) → Engineer verification → Measure PREP (§6) → Engineer **GO IMPL** with PKG-MEMBER CLOSED → implementation → Lex → CLOSED with real evidence. Inventing PASS / N/N / CLOSED is forbidden.

## 9. Changelog

- 2026-10-01 — v0 DRAFT PINS (Architect). Based on the Measure PREP; key finding: the codegen fix already exists and only the `measure` oracle is missing. No own execution.
- 2026-10-01 — v0.1 (Architect): Engineer decisions: B-283-1 / test 4121 / capacity() → ADR-289 (order after 288); D4 with both editions and tools (2015 `clippy-driver` + 2021 `cargo clippy`; IMPL does not reduce); D6 intact; Q1–Q3 resolved. Pins D1–D3, D5, D7 otherwise unchanged.

## 10. Close (CLOSED 2026-10-03)

Datos tomados de `DOC/GATE-CORE10-EMIT-CLIPPY-B282-20261003.md` (sha256 `f04dc2409e30aa7af8e5b07022727b3975ab9d688dfe299dc0e532fc1e71b720`).

- **Veredicto:** GO CLOSED ADR-288 (Ingeniero Rust, 2026-10-03). Medida final **853/853 accepted**, 0 skip.
- **Oráculos** (k = 2, prior N 851 → N = 853): `core09-b282-1-map-get-lit-emit-clippy` (fixture 01) and `core09-b282-2-let-len-emit-clippy` (fixture 02), both accepted; prior 851 unchanged vs `MEASURE_ADR287_PKG_MEMBER_EXCLUSIVE_20261002.json`.
- **Evidencia** (exclusive Lex run, 2026-10-02 23:54 → 2026-10-03 03:40, `/tmp/ing-adr288-final`): `cargo fmt --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` 0; `cargo build --release` 0; `cargo test --workspace -- --test-threads=1` 502 passed / 0 failed; `arita measure` exit 0, pure JSON stdout, 853 unique ids, 853 accepted, 0 skip; Miri inside measure and Veyra (green).
- **Veyra Proof** (`--profile quick`): **ACCEPTED** `20261003T003703Z` (rustfmt, cargo check, clippy, cargo test, cargo-audit, veyra-anti-theater; 0 problems, 0 skipped). The GATE cites no prior REJECTED run.
- **Herramientas:** rustc/cargo 1.97.1, clippy 0.1.97. **PATH note:** B-282 oracles invoke `clippy-driver` (under `~/.cargo/bin`); that directory must be on the measure run’s `PATH`; without it the 2 oracles cannot pass (no silent skip).
- **Artefacto de medición:** `DOC/reviews/MEASURE_ADR288_EMIT_CLIPPY_EXCLUSIVE_20261003.json` (sha256 `5e042970257339a0…`). **Freeze:** `DOC/reviews/MEASURE-ADR288-EMIT-CLIPPY-FREEZE-20261003.sha` (997 lines: 6 header + 991 verifiable with `shasum -a 256 -c`; file sha256 `f0855defdd2b…`) and its addendum `DOC/reviews/MEASURE-ADR288-EMIT-CLIPPY-FREEZE-20261003.addendum.sha` (GATE cites it without path; the path is that of the existing file).
- **Shas de código al cierre:** `crates/arita-cli/src/main.rs` `ed0820b4…` and `package.rs` `5fee2beb…` unchanged; `crates/arita-cli/src/measure.rs` `10443913…` → `159a07ad66fe…` (B282_ORACLES + smoke, additive only); fixture 01 `703aa559ea1e…`, fixture 02 `ac633cb86b90…`.
- **Addendum (Ingeniero, 2026-10-03): fixture 01:** `print(0)` → `print("none")` for E0274 / ADR-051 §2; oracle id, stdout (7) and clippy mechanism unchanged. (Per GATE: `None => { print(0) }` → `None => { print("none") }`; Engineer-approved and recorded in the freeze addendum; §4 programs are not rewritten.)
- **Limitación conocida / backlog:** the GATE declares no new limitation or backlog; only the PATH note above.
- **Orden siguiente:** ADR-289 → ADR-290 (IndexMut) → Mutex. The GATE lists next the Architect close edit (ADR-288 + ADR-289 addendum) and start of ADR-289 IMPL (GO IMPL = ADR-288 CLOSED).
- Note: ADR sha after this close: recorded by the Engineer addendum.
