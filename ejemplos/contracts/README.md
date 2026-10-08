# Contract oracles (CONTRACT-V0 + ATTEST-V0 + LANG-CONTRACT + TARGET-FN)

JSON contracts for `arita contract <path.json>` and measure `ContractOracle` (ADR-017).
Optional `source_sha256` (ADR-018 / ATTEST-V0): SHA-256 hex of **raw** `.arita` bytes.
Language-level `contract { }` inside `.arita` (ADR-019 / LANG-CONTRACT): same expect pipeline; not emitted to Rust.
Optional `target` (ADR-020): JSON `"target": "fn_name"` / language `target <ident>`; default `main`.
When `target ≠ main`, codegen **entry-rewrites** so Rust `fn main` calls `target()` (user `main` → `__arita_user_main`).
`expect_reject` ignores `target` (whole-file check). Missing / non-`Io<()>` target → **E0230**.

| File | id | Kind |
|------|-----|------|
| `contract-hello.json` | `contract-hello` | expect_stdout `hello` |
| `contract-f23-break.json` | `contract-f23-break` | expect_stdout `0` / `1` |
| `contract-neg-e0224.json` | `contract-neg-e0224` | expect_reject `E0224` |
| `contract-hello-attested.json` | `contract-hello-attested` | stdout + correct `source_sha256` |
| `contract-hello-bad-hash.json` | `contract-hello-bad-hash` | wrong sha → contract **rejects**; measure **accepted** iff reject |
| `lang-hello.arita` | `lang-contract-hello` | language `expect_stdout "hello"` |
| `lang-neg-e0224.arita` | `lang-contract-neg-e0224` | language `expect_reject "E0224"` |
| `lang-target-helper.arita` | `lang-contract-target-helper` | `target helper` → stdout `from-helper` (not `from-main`) |
| `contract-target-helper.json` | `contract-target-helper` | JSON `target: helper` same source |

Exactly one expect kind per contract (`expect_stdout` lines **or** `expect_reject`). Invalid schema → inconclusive (never PASS).

## Language surface (ADR-019)

```arita
module demo
fn main() -> Io<()> {
  print("hello")
}
contract hello_c {
  expect_stdout "hello"
}
```

```arita
module demo
fn helper() -> Io<()> {
  print("from-helper")
}
fn main() -> Io<()> {
  print("from-main")
}
contract c {
  target helper
  expect_stdout "from-helper"
}
```

```bash
cargo run -p arita-cli -- contract ejemplos/contracts/lang-hello.arita
cargo run -p arita-cli -- contract ejemplos/contracts/lang-neg-e0224.arita
cargo run -p arita-cli -- contract ejemplos/contracts/lang-target-helper.arita
cargo run -p arita-cli -- contract ejemplos/contracts/contract-target-helper.json
```

## Hash (Mac / Linux sync)

```bash
# macOS
shasum -a 256 ejemplos/01-hello.arita
# Linux
sha256sum ejemplos/01-hello.arita
```

Must hash **raw file bytes**. Current `01-hello.arita`:

`9bac094f245d20b42ef50bd3331803c394bf946c860fa5fcbccc3eacdf1fbc0e`

On successful `arita contract`, sidecar: `target/arita-attest/<id>.attest.json`.

```bash
cargo run -p arita-cli -- contract ejemplos/contracts/contract-hello-attested.json
cargo run -p arita-cli -- attest verify ejemplos/contracts/contract-hello-attested.json
cargo run -p arita-cli -- contract --verify-attest ejemplos/contracts/contract-hello.json
```
