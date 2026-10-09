Translation of `063-std-trim-v0.md`; the original is normative. / Traducción de `063-std-trim-v0.md`; el original es el normativo.

# ADR-063 — String `trim` v0

- **Estado:** **aceptada** + **verified** Lex **137/137**
- **CUT-ID:** `STD-TRIM-20260918`
- **Fecha:** 2026-09-18
- **Autores:** ARITA Arquitecto (pins) · Ingeniero Rust (IMPL)
- **Relacionados:** ADR-026; ADR-060–062 query trio; ADR-058 clone; ADR-022
- **Gobernanza:** Sin idle; **Mutex PARK**. Parser/Codegen HOLD review-only; Ingeniero sole.
- **Barra:** E2E measure; skip ≠ PASS.

## Contexto

Trío contains/starts_with/ends_with Lex **134/134**. Siguiente std IA-útil: **`trim() -> String`** (nuevo string; shared borrow).

## Decisión (pins GO)

### 1. IN v0

| Surface | Args | Receiver | Ret | Emit |
|---------|------|----------|-----|------|
| `s.trim()` | 0 | `String` shared | **`String`** | `.trim().to_string()` (o equiv. owned) |

Pins:

1. Arity 0; args → E0203.
2. No exige `mut`.
3. Retorna **String nueva** (original intacta).
4. Whitelist + `trim`. Safe-only.
5. Unicode-aware = Rust `str::trim` (ws Unicode).

### 2. OUT v0

- `trim_start` / `trim_end` → **ADR-073**; `trim_matches` posterior
- in-place trim
- Mutex

### 3. Oráculos

| Id | Expect |
|----|--------|
| `std-trim-spaces` | `"  hi  ".trim()` → print `hi` / len canónico → **accepted** |
| `std-trim-noop` | sin ws extremos → mismo contenido → **accepted** |
| `neg-e0206-trim-vec` (opc.) | Vec.trim → **E0206** |

## Checklist

- [x] Pins + oráculos
- [x] GO DOC + IMPL
- [x] Landed + measure Lex **137/137** (STABLE_VERIFY)

## Cola

Mutex **PARK**.
