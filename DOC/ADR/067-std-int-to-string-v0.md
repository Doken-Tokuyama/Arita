# ADR-067 — Int `to_string` v0

- **Estado:** **aceptada** + **verified** Lex **149/149**
- **CUT-ID:** `STD-INT-TO-STRING-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-066 parse_int (espejo); ADR-026; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

parse_int Lex **146/146**. Espejo: **Int → String** decimal.

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `n.to_string()` | 0 | **`Int`** | **`String`** | `n.to_string()` / format |

Pins:

1. Arity 0; malo → E0203.
2. Solo receiver **Int** v0 (Bool/Option OUT → E0206).
3. Decimal signed ASCII (Rust i64 Display).
4. Whitelist método en Int (extiende std; no es String method).
5. Round-trip medible: `n.to_string().parse_int()` → Ok(n) en oráculo pos.

### 2. OUT v0

- `to_string` en Bool → **ADR-070**; String/Vec siguen OUT
- hex/bin format
- Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-int-to-string` | print/assert `"42"` etc. → **accepted** |
| `std-int-to-string-roundtrip` | to_string+parse_int Ok same → **accepted** |
| `neg-e0206-to-string-bool` (opc.) | `true.to_string()` → **E0206** |

## Checklist

- [x] Pins + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **149/149** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
