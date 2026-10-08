# ADR-066 — String `parse_int` → Result v0

- **Estado:** **aceptada** + **verified** Lex **146/146**
- **CUT-ID:** `STD-PARSE-INT-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-047 Result; ADR-050 Option; ADR-026; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

Case Lex **143/143**. Tras racha String transform, desbloquear **parse decimal → Int** vía Result (surface ya existe). Útil IA; no Mutex.

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit (orientativo) |
|---------|------|----------|-----|---------------------|
| `s.parse_int()` | 0 | `String` shared | **`Result<Int, String>`** | parse `i64`; Ok(n) / Err(msg) |

Pins:

1. Arity 0; malo → E0203.
2. No mut.
3. Éxito: decimal ASCII opcional signo (`+`/`-`), sin `_` separators v0 → **Ok(Int)**.
4. Fallo (vacío, no-entero, overflow i64): **Err(String)** mensaje EN estable corto (p.ej. `invalid integer` / `integer overflow`) — pin un mensaje canónico por clase o uno genérico `invalid integer` v0.
5. Whitelist + `parse_int`. Safe-only.
6. Match/`if let` Result ya existentes aplican (E0270/E0272/E0275).

**Pin mensaje Err v0:** un solo literal **`invalid integer`** para todo fallo (simple; overflow subsumido).

### 2. OUT v0

- `parse` genérico / floats / radix
- `unwrap` parse
- Option-returning parse
- Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-parse-int-ok` | `"42".parse_int()` → Ok path print 42 → **accepted** |
| `std-parse-int-err` | `"x".parse_int()` → Err; print/`assert` mensaje o branch → **accepted** |
| `neg-e0206-parse-int-vec` (opc.) | Vec.parse_int → **E0206** |

## Checklist

- [x] Pins Result<Int,String> + Err msg + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **146/146** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
