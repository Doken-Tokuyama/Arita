Español | [English](README.md)

# Oráculos de contrato (CONTRACT-V0 + ATTEST-V0 + LANG-CONTRACT + TARGET-FN)

Contratos JSON para `arita contract <path.json>` y measure `ContractOracle` (ADR-017).
`source_sha256` opcional (ADR-018 / ATTEST-V0): SHA-256 hex de los bytes **crudos** del `.arita`.
`contract { }` a nivel de lenguaje dentro de `.arita` (ADR-019 / LANG-CONTRACT): mismo pipeline de expect; no se emite a Rust.
`target` opcional (ADR-020): JSON `"target": "fn_name"` / lenguaje `target <ident>`; por defecto `main`.
Cuando `target ≠ main`, codegen **reescribe la entrada** para que el Rust `fn main` llame a `target()` (el `main` de usuario → `__arita_user_main`).
`expect_reject` ignora `target` (comprobación de fichero completo). Target ausente / no-`Io<()>` → **E0230**.

| Archivo | id | Kind |
|------|-----|------|
| `contract-hello.json` | `contract-hello` | expect_stdout `hello` |
| `contract-f23-break.json` | `contract-f23-break` | expect_stdout `0` / `1` |
| `contract-neg-e0224.json` | `contract-neg-e0224` | expect_reject `E0224` |
| `contract-hello-attested.json` | `contract-hello-attested` | stdout + `source_sha256` correcto |
| `contract-hello-bad-hash.json` | `contract-hello-bad-hash` | sha incorrecto → el contrato **rechaza**; measure **accepted** iff reject |
| `lang-hello.arita` | `lang-contract-hello` | language `expect_stdout "hello"` |
| `lang-neg-e0224.arita` | `lang-contract-neg-e0224` | language `expect_reject "E0224"` |
| `lang-target-helper.arita` | `lang-contract-target-helper` | `target helper` → stdout `from-helper` (not `from-main`) |
| `contract-target-helper.json` | `contract-target-helper` | JSON `target: helper` same source |

Exactamente un kind de expect por contrato (`expect_stdout` lines **or** `expect_reject`). Schema inválido → inconclusive (nunca PASS).

## Surface de lenguaje (ADR-019)

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

## Hash (sincronización Mac / Linux)

```bash
# macOS
shasum -a 256 ejemplos/01-hello.arita
# Linux
sha256sum ejemplos/01-hello.arita
```

Debe hashear los **bytes crudos del fichero**. `01-hello.arita` actual:

`9bac094f245d20b42ef50bd3331803c394bf946c860fa5fcbccc3eacdf1fbc0e`

Tras un `arita contract` con éxito, sidecar: `target/arita-attest/<id>.attest.json`.

```bash
cargo run -p arita-cli -- contract ejemplos/contracts/contract-hello-attested.json
cargo run -p arita-cli -- attest verify ejemplos/contracts/contract-hello-attested.json
cargo run -p arita-cli -- contract --verify-attest ejemplos/contracts/contract-hello.json
```
