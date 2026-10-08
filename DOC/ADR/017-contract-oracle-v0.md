# ADR-017 — Contract oracle v0 (JSON contracts + `arita contract`)

- **Estado:** **aceptada**
- **CUT-ID:** `CONTRACT-V0-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO)
- **Relacionados:** ADR-016 (break/continue — **do not reuse 016**), measure v0, ADR-012 (logic island)
- **Alcance:** External JSON contract files under `ejemplos/contracts/`; CLI `arita contract <path.json>`; measure `ContractOracle` list (~+3 oracles). Real E2E; skip ≠ PASS; inconclusive ≠ accepted.
- **Barra:** Keep existing 36 measure oracles green; grow Mac/Linux measure by ~3 contract oracles; `cargo test --workspace` PASS; clippy `-D warnings` on workspace crates.

## Contexto

Measure already pins ejemplos / neg / logic / clippy oracles. Contracts add a **declarative** layer: a JSON file names a source `.arita` and either expected stdout lines or an expected reject code. This is **not** language-level `contract { }` syntax, attestation, content-addressing, or crypto signatures (deferred).

## Decisión

### 1. MVP schema (IN)

Exactly one of `expect_stdout` or `expect_reject`. Required: `id`, `source`.

```json
{
  "id": "contract-hello",
  "source": "ejemplos/01-hello.arita",
  "expect_stdout": ["hello"]
}
```

```json
{
  "id": "contract-neg-e0224",
  "source": "ejemplos/f2.3/neg/e0224-break-outside.arita",
  "expect_reject": "E0224"
}
```

Invalid schema (missing fields, both expects, neither expect, wrong types) → **inconclusive** / CLI exit 2 — never fake PASS.

### 2. CLI `arita contract <path-to-json>`

| Expect | Behavior | Exit |
|--------|----------|------|
| `expect_stdout` | `build` + run (same as measure ejemplo); compare `normalize_stdout` | 0 match / 1 mismatch or build fail |
| `expect_reject` | `parse_lower_check` must fail containing code | 0 if rejected with code / 1 otherwise |
| missing source / unreadable JSON / bad schema | clear inconclusive messaging | **2** (never PASS) |

### 3. Measure wiring

`CONTRACT_ORACLES` (at least):

| Oracle id | JSON | Kind |
|-----------|------|------|
| `contract-hello` | `ejemplos/contracts/contract-hello.json` | stdout `hello` |
| `contract-f23-break` | `ejemplos/contracts/contract-f23-break.json` | stdout `0`/`1` (while-break) |
| `contract-neg-e0224` | `ejemplos/contracts/contract-neg-e0224.json` | reject `E0224` |

Verdict **accepted** iff the contract check passes. Existing 36 oracles unchanged and must stay green → measure grows **36 → ~39**.

### 4. Pipeline reuse

```text
JSON → parse schema
  expect_stdout → build → run → normalize_stdout compare
  expect_reject → parse_lower_check → must contain code
```

Reuse `build`, `normalize_stdout`, `parse_lower_check` from CLI/measure helpers.

### 5. OUT (future — DOC note only)

- Attestation / content-addressing of contract files
- Crypto signatures over contract + artifact digests
- Language-level `contract { … }` syntax inside `.arita`
- Batch `arita contract ejemplos/contracts/` directory runner (beyond single path)

## Consecuencias

- New module `crates/arita-cli/src/contract.rs` + unit tests (schema error, stdout match, reject match).
- Docs: `ejemplos/README.md`, `STABLE_VERIFY.md` note Mac measure +3 oracles.
- ADR-016 remains break/continue only; this CUT is **ADR-017**.
