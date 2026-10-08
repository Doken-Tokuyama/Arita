# ADR-278 — Core 0.8 slice 2: QUESTION-MARK (`?`)

- **Estado:** **CLOSED** Lex **764/764** (2026-09-26) · gate [`DOC/GATE-CORE08-QMARK-20260926.md`](../GATE-CORE08-QMARK-20260926.md)
- **CUT-ID:** `CORE-0.8-QMARK-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero (GO CLOSED 2026-09-26)
- **Padre:** [ADR-276](276-core-0.8-pins.md) §0.1 slice 2 · HALLAZGOS H3
- **Prev:** slice 1 FN-RESULT (ADR-277) **CLOSED** Lex **758/758**; este CUT **no** reabre 277 pins base
- **Prereq surface (al GO IMPL):** `fn → Result` land (277 CLOSED) + Result match (047)
- **HOLD:** Mutex · IndexMut-assign · idle · TLS/WS · crates.io · repair · I/O-new · String.set · threads · `?` on Option · `?` inside `fn → Io<()>` · surface unwrap/expect (241) · async Result · reopen E0272/E0340/E0341/E0291/E0342

## Objetivo

Unpark operador **`?`** on `Result`: desugar a early-return `Err` desde fn cuyo return es `Result<_,_>`. Pin interacción Io: **`?` solo** dentro de funciones `-> Result<_,_>`; **no** dentro de `fn main() -> Io<()>` (evita widening). Diag **E0343**. Anti: inventar `?` on Option = OUT v0. Emit-ban unwrap sigue.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Desugar** | `expr?` donde `expr: Result<T,E>` en fn `-> Result<U,E>` ≡ `match expr { Ok(v) => v, Err(e) => return Err(e) }` (E unifica; U/T vía pin Ingeniero sin inventar From v0 si OUT — DOC: mismo E o coerce ya IN) |
| **Ámbito** | **solo** body de `fn … -> Result<_,_>` |
| **OUT ámbito** | `?` dentro de `fn main() -> Io<()>` / cualquier fn→Io — **OUT** v0 (main usa match-convert) |
| **Diag** | **E0343** `question mark outside result fn` (texto canónico EN) |
| **Option** | `?` on Option **OUT** v0 |
| **Emit** | desugar → Rust `?` **o** match early-return equivalente **sin** unwrap/expect/panic (241) |
| **OK** | chain `a?; b?; Ok(v)` en helper Result |
| **No reabre** | E0342 (277) · E0272 · E0340 · E0341 · E0291 |

## 1. Surface ejemplo

```text
// POS — ? solo en fn → Result
fn load(path: String) -> Result<String, Int> {
  let t = host.read_text(path)?;
  Ok(t)
}

fn pipeline(path: String) -> Result<String, Int> {
  let a = load(path)?;
  Ok(a)
}

fn main() -> Io<()> {
  // PIN: sin ? aquí — match-convert
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
| `core08-qmark-chain` | chain `?` en fn→Result Ok → stdout esperado → PASS |
| `core08-qmark-err` | first `?` hits Err → early return Err → caller/main match fail → PASS |
| `neg-core08-qmark-outside` | `?` inside `fn → Io<()>` / non-Result fn → **E0343** |
| `neg-core08-qmark-option` | `?` on Option → OUT/reject estable (DOC; no invent E0xxx si OUT parse) |
| `core08-qmark-emit-ban` | emit grep: cero unwrap/expect/panic (241); `?` o match early-return only |
| `core08-qmark-main-io-match` | main Io match-convert sin `?` → PASS |

## 3. Pin E0343

**E0343** `question mark outside result fn` — uso de `?` fuera de función cuyo tipo de retorno es `Result<_,_>`.
**No** reasignar E0342 / E0272 / E0340 / E0341 / E0291.

## 4. Qué NO tocar

| Prohibido | Motivo |
|-----------|--------|
| `?` in Io main / Option? | OUT v0 (pin narrowing) |
| Mutex / IndexMut / idle / TLS/WS / crates.io / repair / I/O-new / String.set / threads | HOLD |
| Surface unwrap/expect | 241 HOLD |
| GO IMPL antes CLOSED 277 + OK Ingeniero | gate Orquestador |
| H4 VOID / H5 DOC | no slice aquí |
| Inventar PASS / Lex N/N | DOC only |

## 5. Criterio CLOSED

Lex §2 verde; tick ADR-276 slice 2; HOLDs §4 intactos. Siguiente: slice 3 SCENARIO-ERRPROP ([ADR-279](279-core-scenario-errprop-v0.md)).

## Checklist

- [x] Pins `?` desugar + E0343 + Io narrowing (GO-listo DOC)
- [x] GO IMPL Orquestador / Ingeniero (post-CLOSED 277 Lex **758/758**)
- [x] IMPL + Lex **764/764** CLOSED (Measure + Veyra ACCEPT)
- [ ] Slice 3 → ADR-279 (pins ya GO-listo DOC; **GO IMPL** post-CLOSED 278)

## Cierre

- **GO Ingeniero** 2026-09-26 · gate [`DOC/GATE-CORE08-QMARK-20260926.md`](../GATE-CORE08-QMARK-20260926.md)
- Lex measure **764/764** accepted → slice 2 **CLOSED** (evidence `DOC/reviews/MEASURE_ADR278_QMARK_REMEASURE_20260926.json`)
- Veyra quick **ACCEPTED** exit 0 · `.veyra/evidence/20260926T143843Z/` (VT008×12 info only)
- `measure_pass:false` pre-CLOSED era metadata «await Measure»; flipped `true` on CLOSED
- Unpark `?` **solo** en fn→Result; **no** en Io main · Option? OUT
- HOLDs globales + unwrap-surface + 279–280 intactos
- Anti-colisión E0342/E0272/E0340/E0341/E0291
- Next: **GO IMPL** ADR-279 SCENARIO-ERRPROP only (280 HOLD) · **no** Core 0.8 CLOSED
