[Español](LSP-EDITOR.md) | English

# LSP editor wiring — `arita lsp` (diagnostics v0)

- **Estado:** DOC v0 (wiring guía; sin CUT IMPL crates)
- **Fecha:** 2026-09-14
- **Relacionados:** [ADR-032](ADR/032-lsp-diagnostics-v0.md) (`LSP-DIAG-V0-20260914`), ROADMAP Fase 4 “LSP diagnostics”, oracle `cargo test -p arita-cli --test lsp_diagnostics`
- **Gobernanza:** documentación de cliente. **No** cambia crates/`measure`. Ampliar features LSP (complete/rename/pull) = CUT sucesor ADR-032.
- **Barra:** skip ≠ PASS; diagnostics reales con códigos E0xxx/E02xx; measure **OUT** v0 (LSP no entra en `arita measure`).

## What v0 is

`arita lsp` is a **stdio** JSON-RPC language server embedded in the CLI. It publishes **`textDocument/publishDiagnostics`** when a `.arita` is opened or edited. It reuses the same parse → lower → check pipeline as the CLI (same EN codes).

| IN v0 | OUT v0 |
|-------|--------|
| `initialize` / `initialized` / `shutdown` / `exit` | autocomplete / completion |
| `textDocument/didOpen` | hover rich |
| `textDocument/didChange` (full sync = `textDocumentSync: 1`) | rename / goto-def |
| `textDocument/publishDiagnostics` (push) | formatting / code actions |
| `serverInfo.name = "arita-lsp"` | pull `textDocument/diagnostic` |
| canonical codes in `code` + `message` (`E0xxx: …`) | TCP / WebSocket |

Unsupported requests → `MethodNotFound` (`unsupported in arita lsp v0: …`).

## How to start the server

From the workspace (bin on `PATH` or via `cargo`):

```bash
# release (recommended for editors)
cargo build -p arita-cli --release
./target/release/arita lsp

# o debug
cargo run -p arita-cli -- lsp
```

- **Transport:** stdin/stdout only. Do not open a port.
- **Logs:** do not pollute stdout (breaks JSON-RPC). Fatal errors → stderr + non-zero exit (`E0400`).
- **Args:** no v0 flags; only the `lsp` subcommand.

The process blocks until `shutdown`/`exit`. Clients spawn it and talk over pipes.

## Connect a client (minimum)

You need: command = `arita` binary, args = `["lsp"]`, language id for `*.arita`, **full** sync.

### VS Code / Cursor (generic LSP client)

No official ARITA v0 extension. Use a generic client (e.g. [vscode-languageclient](https://marketplace.visualstudio.com/items?itemName=ms-vscode.vscode-languageclient) via your own extension, or [LSP Viewer](https://marketplace.visualstudio.com/items?itemName=wscats.lsp-viewer) / “run command as language server” configs).

Minimal contribution example in a local extension `package.json`:

```json
{
  "name": "arita-lsp-v0",
  "engines": { "vscode": "^1.85.0" },
  "activationEvents": ["onLanguage:arita"],
  "contributes": {
    "languages": [{
      "id": "arita",
      "extensions": [".arita"],
      "aliases": ["ARITA"]
    }]
  }
}
```

In `activate` (TypeScript/JS), start the client like this (adjustable bin path):

```ts
import * as path from "path";
import { LanguageClient, TransportKind } from "vscode-languageclient/node";

const serverOptions = {
  command: path.join(workspaceRoot, "target/release/arita"),
  args: ["lsp"],
  transport: TransportKind.stdio,
};
const clientOptions = {
  documentSelector: [{ scheme: "file", language: "arita" }],
};
const client = new LanguageClient("arita", "ARITA LSP", serverOptions, clientOptions);
await client.start();
```

**Cursor:** same VS Code API; point `command` at Lex `arita` (`<repo>/target/release/arita`) or an install on `PATH`.

Settings-only (if you use a “generic LSP” extension that reads JSON):

```json
{
  "arita.lsp.command": "/absolute/path/to/arita",
  "arita.lsp.args": ["lsp"],
  "arita.lsp.filetypes": ["arita"]
}
```

(Exact keys depend on the extension; the ARITA contract is only **stdio + `arita lsp`**.)

### neovim (`nvim-lspconfig` / native LSP)

```lua
-- init.lua / lsp/arita.lua
vim.api.nvim_create_autocmd("FileType", {
  pattern = "arita",
  callback = function()
    vim.lsp.start({
      name = "arita-lsp",
      cmd = { "arita", "lsp" }, -- o ruta absoluta a target/release/arita
      root_dir = vim.fs.root(0, { "Cargo.toml", ".git" }),
    })
  end,
})

vim.filetype.add({ extension = { arita = "arita" } })
```

With `nvim-lspconfig` (if you register a custom server):

```lua
require("lspconfig.configs").arita = {
  default_config = {
    cmd = { "arita", "lsp" },
    filetypes = { "arita" },
    root_dir = require("lspconfig.util").root_pattern("Cargo.toml", ".git"),
    single_file_support = true,
  },
}
require("lspconfig").arita.setup({})
```

Open a `.arita` with a known error (e.g. `ejemplos/f2/neg/e0211-assert-true.arita`): diagnostic **E0211** should appear in the gutter.

## What you will see (and what you will not)

- On **open** or **change** of the buffer → one `publishDiagnostics` notification with 0..N items.
- Each diagnostic: `severity` Error, `source` = `"arita"`, `code` = `E0xxx`/`E02xx`, `message` starts with that code.
- Clean buffer → empty list (clears prior squiggles for the same URI).
- Do **not** expect completion, rename, typed hover, or pull diagnostics in v0.

## Anti-theater oracle

Do not use `arita measure` for LSP (OUT v0; preserves the measure bar).

```bash
cargo test -p arita-cli --test lsp_diagnostics
```

That test spawns `arita lsp`, does an `initialize` handshake, `didOpen` of E0211 / E0225 fixtures, and requires the code in `publishDiagnostics`. Absence / wrong code → fail (skip ≠ PASS).

Module unit tests: `cargo test -p arita-cli lsp::` (diagnose fixture + UTF-16 offset).

## Measure (do not touch)

Current bar (Lex, post ADR-033b/034): **76 accepted** + **2 gated inconclusive** (cross targets) → line `arita measure accepted: 78`. See `STABLE_VERIFY.md`.

History when LSP landed: measure **75/75** preserved (LSP outside measure). Narrative 75→76→78 = host-border + gated targets; **this DOC does not mutate measure or crates**.

## One-line summary

**`arita lsp` over stdio; VS Code/Cursor/neovim client only needs cmd=`arita` args=`lsp`; v0 = push-only diagnostics; oracle = `cargo test -p arita-cli --test lsp_diagnostics`; ADR-032.**
