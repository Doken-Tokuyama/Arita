Translation of `013-contract-oracle-surface.md`; the original is normative. / Traducción de `013-contract-oracle-surface.md`; el original es el normativo.

# ADR-013 — Surface `contract` / `fn` / `oracle` (horizon evidence)

- **Estado:** **aceptada** (DOC-only)
- **CUT-ID:** `CONTRACT-ORACLE-DOC-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO) ← brief ADR-008; nombre = **ARITA**
- **Relacionados:** ADR-008 (evidence chain), ADR-005/006/012 (F1–F3), `THREAT_MODEL.md`, `arita measure`
- **Gobernanza:** **DOC-only**. **No crate / pest / ejemplos decorativos.** Extiende F1–F3 solo con CUT+GO de impl. Nombre = **ARITA** (no Veyra).

## Context

ADR-008 fija tres capas y un horizon de surface: `module` / `type` / `contract` / `fn` / `oracle`, with **contract-before-impl** y `measure` sobre los **mismos** contratos/oracles.

Hoy ya existen:

| Capa | Realidad v0 |
|------|-------------|
| F1.1 | `module` + `fn main` + `print` (ADR-005) |
| F2 | ownership + std + tests (ADR-006/007/009) |
| F3 | `spec`/`fact`/`rule`/`query` (ADR-012 + `arita-logic`) |
| Measure | oracles E2E + neg E02xx/E021x + clippy |

Missing the **DOC sketch** of how declarative `contract` + `oracle` sit on top without theater or a second decorative harness.

## Decision (proposal)

### 1. Principles (immovable in this ADR)

1. **Contract-before-impl:** the contract/oracle is declared **before** (o junto) a la impl; measure evaluates that contract, not a parallel demo.
2. **Same contracts:** `arita measure` / `arita test` / `arita logic` use the **same** versioned artifacts the compiler accepted — no un “suite de marketing”.
3. **Layer, don’t replace:** F1–F3 remain the v0 path; this surface is **F4-ish / horizon** until CUT IMPL.
4. **No fake PASS:** skip / inconclusive / harness roto **never** → `accepted` (THREAT_MODEL).
5. **DOC-only now:** no grammar, no crate, no decorative examples de `contract` until IMPL GO + oracles reales.

### 2. Sketch de forma (canonical — draft, no pest)

```arita
module payments

type Amount = Int

contract Transfer {
  // pre/post / properties (v0: declarative; exact semantics = CUT IMPL)
  requires amount > 0
  ensures balance_out == balance_in - amount
}

fn transfer(amount: Amount) -> Io<()> 
  where Transfer
{
  // body F2-shaped; explicit effects
  ...
}

oracle transfer_positive {
  // same contract / evidence inputs that measure will run
  run transfer(1)
  expect /* stdout / query / post */ ...
}
```

**Pins de forma (ajustables en GO):**

| Constructo | Rol |
|------------|-----|
| `type` | alias / nominal liviano (horizon; F2 ya tiene tipos built-in) |
| `contract Name { … }` | obligaciones pre/post o propiedades nombradas |
| `fn … where Contract` | impl ligada a contrato (contract-before-impl) |
| `oracle Name { … }` | evidence ejecutable **versionada** (run + expect) ligada al same contrato |

### 3. How it stacks on F1–F3

```text
F1.1  print/main          →  oráculo stdout (ya)
F2    let/fn/test         →  oráculo build+run + E02xx (ya)
F3    spec/fact/rule/query→  arita logic + E03xx (ya)
F?    contract + oracle   →  unifica “qué promete” + “cómo se mide”
         │
         ▼
    Evidence Kernel (ADR-008 capa 2) + measure (capa 3)
```

| Surface existente | Rol regarding contract/oracle |
|-------------------|--------------------------------|
| F1.1 / F2 `fn` | Cuerpo de impl; puede quedar referenciado por `fn … where C` |
| F2 `test` / `assert` | Subset of unit evidence; **does not** replace acceptance `oracle` |
| F3 `query` | May be the **backend** of a logic `oracle` (UNSAT → rejected) |
| `ejemplos/` + measure | Hoy ya son oracles; el future `oracle` **nombra** y **versiona** esos contratos en surface |

**Anti-double-harness rule:** do not create an `oracle` that measures a binary different from the one emitted by the `arita build` / `arita logic` pipeline of the same module/contract.

### 4. Measure = mismos contratos

| Today | Tomorrow (after IMPL GO) |
|-----|------------------------|
| Tablas embebidas en measure + `ejemplos/` | Declaraciones `oracle` en `.arita` (o manifiesto generado from ellas) |
| clippy-workspace + neg E02/E021 | + oracles de contrato fallidos → `rejected` |
| JSON verdict rejected/inconclusive/accepted | No state change; **same** THREAT_MODEL bar |

Si un `oracle` no puede ejecutarse → **`inconclusive`**, never `accepted`.

### 5. Diagnostics draft (E04xx — no congelados)

| Code | English (draft) |
|------|-----------------|
| **E0401** | `contract not satisfied` |
| **E0402** | `oracle expectation mismatch` |
| **E0403** | `fn missing required contract` |
| **E0404** | `oracle not bound to contract / artifact` |

### 6. Anti-theater

| Forbidden | Why |
|-----------|---------|
| `oracle` que only imprime “passed” | Fake evidence (THREAT_MODEL) |
| Contrato without camino de measure | Theater documental |
| Segundo harness que no usa emit/logic reales | Bypass de cadena de evidence |
| Few-shots inventados without `.arita` E2E | skip ≠ PASS |

### 7. Out of this ADR / this CUT

- **No crate**, no pest, no ejemplos `contract` decorativos.
- No renombrar ARITA / no tirar F1–F3.
- No IDNI.
- No attestation / hashing (horizon ADR-008).

## Checklist GO (Ingeniero) — cerrado

- [x] Sketch `contract` / `fn where` / `oracle` OK
- [x] Layering F1–F3 OK (extends, no sustituye)
- [x] measure = mismos contratos OK
- [x] DOC-only / no crate / no pest / no ejemplos decorativos OK
- [x] Anti-theater + veredictos OK
- [x] GO `CONTRACT-ORACLE-DOC-20260913` → **accepted**; IMPL = later CUTs

## Enlaces

- `DOC/ADR/008-evidence-architecture.md`
- `DOC/THREAT_MODEL.md`
- `DOC/ADR/005-f1.1-surface-freeze.md`, `DOC/ADR/006-f2-ownership-surface.md`, `DOC/ADR/012-logic-island-v0.md`
- `ROADMAP.md`
