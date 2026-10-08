# ADR-013 — Surface `contract` / `fn` / `oracle` (horizonte evidencia)

- **Estado:** **aceptada** (DOC-only)
- **CUT-ID:** `CONTRACT-ORACLE-DOC-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO) ← brief ADR-008; nombre = **ARITA**
- **Relacionados:** ADR-008 (evidence chain), ADR-005/006/012 (F1–F3), `THREAT_MODEL.md`, `arita measure`
- **Gobernanza:** **DOC-only**. **No crate / pest / ejemplos decorativos.** Extiende F1–F3 solo con CUT+GO de impl. Nombre = **ARITA** (no Veyra).

## Contexto

ADR-008 fija tres capas y un horizonte de surface: `module` / `type` / `contract` / `fn` / `oracle`, con **contract-before-impl** y `measure` sobre los **mismos** contratos/oráculos.

Hoy ya existen:

| Capa | Realidad v0 |
|------|-------------|
| F1.1 | `module` + `fn main` + `print` (ADR-005) |
| F2 | ownership + std + tests (ADR-006/007/009) |
| F3 | `spec`/`fact`/`rule`/`query` (ADR-012 + `arita-logic`) |
| Measure | oráculos E2E + neg E02xx/E021x + clippy |

Falta el **sketch DOC** de cómo `contract` + `oracle` declarativos se apoyan encima sin theater ni segundo harness decorativo.

## Decisión propuesta

### 1. Principios (inamovibles en este ADR)

1. **Contract-before-impl:** el contrato/oráculo se declara **antes** (o junto) a la impl; measure evalúa ese contrato, no un demo paralelo.
2. **Same contracts:** `arita measure` / `arita test` / `arita logic` usan los **mismos** artefactos versionados que el compilador aceptó — no un “suite de marketing”.
3. **Layer, don’t replace:** F1–F3 siguen siendo el camino v0; este surface es **F4-ish / horizonte** hasta CUT IMPL.
4. **No fake PASS:** skip / inconclusive / harness roto **nunca** → `accepted` (THREAT_MODEL).
5. **DOC-only ahora:** sin grammar, sin crate, sin ejemplos decorativos de `contract` hasta IMPL GO + oráculos reales.

### 2. Sketch de forma (canónico — draft, no pest)

```arita
module payments

type Amount = Int

contract Transfer {
  // pre/post / properties (v0: declarativo; semántica exacta = CUT IMPL)
  requires amount > 0
  ensures balance_out == balance_in - amount
}

fn transfer(amount: Amount) -> Io<()> 
  where Transfer
{
  // body F2-shaped; efectos explícitos
  ...
}

oracle transfer_positive {
  // mismo contrato / entradas de evidencia que measure ejecutará
  run transfer(1)
  expect /* stdout / query / post */ ...
}
```

**Pins de forma (ajustables en GO):**

| Constructo | Rol |
|------------|-----|
| `type` | alias / nominal liviano (horizonte; F2 ya tiene tipos built-in) |
| `contract Name { … }` | obligaciones pre/post o propiedades nombradas |
| `fn … where Contract` | impl ligada a contrato (contract-before-impl) |
| `oracle Name { … }` | evidencia ejecutable **versionada** (run + expect) ligada al mismo contrato |

### 3. Cómo se apila sobre F1–F3

```text
F1.1  print/main          →  oráculo stdout (ya)
F2    let/fn/test         →  oráculo build+run + E02xx (ya)
F3    spec/fact/rule/query→  arita logic + E03xx (ya)
F?    contract + oracle   →  unifica “qué promete” + “cómo se mide”
         │
         ▼
    Evidence Kernel (ADR-008 capa 2) + measure (capa 3)
```

| Surface existente | Rol respecto a contract/oracle |
|-------------------|--------------------------------|
| F1.1 / F2 `fn` | Cuerpo de impl; puede quedar referenciado por `fn … where C` |
| F2 `test` / `assert` | Subconjunto de evidencia unitaria; **no** sustituye `oracle` de aceptación |
| F3 `query` | Puede ser **backend** de un `oracle` lógico (UNSAT → rejected) |
| `ejemplos/` + measure | Hoy ya son oráculos; el futuro `oracle` **nombra** y **versiona** esos contratos en surface |

**Regla anti-doble-harness:** no crear `oracle` que mida un binario distinto al emitido por el pipeline de `arita build` / `arita logic` del mismo módulo/contrato.

### 4. Measure = mismos contratos

| Hoy | Mañana (tras IMPL GO) |
|-----|------------------------|
| Tablas embebidas en measure + `ejemplos/` | Declaraciones `oracle` en `.arita` (o manifiesto generado desde ellas) |
| clippy-workspace + neg E02/E021 | + oráculos de contrato fallidos → `rejected` |
| JSON verdict rejected/inconclusive/accepted | Sin cambio de estados; **misma** barra THREAT_MODEL |

Si un `oracle` no puede ejecutarse → **`inconclusive`**, nunca `accepted`.

### 5. Diagnósticos draft (E04xx — no congelados)

| Code | English (draft) |
|------|-----------------|
| **E0401** | `contract not satisfied` |
| **E0402** | `oracle expectation mismatch` |
| **E0403** | `fn missing required contract` |
| **E0404** | `oracle not bound to contract / artifact` |

### 6. Anti-theater

| Prohibido | Por qué |
|-----------|---------|
| `oracle` que solo imprime “passed” | Fake evidence (THREAT_MODEL) |
| Contrato sin camino de measure | Theater documental |
| Segundo harness que no usa emit/logic reales | Bypass de cadena de evidencia |
| Few-shots inventados sin `.arita` E2E | skip ≠ PASS |

### 7. Fuera de este ADR / este CUT

- **No crate**, no pest, no ejemplos `contract` decorativos.
- No renombrar ARITA / no tirar F1–F3.
- No IDNI.
- No attestation / hashing (horizonte ADR-008).

## Checklist GO (Ingeniero) — cerrado

- [x] Sketch `contract` / `fn where` / `oracle` OK
- [x] Layering F1–F3 OK (extiende, no sustituye)
- [x] measure = mismos contratos OK
- [x] DOC-only / no crate / no pest / no ejemplos decorativos OK
- [x] Anti-theater + veredictos OK
- [x] GO `CONTRACT-ORACLE-DOC-20260913` → **aceptada**; IMPL = CUTs posteriores

## Enlaces

- `DOC/ADR/008-evidence-architecture.md`
- `DOC/THREAT_MODEL.md`
- `DOC/ADR/005-f1.1-surface-freeze.md`, `DOC/ADR/006-f2-ownership-surface.md`, `DOC/ADR/012-logic-island-v0.md`
- `ROADMAP.md`
