Translation of `279-core-scenario-errprop-v0.md`; the original is normative. / Traducción de `279-core-scenario-errprop-v0.md`; el original es el normativo.

# ADR-279 — Core 0.8 slice 3: SCENARIO-ERRPROP

- **Estado:** **CLOSED** Lex **770/770** (Ingeniero; Measure accepted · Veyra ACCEPTED `.veyra/evidence/20260926T150744Z/` · [`GATE-CORE08-SCENARIO-ERRPROP-20260926.md`](../GATE-CORE08-SCENARIO-ERRPROP-20260926.md))
- **CUT-ID:** `CORE-0.8-SCENARIO-ERRPROP-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero · Orquestador (GO IMPL)
- **Padre:** [ADR-276](276-core-0.8-pins.md) §0.1 slice 3
- **Prev:** slice 2 QMARK (ADR-278) **CLOSED** Lex **764/764**; este CUT **no** reabre 277/278
- **Prereq surface (al GO IMPL):** FN-RESULT (277) + QMARK (278) en tree + host ADR-238
- **HOLD:** Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · I/O-new · String.set · threads · `?` in Io main · Option? · surface unwrap/expect · reopen E0340 (discard) · HTTP scenarios

## Goal

Scenarios/acceptance that exercise **H3** error-prop: happy `?` chain · Err path · neg outside-Result `?` (E0343) · reuse host 238 Result **with** `?` **inside** a helper `fn → Result` (**not** E0340 discard — different subject: typed propagate, not discard theater). If surface grows `is_ok` check-then-unwrap: DOC anti-pattern + emit-ban (do not invent a gate that reopens E0291). Bridge toward slice 4 REF-ERRPROP.

## 0. Decisive cut

| Pin | Decision |
|-----|----------|
| **Harness** | language-level `scenario`/`acceptance` (reuse 0.1 / 274 / 267 style) |
| **Input** | helpers `fn → Result` + host 238 `read_text`/`write_text` Result via `?` **inside** the helper; main Io match-convert |
| **Minimum** | ≥3 scenarios: happy `?` chain · Err early-return · ≥1 neg (E0343 **or** invent unwrap) |
| **Host 238** | `?` on read/write Result **inside** fn→Result = OK; discard Result without handling = **still** E0340 (0.7) — **no** morph or reopen |
| **Check-then-unwrap** | if `is_ok` + unwrap theater appears: DOC anti-pattern + emit-ban 241; **no** new E0xxx that reopens E0291 |
| **OUT** | `?` in main Io · Option? · Mutex · net/HTTP · IndexMut · crates.io |

## 1. Surface

No new keyword. Reuses `scenario` + `fn → Result` + `?` + host 238 + `fn main() -> Io<()>` match.

```text
fn load(path: String) -> Result<String, Int> {
  let t = host.read_text(path)?;
  Ok(t)
}

scenario errprop_happy {
  // helper ? chain Ok → fixed stdout via main match
  acceptance { /* measure */ }
}
scenario errprop_err {
  // read Err → ? early return → main match fail; no panic
  acceptance { /* measure */ }
}
// neg: ? in main Io → E0343
// neg: invent unwrap after is_ok → emit-ban / reject (DOC)
```

## 2. Oracles (Measure) — measurable success

| Id | Expect |
|----|--------|
| `core08-scen-errprop-happy` | `?` chain Ok → deterministic stdout → **PASS** |
| `core08-scen-errprop-err` | Err via `?` → fail path → **PASS** (no panic) |
| `core08-scen-errprop-host238` | host 238 Result + `?` inside helper fn→Result → PASS (≠ E0340 discard) |
| `neg-core08-scen-qmark-outside` | `?` outside Result fn → **E0343** |
| `neg-core08-scen-unwrap-theater` | invent unwrap / check-then-unwrap → emit-ban/reject (DOC; no E0291 reopen) |
| `core08-scen-errprop-build` | green measure build |

**Anti-theater:** skip ≠ PASS. Do not invent PASS. **Do not** use Result discard (E0340) as an oracle of this slice except as a regression note.

## 3. What NOT to touch

| Forbidden | Reason |
|-----------|--------|
| Mutex · IndexMut · idle · TLS/WS · crates.io · repair · I/O-new · String.set · threads | HOLD |
| GO IMPL before CLOSED 277+278 + Engineer OK | Orchestrator gate |
| Reopen E0340/E0341/E0272/E0291 | anti-collision |
| Declare Core 0.8 CLOSED | only slice 4 REF-ERRPROP |
| H4 VOID / H5 DOC IMPL | not a slice |

## 4. CLOSED criterion

Green Lex §2; ADR-276 slice 3 tick; §3 HOLDs intact. Next: slice 4 REF-ERRPROP (ADR-280).

## Checklist

- [x] H3 scenario pins + oracles (DOC GO-ready)
- [x] GO IMPL Orchestrator / Engineer (post-CLOSED 277+278 Lex **758**/ **764**)
- [x] IMPL + Lex Measure **770/770** accepted · Veyra ACCEPTED exit 0 · GATE CLOSED
- [x] Slice 4 → ADR-280 **GO IMPL** (post-CLOSED 279; **no** Core 0.8 CLOSED yet)

## Close

- **CLOSED** 2026-09-26 Lex Measure **770/770** · gate [`DOC/GATE-CORE08-SCENARIO-ERRPROP-20260926.md`](../GATE-CORE08-SCENARIO-ERRPROP-20260926.md) · Veyra **ACCEPTED** exit 0 · evidence `20260926T150744Z` (trail: `150320Z` rustfmt-diff → `cargo fmt --all` → ACCEPT)
- Prereq: ADR-277 **CLOSED** **758/758** · ADR-278 **CLOSED** **764/764**
- Host 238 + `?` in helper ≠ E0340 discard
- Global HOLDs intact · Next **GO IMPL** ADR-280 REF-ERRPROP only · **no** Core 0.8 CLOSED · Vertical ≠ GP ceiling
