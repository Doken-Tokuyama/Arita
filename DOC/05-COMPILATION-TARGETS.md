# Compilación multiplataforma

## Estrategia MVP

**ARITA → Rust source → rustc** (o `cargo`). Ver ADR-001.

Ventajas: targets Linux/Windows/macOS × x64/ARM ya existen; LTO, cross via `cargo zigbuild` / `cross`.

## Targets (misma tabla que `02-ARCHITECTURE.md`)

| OS | Arch | Fase |
|----|------|------|
| macOS | aarch64 | MVP (Fase 1) |
| Linux | x86_64 | MVP (Fase 1) |
| Linux | aarch64 | Fase 4 |
| Windows | x86_64 | Fase 4 |
| Windows | aarch64 | post-Fase 4 |
| macOS | x86_64 | post-Fase 4 |

**Estado spike (ADR-003):** emit-Rust verificado en Linux x86_64; macOS aarch64 hello pendiente (el gate local Shell `zsh ENOENT`, no diseño). Freeze emit AST: CUT-ID `F1-AST-RICH-20260913` (Module/Function/Call/LitStr).

## rustc target triples (prep multi-OS)

Triples **host/default** que usará `arita build` vía `rustc` / `cargo` más adelante. No son un cambio de emit; solo inventario.

| OS | Arch | Triple rustc (canónico) | Notas |
|----|------|-------------------------|--------|
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | MVP F1; verificado en spike |
| Linux | aarch64 | `aarch64-unknown-linux-gnu` | Fase 4; glibc. Musl: `aarch64-unknown-linux-musl` (opc.) |
| macOS | aarch64 | `aarch64-apple-darwin` | MVP F1; hold hasta Shell lex |
| macOS | x86_64 | `x86_64-apple-darwin` | post-Fase 4 |
| Windows | x86_64 | `x86_64-pc-windows-gnu` | Fase 4 / ADR-034 **v0 default**. `x86_64-pc-windows-msvc` = **OUT v0** |
| Windows | aarch64 | `aarch64-pc-windows-msvc` | post-Fase 4 |

### Instalación (ADR-034 / TARGETS-V0)

Antes de oráculos cross o `arita build --target <triple>`, instalar el target:

```bash
# Pins v0 (ADR-034)
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-unknown-linux-gnu

# Host / otros (según máquina)
rustup target add aarch64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
# msvc OUT v0 — no requerido:
# rustup target add x86_64-pc-windows-msvc
```

Sin target/linker: measure oráculos `target-win-gnu` / `target-linux-arm64` → **inconclusive** (gated; no tumba la suite; nunca PASS inventado). Cross Mac→Linux/Windows suele necesitar linker (`zig` / `cross` / mingw).

### Mapeo CLI (IMPL — ADR-034)

**Estado IMPL:** landed en el gate local — suite **accepted** **89/89** (`TARGETS-CROSS-LEX-20260915`). Host linkers in `.cargo/config.toml`: `aarch64-linux-gnu-gcc` + `x86_64-w64-mingw32-gcc` (mingw-w64). Without linkers, gated oracles stay **inconclusive** (never fake PASS).

```
arita build [--profile debug|release] [--target <triple>] <file.arita>
```

Default: host triple (sin `--target`). Con `--target` → camino Cargo emit (`cargo build --target <triple>`).

## Distribución

- `arita` CLI (Rust) + plantilla `arita new`.
- CI: GitHub Actions / local lex matrix (mac + linux) — pendiente F1.
- Emitir `.o` / staticlib C ABI: **post-MVP**, no camino crítico.

## Requisitos de usuario

- rustup + toolchain pin (MSRV a documentar en F1; crates spike = edition 2021).
- En Windows: Build Tools o llvm-mingw (documentar en F4).
