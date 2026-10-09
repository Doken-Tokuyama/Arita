Translation of `020-contract-target-fn.md`; the original is normative. / Traducción de `020-contract-target-fn.md`; el original es el normativo.

# ADR-020 — Contract `target: fn` (entry rewrite)

- **Estado:** **aceptada**
- **CUT-ID:** `CONTRACT-TARGET-FN-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO)
- **Relacionados:** ADR-017 (JSON contracts), ADR-018 (attestation — **do not revive attestation-sign / ed25519 / keys**), ADR-019 (language `contract { }`)
- **Alcance:** Optional contract entry `target` (JSON `"target": "fn_name"` / language `target <ident>`); default `main`. When `target ≠ main`, codegen emits a thin Rust `fn main() { target(); }` and demotes the user `main` body to `__arita_user_main`. Real E2E; skip ≠ PASS; measure ~43 → ~44–45.
- **Barra:** Keep existing ~43 measure oracles green; `cargo test --workspace` PASS; clippy `-p arita-syntax -p arita-codegen -p arita-hir -p arita-logic -p arita-cli -- -D warnings`. Never add demo keys / `keys/` / ed25519.

## Contexto

ADR-019 pins whole-file language contracts (entry always `main`). Callers need to oracle a helper `fn` that prints differently from `main` without running `main`'s side effects. Attestation-sign / crypto remain OUT (ADR-018 deferred crypto; this CUT does not touch attest signing).

## Decisión

### 1. Approach — **entry rewrite** (chosen)

Prefer rewrite over “treat target as sole emit”:

1. Parse/HIR unchanged for the module (still requires `fn main() -> Io<()>` with a usable print).
2. Non-main `fn foo() -> Io<()>` with **empty params** is allowed (MVP entry-shaped helpers; Int helpers unchanged).
3. For `expect_stdout` with `target = T` where `T ≠ "main"`:
   - Resolve `T` in `Module.functions`; missing or not `fn() -> Io<()>` → **E0230**.
   - `emit_rust_with_entry(module, T)`:
     - Emit all non-`main` fns as usual (including `T`).
     - Emit user `main` as `__arita_user_main` (not Rust entry).
     - Emit thin `fn main() { T(); }`.
   - Build + run that binary; compare stdout to expect (helper output only).
4. `target` default / absent ⇒ `"main"` ⇒ existing `emit_rust` behavior (no wrapper).
5. **`expect_reject`:** `target` is **ignored** (whole-file `parse_lower_check`); documented here and in ejemplos README.

### 2. Surfaces

**JSON (ADR-017 extension):**

```json
{
  "id": "contract-target-helper",
  "source": "ejemplos/contracts/lang-target-helper.arita",
  "target": "helper",
  "expect_stdout": ["from-helper"]
}
```

**Language (ADR-019 extension):**

```arita
contract c {
  target helper
  expect_stdout "from-helper"
}
```

AST: `Contract.target: Option<String>` (`None` ⇒ main).

### 3. E0230

Stable code **E0230** when the named target fn is missing, or is not MVP-shaped `fn name() -> Io<()>` (empty params). Surfaced on contract stdout build path (not invent PASS).

### 4. Measure

| Oracle id | Source | Kind |
|-----------|--------|------|
| `lang-contract-target-helper` | `ejemplos/contracts/lang-target-helper.arita` | expect_stdout via `target helper` |

Optional JSON twin may share the same `.arita` source. Existing 43 oracles unchanged → **~44–45**.

### 5. OUT (future)

- Multiple targets / batch
- Non-`Io<()>` targets
- Crypto / attestation-sign / ed25519 / keys directory / demo secrets

## Consecuencias

- Pest: `contract_target` / keyword `target`; AST + JSON `target`.
- `arita-codegen`: `emit_rust_with_entry`; `emit_rust` = entry `main`.
- `arita-cli` contract/build: pass entry into emit; E0230 on resolve failure.
- Docs: this ADR, `ejemplos/contracts/README.md`, `STABLE_VERIFY.md`.
- Do **not** revive attestation-sign.
