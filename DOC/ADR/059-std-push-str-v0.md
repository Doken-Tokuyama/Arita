# ADR-059 — String `push_str` v0

- **Estado:** **aceptada** + **verified** Lex **125/125**
- **CUT-ID:** `STD-PUSH-STR-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-026 std minima; ADR-049 clear; ADR-058 clone; ADR-006; ADR-022 safe-only
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole stack.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

Clone Lex **122/122**. Vec ya tiene `push`; String necesita append medible para IAs. **`push_str`** — sin Mutex ni Char type.

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.push_str(arg)` | `String` **o** lit str | **`mut String`** | `()` | `String::push_str` (`&str` / `&arg`) |

Pins:

1. Arity **1**; 0/2+ → E0203.
2. Receiver mut → **E0202** si falta `mut` (igual clear/push Vec).
3. Arg: `String` (emit `push_str(&arg)` — borrow arg; no move de `s`) **o** lit str (`push_str("…")`).
4. Whitelist + `push_str`. Safe-only.
5. Postcondición medible vía `len` / `print(s)` (no timed).

### 2. OUT v0

- `push(char)` / `Char` type
- `+=` sugar
- `insert_str` / `replace`
- Vec `push_str`
- Mutex

### 3. Oráculos measure

| Id | Expect |
|----|--------|
| `std-push-str-lit` | mut s; push_str lit; print/len canónico → **accepted** |
| `std-push-str-string` | push_str de otra String (clone ok); stdout canónico → **accepted** |
| `neg-e0202-push-str` (opc.) | push_str sin mut → **E0202** |

## Checklist

- [x] Pins + OUT + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **125/125** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
