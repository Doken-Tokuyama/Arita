[Español](05-COMPILATION-TARGETS.md) | English

# Cross-platform compilation

## MVP strategy

**ARITA → Rust source → rustc** (or `cargo`). See ADR-001.

Advantages: Linux/Windows/macOS × x64/ARM targets already exist; LTO, cross via `cargo zigbuild` / `cross`.

## Targets (same table as `02-ARCHITECTURE.md`)

| OS | Arch | Fase |
|----|------|------|
| macOS | aarch64 | MVP (Fase 1) |
| Linux | x86_64 | MVP (Fase 1) |
| Linux | aarch64 | Fase 4 |
| Windows | x86_64 | Fase 4 |
| Windows | aarch64 | post-Fase 4 |
| macOS | x86_64 | post-Fase 4 |

**Spike status (ADR-003):** emit-Rust verified on Linux x86_64; macOS aarch64 hello pending (local-gate Shell `zsh ENOENT`, not design). Emit AST freeze: CUT-ID `F1-AST-RICH-20260913` (Module/Function/Call/LitStr).

## rustc target triples (multi-OS prep)

**Host/default** triples that `arita build` will use via `rustc` / `cargo` later. Not an emit change; inventory only.

| OS | Arch | canonical rustc triple | Notes |
|----|------|-------------------------|--------|
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | MVP F1; verificado en spike |
| Linux | aarch64 | `aarch64-unknown-linux-gnu` | Fase 4; glibc. Musl: `aarch64-unknown-linux-musl` (opc.) |
| macOS | aarch64 | `aarch64-apple-darwin` | MVP F1; hold until local-gate Shell |
| macOS | x86_64 | `x86_64-apple-darwin` | post-Fase 4 |
| Windows | x86_64 | `x86_64-pc-windows-gnu` | Fase 4 / ADR-034 **v0 default**. `x86_64-pc-windows-msvc` = **OUT v0** |
| Windows | aarch64 | `aarch64-pc-windows-msvc` | post-Fase 4 |

### Installation (ADR-034 / TARGETS-V0)

Before cross oracles or `arita build --target <triple>`, install the target:

```bash
# Pins v0 (ADR-034)
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-unknown-linux-gnu

# Host / others (per machine)
rustup target add aarch64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
# msvc OUT v0 — no requerido:
# rustup target add x86_64-pc-windows-msvc
```

Without target/linker: measure oracles `target-win-gnu` / `target-linux-arm64` → **inconclusive** (gated; does not fail the suite; never invented PASS). Mac→Linux/Windows cross usually needs a linker (`zig` / `cross` / mingw).

### CLI mapping (IMPL — ADR-034)

**Estado IMPL:** landed local gate — suite **accepted** **89/89** (`TARGETS-CROSS-LEX-20260915`). Host linkers in `.cargo/config.toml`: `aarch64-linux-gnu-gcc` + `x86_64-w64-mingw32-gcc` (mingw-w64). Without linkers, gated oracles stay **inconclusive** (never fake PASS).

```
arita build [--profile debug|release] [--target <triple>] <file.arita>
```

Default: host triple (no `--target`). With `--target` → Cargo emit path (`cargo build --target <triple>`).

## Distribution

- `arita` CLI (Rust) + `arita new` template.
- CI: GitHub Actions / local lex matrix (mac + linux) — pending F1.
- Emit `.o` / C ABI staticlib: **post-MVP**, not the critical path.

## User requirements

- rustup + pinned toolchain (MSRV to document in F1; spike crates = edition 2021).
- On Windows: Build Tools or llvm-mingw (document in F4).
