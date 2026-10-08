# ADR-032 — LSP diagnostics v0 (Fase 4)

- **Estado:** **aceptada** + **verified** (`arita lsp` + `cargo test` `lsp_diagnostics`; measure **75/75** preserved)
- **CUT-ID:** `LSP-DIAG-V0-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (draft/align) + Ingeniero Rust (**GO DOC+IMPL**)
- **Relacionados:** ADR-005 (E0xxx), ROADMAP Fase 4 “LSP diagnostics”, `DOC/LSP-EDITOR.md` (wiring editores), baseline measure **75/75**
- **Gobernanza:** **aceptada** + **verified**. CUT `LSP-DIAG-V0-20260914`. Parser/Codegen HOLD salvo review. Safe-only (ADR-022). skip ≠ PASS. Oracle: `cargo test` only (measure OUT v0).
- **Barra:** diagnóstico real desde parse/check; oráculo E2E/integración con código E0xxx en publishDiagnostics — no theater.

## Contexto

ROADMAP Fase 4 lista LSP diagnostics. Tras trampas E0214/E0225 (measure 75/75), el siguiente hueco útil para IAs/editores es **ver errores ARITA en el editor** con códigos estables EN.

## Decisión (MVP)

### 1. IN

| Pieza | Regla v0 |
|-------|----------|
| Protocolo | LSP: al menos `initialize`, `textDocument/didOpen` / `didChange`, **`textDocument/publishDiagnostics`** |
| Fuente | Reusar pipeline `arita` parse/check (mismos E0xxx / E02xx que CLI) |
| Payload | Cada diagnostic: `message` incluye **código canónico** (`E0xxx` / `E02xx`…) + texto EN; `range` en el fichero |
| Transport | **stdio** (MVP) |
| Invocación | **`arita lsp`** (subcommand CLI; no bin separado) |
| Trigger | **Push only:** `didOpen` + `didChange`. Pull `textDocument/diagnostic` = **OUT** v0 |
| OUT v0 | autocomplete, hover rich, rename, goto-def, formatting, code actions |

### 2. Oráculo (anti-theater)

1. Fixture `.arita` con error conocido (p.ej. neg E0211 o E0225).
2. Cliente/test de integración arranca el server, `didOpen`, espera `publishDiagnostics`.
3. **accepted** solo si aparece el código esperado en diagnostics; ausencia / wrong code → rejected; skip → inconclusive (nunca accepted).

**Oráculo v0:** solo **`cargo test`** de integración (fixture → `publishDiagnostics` con E0xxx). **`arita measure` = OUT v0** (evita theater de proceso LSP en la suite).

### 3. Safe-only

Sin `unsafe` en surface ARITA; el server es Rust host (Clippy `-D warnings` en crates tocados).

### 4. Forma de invocación (pin cerrado)

```bash
arita lsp
```

## Decisiones cerradas (2026-09-14 — Ingeniero GO)

1. **`arita lsp`** (subcommand CLI; no bin separado).
2. **Push only** v0: `didOpen` + `didChange`. Pull `textDocument/diagnostic` = OUT v0.
3. Oráculo: **`cargo test`** integración. `arita measure` = OUT v0.

## OUT

- Language features más allá de diagnostics
- TCP/WebSocket sin CUT
- Pull diagnostics v0
- Inventar códigos distintos del compilador
- Meter LSP en `arita measure` v0
- “LSP done” sin oráculo `cargo test` que vea E0xxx

## Consecuencias

- CUT IMPL autorizado: executor Ingeniero (stack completo).
- Parser/Codegen: HOLD salvo review-only.
- Baseline measure: **75/75** pre-CUT (no regresión suite por LSP).

## Checklist GO

- [x] publishDiagnostics + E0xxx EN OK
- [x] Sin autocomplete/rename v0 OK
- [x] Oráculo `cargo test` anti-theater OK
- [x] Abiertas 1–3 **cerradas**
- [x] GO DOC → **aceptada** + IMPL autorizado
- [x] `arita lsp` + tests integración (Ingeniero) — landed 2026-09-14; oracle `cargo test -p arita-cli --test lsp_diagnostics`
