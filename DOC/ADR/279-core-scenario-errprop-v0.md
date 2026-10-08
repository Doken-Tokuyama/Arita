# ADR-279 — Core 0.8 slice 3: SCENARIO-ERRPROP

- **Estado:** **CLOSED** Lex **770/770** (Ingeniero; Measure accepted · Veyra ACCEPTED `.veyra/evidence/20260926T150744Z/` · [`GATE-CORE08-SCENARIO-ERRPROP-20260926.md`](../GATE-CORE08-SCENARIO-ERRPROP-20260926.md))
- **CUT-ID:** `CORE-0.8-SCENARIO-ERRPROP-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero · Orquestador (GO IMPL)
- **Padre:** [ADR-276](276-core-0.8-pins.md) §0.1 slice 3
- **Prev:** slice 2 QMARK (ADR-278) **CLOSED** Lex **764/764**; este CUT **no** reabre 277/278
- **Prereq surface (al GO IMPL):** FN-RESULT (277) + QMARK (278) en tree + host ADR-238
- **HOLD:** Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · I/O-new · String.set · threads · `?` in Io main · Option? · surface unwrap/expect · reopen E0340 (discard) · HTTP scenarios

## Objetivo

Scenarios/acceptance que ejercitan **H3** error-prop: happy `?` chain · Err path · neg outside-Result `?` (E0343) · reuse host 238 Result **con** `?` **dentro** de helper `fn → Result` (**no** discard E0340 — sujeto distinto: propagate tipado, no theater discard). Si surface crece `is_ok` check-then-unwrap: DOC anti-pattern + emit-ban (no inventar gate E0291 reopen). Puente hacia slice 4 REF-ERRPROP.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Harness** | `scenario`/`acceptance` language-level (reusar 0.1 / 274 / 267 style) |
| **Entrada** | helpers `fn → Result` + host 238 `read_text`/`write_text` Result vía `?` **dentro** helper; main Io match-convert |
| **Mínimo** | ≥3 scenarios: happy `?` chain · Err early-return · ≥1 neg (E0343 **o** invent unwrap) |
| **Host 238** | `?` on read/write Result **inside** fn→Result = OK; discard Result sin handle = **sigue** E0340 (0.7) — **no** morph ni reopen |
| **Check-then-unwrap** | si `is_ok` + unwrap theater aparece: DOC anti-pattern + emit-ban 241; **no** new E0xxx que reabra E0291 |
| **OUT** | `?` in main Io · Option? · Mutex · net/HTTP · IndexMut · crates.io |

## 1. Surface

Sin keyword nueva. Reusa `scenario` + `fn → Result` + `?` + host 238 + `fn main() -> Io<()>` match.

```text
fn load(path: String) -> Result<String, Int> {
  let t = host.read_text(path)?;
  Ok(t)
}

scenario errprop_happy {
  // helper ? chain Ok → stdout fijo vía main match
  acceptance { /* measure */ }
}
scenario errprop_err {
  // read Err → ? early return → main match fail; sin panic
  acceptance { /* measure */ }
}
// neg: ? in main Io → E0343
// neg: invent unwrap after is_ok → emit-ban / reject (DOC)
```

## 2. Oracles (Measure) — éxito medible

| Id | Expect |
|----|--------|
| `core08-scen-errprop-happy` | `?` chain Ok → stdout determinista → **PASS** |
| `core08-scen-errprop-err` | Err via `?` → fail path → **PASS** (no panic) |
| `core08-scen-errprop-host238` | host 238 Result + `?` inside helper fn→Result → PASS (≠ E0340 discard) |
| `neg-core08-scen-qmark-outside` | `?` outside Result fn → **E0343** |
| `neg-core08-scen-unwrap-theater` | invent unwrap / check-then-unwrap → emit-ban/reject (DOC; no E0291 reopen) |
| `core08-scen-errprop-build` | measure build verde |

**Anti-theater:** skip ≠ PASS. No inventar PASS. **No** discard Result (E0340) como oracle de este slice salvo regresión note.

## 3. Qué NO tocar

| Prohibido | Motivo |
|-----------|--------|
| Mutex · IndexMut · idle · TLS/WS · crates.io · repair · I/O-new · String.set · threads | HOLD |
| GO IMPL antes CLOSED 277+278 + OK Ingeniero | gate Orquestador |
| Reabrir E0340/E0341/E0272/E0291 | anti-colisión |
| Declarar CLOSED Core 0.8 | solo slice 4 REF-ERRPROP |
| H4 VOID / H5 DOC IMPL | no slice |

## 4. Criterio CLOSED

Lex §2 verde; tick ADR-276 slice 3; HOLDs §3 intactos. Siguiente: slice 4 REF-ERRPROP (ADR-280).

## Checklist

- [x] Pins scenarios H3 + oracles (GO-listo DOC)
- [x] GO IMPL Orquestador / Ingeniero (post-CLOSED 277+278 Lex **758**/ **764**)
- [x] IMPL + Lex Measure **770/770** accepted · Veyra ACCEPTED exit 0 · GATE CLOSED
- [x] Slice 4 → ADR-280 **GO IMPL** (post-CLOSED 279; **no** Core 0.8 CLOSED aún)

## Cierre

- **CLOSED** 2026-09-26 Lex Measure **770/770** · gate [`DOC/GATE-CORE08-SCENARIO-ERRPROP-20260926.md`](../GATE-CORE08-SCENARIO-ERRPROP-20260926.md) · Veyra **ACCEPTED** exit 0 · evidence `20260926T150744Z` (trail: `150320Z` rustfmt-diff → `cargo fmt --all` → ACCEPT)
- Prereq: ADR-277 **CLOSED** **758/758** · ADR-278 **CLOSED** **764/764**
- Host 238 + `?` in helper ≠ E0340 discard
- HOLDs globales intactos · Next **GO IMPL** ADR-280 REF-ERRPROP only · **no** Core 0.8 CLOSED · Vertical ≠ techo GP
