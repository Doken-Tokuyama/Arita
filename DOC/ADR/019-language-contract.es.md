Translation of `019-language-contract.md`; the original is normative. / Traducción de `019-language-contract.md`; el original es el normativo.

# ADR-019 — Language-level `contract { }` (LANG-CONTRACT)

- **Estado:** **aceptada**
- **CUT-ID:** `LANG-CONTRACT-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO)
- **Relacionados:** ADR-017 (JSON contract oracle — **do not break**), ADR-018 (attestation — **do not break**), measure v0
- **Alcance:** Surface syntax `contract <ident> { expect_stdout "…" | expect_reject "E0xxx" }` inside `.arita`; AST `Module.contracts`; CLI `arita contract path.arita`; measure +2 oracles. Real E2E; skip ≠ PASS; contracts **not** emitted to Rust binary.
- **Barra:** Keep existing **41** measure oracles green; grow by **~2** → **~43**; `cargo test --workspace` PASS; clippy `-D warnings` on `arita-syntax` / `arita-codegen` / `arita-hir` / `arita-logic` / `arita-cli`.

## Contexto

ADR-017 pins declarative **JSON** contracts. Language-level contracts co-locate the expect with the program under test so ejemplos can declare their own oracle surface without a sidecar JSON. Attestation fields in surface syntax and nested/target-fn selection remain **OUT**.

## Decisión

### 1. MVP surface (IN)

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
module bad
fn main() -> Io<()> {
  if true { break }
  print("x")
}
contract bad_c {
  expect_reject "E0224"
}
```

Rules:

- One expect **kind** per contract: either one or more `expect_stdout` strings (ordered lines, same normalize as measure) **or** a single `expect_reject "CODE"`.
- Mixing stdout + reject in one contract → parse error.
- `contract` is a top-level item (with `module` / `fn` / `test`); does **not** emit to Rust (`emit_rust` ignores `Module.contracts`).
- HIR may omit contracts; AST `Module.contracts: Vec<Contract>` is the pin.

### 2. CLI

| Input | Behavior |
|-------|----------|
| `arita contract path.json` | ADR-017/018 unchanged |
| `arita contract path.arita` | Parse AST contracts; run same pipeline as JSON (`build`+stdout or `parse_lower_check` reject); print computed `source_sha256` on accept |

Empty contracts / unreadable / invalid → **inconclusive** (exit 2). Fail → exit 1. All contracts in file must accept for exit 0.

### 3. Measure (+2)

| Oracle id | Source | Kind |
|-----------|--------|------|
| `lang-contract-hello` | `ejemplos/contracts/lang-hello.arita` | expect_stdout `hello` |
| `lang-contract-neg-e0224` | `ejemplos/contracts/lang-neg-e0224.arita` | expect_reject `E0224` |

Existing 41 oracles unchanged → measure **~43**.

### 4. OUT (future)

- Nested contracts
- `target: fn <name>` beyond whole-file `main`
- Attestation fields in surface syntax (CLI may still **print** computed hash only)
- Batch directory runner

## Consecuencias

- Pest: `contract_item` / `expect_stdout` / `expect_reject` rules; keyword `contract`.
- `arita-syntax`: `Contract` + `Module.contracts`; parse tests.
- `arita-cli` `contract.rs`: `.arita` dispatch; measure oracles.
- Docs: `ejemplos/contracts/README.md`, `STABLE_VERIFY.md`.
- ADR-017/018 remain JSON/attest only; this CUT is **ADR-019**.
