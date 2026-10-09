Translation of `008-evidence-architecture.md`; the original is normative. / Traducción de `008-evidence-architecture.md`; el original es el normativo.

# ADR-008 — Arquitectura de cadena de evidence (ARITA)

- **Estado:** **aceptada**
- **CUT-ID:** `EVIDENCE-CHAIN-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) ← brief amigo de <person>; nombre = **ARITA** (no Veyra)
- **Relacionados:** `DOC/THREAT_MODEL.md`, ADR-001 (emit-Rust), ADR-002 (isla propia), ADR-005/006/007 (F1/F2), `ROADMAP.md`
- **Gobernanza:** no renombrar proyecto; **no** borrar F1/F2; Tau/TML = inspiración semántica (**cero código IDNI**)

## Context

ARITA must not degenerate into a “code generator with showcase tests”. The external brief proposes an **executable evidence chain**. This ADR adopts that architecture **under the ARITA name** and folds the F1/F2/emit-Rust/anti-theater work as **v0** toward contracts and oracles.

## Principio

> No existe “correcto” por parecerlo. Solo **`accepted`** when oracles declarados, reproducibles y anti-bypass aprueban el artefacto exacto.  
> Estados: `rejected` | `inconclusive` | `accepted`. **Nunca** `inconclusive` → `accepted`. Ver `THREAT_MODEL.md`.

## Decision — three layers

```text
          intención humana / tarea de agente
                        │
                        ▼
┌────────────────────────────────────────────────┐
│  1. ARITA Surface                               │
│  Lenguaje AI-first: tipos, contratos, efectos,  │
│  límites, propiedades y objetivos declarativos  │
│  (hoy: F1.1 + F2 ownership — ADR-005/006/007)   │
└───────────────────────┬────────────────────────┘
                        │ AST + IR tipado
                        ▼
┌────────────────────────────────────────────────┐
│  2. Logical Island / Evidence Kernel            │
│  Motor propio inspirado en Tau + TML            │
│  - Reglas de admisibilidad                      │
│  - Satisfacibilidad y contradicciones           │
│  - Dependencias y procedencia                   │
│  - Oráculos y obligaciones de prueba            │
│  - Políticas anti-bypass                        │
│  (hoy: anti-theater + E0xxx; F3 isla `spec`…) │
└───────────────────────┬────────────────────────┘
                        │ Proof/Evidence IR
                        ▼
┌────────────────────────────────────────────────┐
│  3. Backends verificables + measure             │
│  Rust inicialmente (ADR-001); luego WASM/otros │
│  - Código generado / harness / sandbox          │
│  - Artefactos con hash (horizonte)              │
│  - `arita measure` = mismos oráculos/contratos  │
└────────────────────────────────────────────────┘
```

### Capa 1 — Surface

- AI-first, forma canonical corta; peligroso = explicit y costoso.
- **v0:** F1.1 (`module`/`main`/`print`) + F2 (`let`, tipos, ownership rules en DOC).
- Horizonte (brief): `module` / `type` / `contract` / `fn` / `oracle` — **no** sustituye F1/F2; los extends when haya CUT+GO.
- Deferred principles (later phases): **contract-before-impl**; effects explicit; deny lists; `measure` evaluates el **same** contrato/oracles (no a showcase benchmark).

### Capa 2 — Logical Island / Evidence Kernel

- Motor **own**; Tau/TML only inspiration (ADR-002 / `08-LICENSES-IDNI.md`).
- Decides what is derived, what contradicts invariants, what evidence is missing.
- **v0:** denylist anti-theater, diagnostics stable, measure FAIL/skip≠PASS.
- Horizonte F3+: `spec`/`fact`/`rule`/`query`, UNSAT → `rejected`.

### Capa 3 — Backends + measure

- Backend MVP: **emit-Rust** (ADR-001).
- **`measure` does not invent success:** evaluates el **same contrato** the compiler accepted, with los **same oracles** versioned (`ejemplos/`, suites E2E).
- **v0 (verificado):** `arita measure` = oracles `ejemplos/` 01–05 + `f2/01` (build+run+stdout) **+** `clippy-workspace` mandatory; overall `accepted` **needs clippy on host** (without clippy → `inconclusive`).

## Plegado F1/F2 → evidence (v0 path)

| Pieza existente | Rol en la cadena |
|-----------------|------------------|
| ADR-001 emit-Rust | Backend verificable v0 |
| ADR-002 own island | Logic kernel (without IDNI) |
| ADR-004/007 AST | IR tipado surface→checker |
| ADR-005 F1.1 + PACK | Surface minimal + few-shots reales |
| ADR-006 F2 ownership | Surface tipada + anti-theater E02xx |
| `ejemplos/` E2E | Oracles compile+run (not decorative) |
| `THREAT_MODEL.md` | Catalog theater + estados de veredicto |

## Out of this ADR

- Rename a Veyra u otro nombre
- Tirar F1/F2 o reset of the repo
- Link/submodule code IDNI
- Implementar kernel full / attestation en this CUT (DOC only)

## Checklist GO (Ingeniero) — cerrado

- [x] Diagrama 3 capas OK (nombre ARITA)
- [x] Plegado F1/F2 as v0 OK
- [x] measure = mismos oracles/contratos OK
- [x] Alineado with `THREAT_MODEL.md`
- [x] GO `EVIDENCE-CHAIN-20260913` → accepted + Docs index

## Enlaces

- `DOC/THREAT_MODEL.md`
- `DOC/00-VISION.md`, `DOC/02-ARCHITECTURE.md`
- `ROADMAP.md`
