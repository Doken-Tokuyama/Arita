Translation of `066-std-parse-int-v0.md`; the original is normative. / Traducción de `066-std-parse-int-v0.md`; el original es el normativo.

# ADR-066 — String `parse_int` → Result v0

- **Estado:** **aceptada** + **verified** Lex **146/146**
- **CUT-ID:** `STD-PARSE-INT-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-047 Result; ADR-050 Option; ADR-026; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

Case Lex **143/143**. After the String-transform streak, unlock **parse decimal → Int** via Result (surface already exists). AI-useful; no Mutex.

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit (orientativo) |
|---------|------|----------|-----|---------------------|
| `s.parse_int()` | 0 | `String` shared | **`Result<Int, String>`** | parse `i64`; Ok(n) / Err(msg) |

Pins:

1. Arity 0; bad → E0203.
2. No mut.
3. Success: decimal ASCII optional sign (`+`/`-`), no `_` separators v0 → **Ok(Int)**.
4. Failure (empty, non-integer, overflow i64): **Err(String)** short stable EN message (e.g. `invalid integer` / `integer overflow`) — pin one message canonical por clase o uno generic `invalid integer` v0.
5. Whitelist + `parse_int`. Safe-only.
6. Match/`if let` Result ya existentes aplican (E0270/E0272/E0275).

**Pin message Err v0:** a single literal **`invalid integer`** for every failure (simple; overflow subsumed).

### 2. OUT v0

- `generic `parse` / floats / radix
- `unwrap` parse
- Option-returning parse
- Mutex

### 3. Oracles

| Id | Expect |
|----|--------|
| `std-parse-int-ok` | `"42".parse_int()` → Ok path print 42 → **accepted** |
| `std-parse-int-err` | `"x".parse_int()` → Err; print/`assert` message o branch → **accepted** |
| `neg-e0206-parse-int-vec` (opt.) | Vec.parse_int → **E0206** |

## Checklist

- [x] Pins Result<Int,String> + Err msg + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **146/146** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
