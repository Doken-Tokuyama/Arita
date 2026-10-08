# LSP editor wiring — `arita lsp` (diagnostics v0)

- **Estado:** DOC v0 (wiring guía; sin CUT IMPL crates)
- **Fecha:** 2026-09-14
- **Relacionados:** [ADR-032](ADR/032-lsp-diagnostics-v0.md) (`LSP-DIAG-V0-20260914`), ROADMAP Fase 4 “LSP diagnostics”, oracle `cargo test -p arita-cli --test lsp_diagnostics`
- **Gobernanza:** documentación de cliente. **No** cambia crates/`measure`. Ampliar features LSP (complete/rename/pull) = CUT sucesor ADR-032.
- **Barra:** skip ≠ PASS; diagnostics reales con códigos E0xxx/E02xx; measure **OUT** v0 (LSP no entra en `arita measure`).

## Qué es v0

`arita lsp` es un language server **stdio** JSON-RPC embebido en el CLI. Publica **`textDocument/publishDiagnostics`** al abrir o editar un `.arita`. Reusa el mismo pipeline parse → lower → check que la CLI (mismos códigos EN).

| IN v0 | OUT v0 |
|-------|--------|
| `initialize` / `initialized` / `shutdown` / `exit` | autocomplete / completion |
| `textDocument/didOpen` | hover rich |
| `textDocument/didChange` (full sync = `textDocumentSync: 1`) | rename / goto-def |
| `textDocument/publishDiagnostics` (push) | formatting / code actions |
| `serverInfo.name = "arita-lsp"` | pull `textDocument/diagnostic` |
| códigos canónicos en `code` + `message` (`E0xxx: …`) | TCP / WebSocket |

Peticiones no soportadas → `MethodNotFound` (`unsupported in arita lsp v0: …`).

## Cómo arrancar el server

Desde el workspace (bin en `PATH` o vía `cargo`):

```bash
# release (recomendado para editores)
cargo build -p arita-cli --release
./target/release/arita lsp

# o debug
cargo run -p arita-cli -- lsp
```

- **Transport:** stdin/stdout únicamente. No abrir puerto.
- **Logs:** no ensuciar stdout (rompe JSON-RPC). Errores fatales → stderr + exit no-cero (`E0400`).
- **Args:** ningún flag v0; solo el subcomando `lsp`.

El proceso bloquea hasta `shutdown`/`exit`. Los clientes lo spawnean y hablan por pipes.

## Conectar un cliente (mínimo)

Necesitas: comando = binario `arita`, args = `["lsp"]`, language id para `*.arita`, sync **full**.

### VS Code / Cursor (generic LSP client)

Sin extensión oficial ARITA v0. Usa un cliente genérico (p.ej. [vscode-languageclient](https://marketplace.visualstudio.com/items?itemName=ms-vscode.vscode-languageclient) vía extensión propia, o [LSP Viewer](https://marketplace.visualstudio.com/items?itemName=wscats.lsp-viewer) / configs de “run command as language server”).

Ejemplo de contribución mínima en `package.json` de una extensión local:

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

En el `activate` (TypeScript/JS), arrancar el cliente así (ruta al bin ajustable):

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

**Cursor:** misma API VS Code; apunta `command` al `arita` de Lex (`<repo>/target/release/arita`) o a un install en `PATH`.

Settings-only (si usas una extensión “generic LSP” que lea JSON):

```json
{
  "arita.lsp.command": "/absolute/path/to/arita",
  "arita.lsp.args": ["lsp"],
  "arita.lsp.filetypes": ["arita"]
}
```

(Exact keys dependen de la extensión; el contrato ARITA es solo **stdio + `arita lsp`**.)

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

Con `nvim-lspconfig` (si registras un server custom):

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

Abre un `.arita` con error conocido (p.ej. `ejemplos/f2/neg/e0211-assert-true.arita`): debe aparecer diagnóstico **E0211** en el gutter.

## Qué verás (y qué no)

- Al **abrir** o **cambiar** el buffer → una notificación `publishDiagnostics` con 0..N items.
- Cada diagnostic: `severity` Error, `source` = `"arita"`, `code` = `E0xxx`/`E02xx`, `message` empieza por ese código.
- Buffer limpio → lista vacía (limpia squiggles previos del mismo URI).
- **No** esperes completion, rename, hover tipado, ni pull diagnostics en v0.

## Oráculo anti-theater

No uses `arita measure` para LSP (OUT v0; preserva la barra measure).

```bash
cargo test -p arita-cli --test lsp_diagnostics
```

Ese test spawnea `arita lsp`, hace handshake `initialize`, `didOpen` de fixtures E0211 / E0225, y exige el código en `publishDiagnostics`. Ausencia / wrong code → fail (skip ≠ PASS).

Unitarios del módulo: `cargo test -p arita-cli lsp::` (diagnose fixture + offset UTF-16).

## Measure (no tocar)

Barra actual (Lex, post ADR-033b/034): **76 accepted** + **2 gated inconclusive** (targets cross) → línea `arita measure accepted: 78`. Ver `STABLE_VERIFY.md`.

Histórico al landear LSP: measure **75/75** preservado (LSP fuera de measure). Narrativa 75→76→78 = host-border + targets gated; **este DOC no muta measure ni crates**.

## Resumen una línea

**`arita lsp` por stdio; cliente VS Code/Cursor/neovim solo necesita cmd=`arita` args=`lsp`; v0 = diagnostics push-only; oracle = `cargo test -p arita-cli --test lsp_diagnostics`; ADR-032.**
