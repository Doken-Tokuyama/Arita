Translation of `278-core-question-mark-v0.md`; the original is normative. / Traducción de `278-core-question-mark-v0.md`; el original es el normativo.

# ADR-278 — Core 0.8 slice 2: QUESTION-MARK (`?`)

- **Estado:** **CLOSED** Lex **764/764** (2026-09-26) · gate `DOC/GATE-CORE08-QMARK-20260926.md` (not in the public export / no incluido en el export público)
- **CUT-ID:** `CORE-0.8-QMARK-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero (GO CLOSED 2026-09-26)
- **Padre:** [ADR-276](276-core-0.8-pins.md) §0.1 slice 2 · HALLAZGOS H3
- **Prev:** slice 1 FN-RESULT (ADR-277) **CLOSED** Lex **758/758**; este CUT **no** reabre 277 pins base
- **Prereq surface (al GO IMPL):** `fn → Result` land (277 CLOSED) + Result match (047)
- **HOLD:** Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · I/O-new · String.set · threads · `?` on Option · `?` inside `fn → Io<()>` · surface unwrap/expect (241) · async Result · reopen E0272/E0340/E0341/E0291/E0342

## Goal

Unpark the **`?`** operator on `Result`: desugar to early-return `Err` from a fn whose return is `Result<_,_>`. Io interaction pin: **`?` only** inside functions `-> Result<_,_>`; **not** inside `fn main() -> Io<()>` (avoids widening). Diag **E0343**. Anti: inventing `?` on Option = OUT v0. Unwrap emit-ban remains.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Desugar** | `expr?` where `expr: Result<T,E>` in fn `-> Result<U,E>` ≡ `match expr { Ok(v) => v, Err(e) => return Err(e) }` (E unifies; U/T via Engineer pin without inventing From v0 if OUT — DOC: same E or already-IN coerce) |
| **Scope** | **only** the body of `fn … -> Result<_,_>` |
| **OUT of scope** | `?` inside `fn main() -> Io<()>` / any fn→Io — **OUT** v0 (main uses match-convert) |
| **Diag** | **E0343** `question mark outside result fn` (canonical EN text) |
| **Option** | `?` on Option **OUT** v0 |
| **Emit** | desugar → Rust `?` **or** equivalent early-return match **without** unwrap/expect/panic (241) |
| **OK** | chain `a?; b?; Ok(v)` in a Result helper |
| **Does not reopen** | E0342 (277) · E0272 · E0340 · E0341 · E0291 |

## 1. Surface example

```text
// POS — ? only in fn → Result
fn load(path: String) -> Result<String, Int> {
  let t = host.read_text(path)?;
  Ok(t)
}

fn pipeline(path: String) -> Result<String, Int> {
  let a = load(path)?;
  Ok(a)
}

fn main() -> Io<()> {
  // PIN: no ? here — match-convert
  match pipeline("a.txt") {
    Ok(t) => print(t),
    Err(_) => print("failed")
  }
}

// NEG → E0343
fn main() -> Io<()> {
  let t = host.read_text("a.txt")?;  // ? outside result fn
  print(t)
}

// OUT v0
// opt?  where opt: Option<_>
```

## 2. Oracles

| Id | Expect |
|----|--------|
| `core08-qmark-chain` | `?` chain in fn→Result Ok → expected stdout → PASS |
| `core08-qmark-err` | first `?` hits Err → early return Err → caller/main match fail → PASS |
| `neg-core08-qmark-outside` | `?` inside `fn → Io<()>` / non-Result fn → **E0343** |
| `neg-core08-qmark-option` | `?` on Option → stable OUT/reject (DOC; do not invent E0xxx if OUT parse) |
| `core08-qmark-emit-ban` | emit grep: zero unwrap/expect/panic (241); `?` or early-return match only |
| `core08-qmark-main-io-match` | main Io match-convert without `?` → PASS |

## 3. E0343 pin

**E0343** `question mark outside result fn` — use of `?` outside a function whose return type is `Result<_,_>`.
**Do not** reassign E0342 / E0272 / E0340 / E0341 / E0291.

## 4. What NOT to touch

| Forbidden | Reason |
|-----------|--------|
| `?` in Io main / Option? | OUT v0 (narrowing pin) |
| Mutex / IndexMut / idle / TLS/WS / crates.io / repair / I/O-new / String.set / threads | HOLD |
| Surface unwrap/expect | 241 HOLD |
| GO IMPL before CLOSED 277 + Engineer OK | Orchestrator gate |
| H4 VOID / H5 DOC | not this slice |
| Invent PASS / Lex N/N | DOC only |

## 5. CLOSED criterion

Green Lex §2; ADR-276 slice 2 tick; §4 HOLDs intact. Next: slice 3 SCENARIO-ERRPROP ([ADR-279](279-core-scenario-errprop-v0.en.md)).

## Checklist

- [x] Pins `?` desugar + E0343 + Io narrowing (DOC GO-ready)
- [x] GO IMPL Orchestrator / Engineer (post-CLOSED 277 Lex **758/758**)
- [x] IMPL + Lex **764/764** CLOSED (Measure + Veyra ACCEPT)
- [ ] Slice 3 → ADR-279 (pins already DOC GO-ready; **GO IMPL** post-CLOSED 278)

## Close

- **GO Ingeniero** 2026-09-26 · gate `DOC/GATE-CORE08-QMARK-20260926.md` (not in the public export / no incluido en el export público)
- Lex measure **764/764** accepted → slice 2 **CLOSED** (evidence `DOC/reviews/MEASURE_ADR278_QMARK_REMEASURE_20260926.json`)
- Veyra quick **ACCEPTED** exit 0 · `.veyra/evidence/20260926T143843Z/` (VT008×12 info only)
- `measure_pass:false` pre-CLOSED was “await Measure” metadata; flipped `true` on CLOSED
- Unpark `?` **only** in fn→Result; **not** in Io main · Option? OUT
- Global HOLDs + unwrap-surface + 279–280 intact
- Anti-collision E0342/E0272/E0340/E0341/E0291
- Next: **GO IMPL** ADR-279 SCENARIO-ERRPROP only (280 HOLD) · **no** Core 0.8 CLOSED
