Translation of `032-lsp-diagnostics-v0.md`; the original is normative. / Traducción de `032-lsp-diagnostics-v0.md`; el original es el normativo.

# ADR-032 — LSP diagnostics v0 (Phase 4)

- **Estado:** **aceptada** + **verified** (`arita lsp` + `cargo test` `lsp_diagnostics`; measure **75/75** preserved)
- **CUT-ID:** `LSP-DIAG-V0-20260914`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (draft/align) + Ingeniero Rust (**GO DOC+IMPL**)
- **Relacionados:** ADR-005 (E0xxx), ROADMAP Fase 4 “LSP diagnostics”, `DOC/LSP-EDITOR.md` (wiring editores), baseline measure **75/75**
- **Gobernanza:** **aceptada** + **verified**. CUT `LSP-DIAG-V0-20260914`. Parser/Codegen HOLD salvo review. Safe-only (ADR-022). skip ≠ PASS. Oracle: `cargo test` only (measure OUT v0).
- **Barra:** diagnóstico real desde parse/check; oráculo E2E/integración con código E0xxx en publishDiagnostics — no theater.

## Context

ROADMAP Phase 4 lists LSP diagnostics. After E0214/E0225 traps (measure 75/75), the next useful gap for AIs/editors is **seeing ARITA errors in the editor** with stable EN codes.

## Decision (MVP)

### 1. IN

| Piece | v0 rule |
|-------|----------|
| Protocol | LSP: at least `initialize`, `textDocument/didOpen` / `didChange`, **`textDocument/publishDiagnostics`** |
| Source | Reuse `arita` parse/check pipeline (same E0xxx / E02xx as CLI) |
| Payload | Each diagnostic: `message` includes **canonical code** (`E0xxx` / `E02xx`…) + EN text; `range` in the file |
| Transport | **stdio** (MVP) |
| Invocation | **`arita lsp`** (CLI subcommand; no separate bin) |
| Trigger | **Push only:** `didOpen` + `didChange`. Pull `textDocument/diagnostic` = **OUT** v0 |
| OUT v0 | autocomplete, hover rich, rename, goto-def, formatting, code actions |

### 2. Oracle (anti-theater)

1. `.arita` fixture with a known error (e.g. neg E0211 or E0225).
2. Integration client/test starts the server, `didOpen`, waits for `publishDiagnostics`.
3. **accepted** only if the expected code appears in diagnostics; absence / wrong code → rejected; skip → inconclusive (never accepted).

**v0 oracle:** integration **`cargo test` only** (fixture → `publishDiagnostics` with E0xxx). **`arita measure` = OUT v0** (avoids LSP-process theater in the suite).

### 3. Safe-only

No `unsafe` in ARITA surface; the server is a Rust host (Clippy `-D warnings` on touched crates).

### 4. Invocation shape (closed pin)

```bash
arita lsp
```

## Closed decisions (2026-09-14 — Rust Engineer GO)

1. **`arita lsp`** (CLI subcommand; no separate bin).
2. **Push only** v0: `didOpen` + `didChange`. Pull `textDocument/diagnostic` = OUT v0.
3. Oracle: integration **`cargo test`**. `arita measure` = OUT v0.

## OUT

- Language features beyond diagnostics
- TCP/WebSocket without CUT
- Pull diagnostics v0
- Invent codes different from the compiler
- Put LSP into `arita measure` v0
- “LSP done” without a `cargo test` oracle that sees E0xxx

## Consequences

- CUT IMPL authorized: Engineer executor (full stack).
- Parser/Codegen: HOLD except review-only.
- Baseline measure: **75/75** pre-CUT (no suite regression from LSP).

## GO checklist

- [x] publishDiagnostics + E0xxx EN OK
- [x] No autocomplete/rename v0 OK
- [x] Anti-theater `cargo test` oracle OK
- [x] Open items 1–3 **closed**
- [x] GO DOC → **accepted** + IMPL authorized
- [x] `arita lsp` + integration tests (Engineer) — landed 2026-09-14; oracle `cargo test -p arita-cli --test lsp_diagnostics`
