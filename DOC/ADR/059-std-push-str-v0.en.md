Translation of `059-std-push-str-v0.md`; the original is normative. / Traducción de `059-std-push-str-v0.md`; el original es el normativo.

# ADR-059 — String `push_str` v0

- **Estado:** **aceptada** + **verified** Lex **125/125**
- **CUT-ID:** `STD-PUSH-STR-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-026 std minima; ADR-049 clear; ADR-058 clone; ADR-006; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS.

## Context

Clone Lex **122/122**. Vec already has `push`; String needs measurable append for AIs. **`push_str`** — no Mutex or Char type.

## Decision (GO pins)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.push_str(arg)` | `String` **o** lit str | **`mut String`** | `()` | `String::push_str` (`&str` / `&arg`) |

Pins:

1. Arity **1**; 0/2+ → E0203.
2. Receiver mut → **E0202** if missing `mut` (igual clear/push Vec).
3. Arg: `String` (emit `push_str(&arg)` — borrow arg; no move de `s`) **o** lit str (`push_str("…")`).
4. Whitelist + `push_str`. Safe-only.
5. Measurable postcondition via `len` / `print(s)` (no timed).

### 2. OUT v0

- `push(char)` / `Char` type
- `+=` sugar
- `insert_str` / `replace`
- Vec `push_str`
- Mutex

### 3. Oracles measure

| Id | Expect |
|----|--------|
| `std-push-str-lit` | mut s; push_str lit; canonical print/len → **accepted** |
| `std-push-str-string` | push_str de otra String (clone ok); canonical stdout → **accepted** |
| `neg-e0202-push-str` (opt.) | push_str missing mut → **E0202** |

## Checklist

- [x] Pins + OUT + oracles
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **125/125** (STABLE_VERIFY)

## Queue

Mutex **PARK**.
