Translation of `018-attestation-v0.md`; the original is normative. / Traducción de `018-attestation-v0.md`; el original es el normativo.

# ADR-018 — Attestation v0 (source sha256 + contract sidecar)

- **Estado:** **aceptada**
- **CUT-ID:** `ATTEST-V0-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO)
- **Relacionados:** ADR-017 (contract oracle — **do not overwrite 017**), measure v0
- **Alcance:** Optional `source_sha256` on contract JSON; sidecar attestation under `target/arita-attest/`; CLI `arita attest verify` / `arita contract --verify-attest`; measure +2 oracles. Real E2E; skip ≠ PASS; hash mismatch → FAIL (never PASS).
- **Barra:** Keep existing ~39 measure oracles green; grow by ~2 attest oracles → **~41**; `cargo test --workspace` PASS; clippy `-D warnings` on workspace crates.

## Contexto

CONTRACT-V0 pins declarative JSON contracts. ATTEST-V0 binds a contract to the **raw bytes** of its `.arita` source via SHA-256 so silent source tampering cannot still PASS. This is **not** crypto signatures, notarization, remote attestation, or content-addressed IR/binary chains (deferred).

## Decisión

### 1. Optional `source_sha256` on contract JSON

```json
{
  "id": "contract-hello-attested",
  "source": "ejemplos/01-hello.arita",
  "source_sha256": "<hex lowercase of raw file bytes>",
  "expect_stdout": ["hello"]
}
```

- Hash = SHA-256 of **raw source file bytes** (hex, lowercase). Not normalized text.
- `arita contract`: if field **present**, recompute and compare before expect — mismatch → **rejected** (exit 1). If **absent**, run as ADR-017 and **print** computed hash.
- Invalid type / empty string for `source_sha256` → **inconclusive** (never PASS).

### 2. Sidecar on successful `arita contract`

On accept, write `target/arita-attest/<contract_id>.attest.json`:

| Field | Meaning |
|-------|---------|
| `version` | `ATTEST-V0` |
| `contract_id` | contract `id` |
| `source_path` | relative source path |
| `source_sha256` | computed hex |
| `expect_kind` | `stdout` \| `reject` |
| `expect_payload` | stdout lines array or reject code string |
| `tool_version` | CLI version string (`0.1.0-f1`) |

### 3. Verify mode

`arita attest verify <contract.json>` **or** `arita contract --verify-attest <contract.json>`:

1. Resolve expected hash from embedded `source_sha256` **or** existing sidecar (must exist from a prior successful contract run).
2. Missing expected hash when required → **inconclusive** (exit 2) — never PASS.
3. Re-read source bytes, recompute sha256; mismatch → **rejected** (exit 1).
4. Contract expect must still pass (same pipeline as `arita contract`).

Tamper source bytes without updating hash → verify **FAIL**.

### 4. Measure wiring (+2)

| Oracle id | JSON | Measure accepted iff |
|-----------|------|----------------------|
| `contract-hello-attested` | `contract-hello-attested.json` | hash matches + stdout `hello` |
| `contract-hello-bad-hash` | `contract-hello-bad-hash.json` | contract correctly **rejects** (wrong sha) — neg-style |

Existing CONTRACT-V0 oracles unchanged (~39 → ~41).

### 5. Deps

`sha2` (crates.io) + minimal std hex encode in `arita-cli`. No serde.

### 6. OUT (future — DOC note only)

- Crypto signatures / notarization over contract + digests
- Remote attestation
- Content-addressed IR / binary chain
- Language-level attestation syntax

## Consecuencias

- Modules: `crates/arita-cli/src/attest.rs` + extensions in `contract.rs` / `main.rs`.
- Examples: `ejemplos/contracts/contract-hello-attested.json`, `contract-hello-bad-hash.json`.
- Docs: `ejemplos/README.md`, `ejemplos/contracts/README.md`, `STABLE_VERIFY.md`.
- ADR-017 remains contract-oracle only; this CUT is **ADR-018**.
