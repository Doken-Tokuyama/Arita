# ADR-008 — Arquitectura de cadena de evidencia (ARITA)

- **Estado:** **aceptada**
- **CUT-ID:** `EVIDENCE-CHAIN-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) ← brief amigo de <person>; nombre = **ARITA** (no Veyra)
- **Relacionados:** `DOC/THREAT_MODEL.md`, ADR-001 (emit-Rust), ADR-002 (isla propia), ADR-005/006/007 (F1/F2), `ROADMAP.md`
- **Gobernanza:** no renombrar proyecto; **no** borrar F1/F2; Tau/TML = inspiración semántica (**cero código IDNI**)

## Contexto

ARITA no debe degenerar en “generador de código con tests de escaparate”. El brief externo propone una **cadena de evidencia ejecutable**. Este ADR adopta esa arquitectura **bajo el nombre ARITA** y pliega el trabajo F1/F2/emit-Rust/anti-theater como **v0** hacia contratos y oráculos.

## Principio

> No existe “correcto” por parecerlo. Solo **`accepted`** cuando oráculos declarados, reproducibles y anti-bypass aprueban el artefacto exacto.  
> Estados: `rejected` | `inconclusive` | `accepted`. **Nunca** `inconclusive` → `accepted`. Ver `THREAT_MODEL.md`.

## Decisión — tres capas

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

- AI-first, forma canónica corta; peligroso = explícito y costoso.
- **v0:** F1.1 (`module`/`main`/`print`) + F2 (`let`, tipos, ownership rules en DOC).
- Horizonte (brief): `module` / `type` / `contract` / `fn` / `oracle` — **no** sustituye F1/F2; los extiende cuando haya CUT+GO.
- Principios diferidos (fases posteriores): **contract-before-impl**; efectos explícitos; deny lists; `measure` evalúa el **mismo** contrato/oráculos (no un benchmark de escaparate).

### Capa 2 — Logical Island / Evidence Kernel

- Motor **propio**; Tau/TML solo inspiración (ADR-002 / `08-LICENSES-IDNI.md`).
- Decide qué se deriva, qué contradice invariantes, qué evidencia falta.
- **v0:** denylist anti-theater, diagnósticos estables, measure FAIL/skip≠PASS.
- Horizonte F3+: `spec`/`fact`/`rule`/`query`, UNSAT → `rejected`.

### Capa 3 — Backends + measure

- Backend MVP: **emit-Rust** (ADR-001).
- **`measure` no inventa éxito:** evalúa el **mismo contrato** que el compilador aceptó, con los **mismos oráculos** versionados (`ejemplos/`, suites E2E).
- **v0 (verificado):** `arita measure` = oráculos `ejemplos/` 01–05 + `f2/01` (build+run+stdout) **+** `clippy-workspace` obligatorio; overall `accepted` **needs clippy on host** (sin clippy → `inconclusive`).

## Plegado F1/F2 → evidencia (v0 path)

| Pieza existente | Rol en la cadena |
|-----------------|------------------|
| ADR-001 emit-Rust | Backend verificable v0 |
| ADR-002 isla propia | Kernel lógico (sin IDNI) |
| ADR-004/007 AST | IR tipado surface→checker |
| ADR-005 F1.1 + PACK | Surface mínima + few-shots reales |
| ADR-006 F2 ownership | Surface tipada + anti-theater E02xx |
| `ejemplos/` E2E | Oráculos compile+run (no decorativos) |
| `THREAT_MODEL.md` | Catálogo theater + estados de veredicto |

## Fuera de este ADR

- Rename a Veyra u otro nombre
- Tirar F1/F2 o reset del repo
- Link/submodule código IDNI
- Implementar kernel completo / attestation en este CUT (DOC only)

## Checklist GO (Ingeniero) — cerrado

- [x] Diagrama 3 capas OK (nombre ARITA)
- [x] Plegado F1/F2 como v0 OK
- [x] measure = mismos oráculos/contratos OK
- [x] Alineado con `THREAT_MODEL.md`
- [x] GO `EVIDENCE-CHAIN-20260913` → aceptada + Docs índice

## Enlaces

- `DOC/THREAT_MODEL.md`
- `DOC/00-VISION.md`, `DOC/02-ARCHITECTURE.md`
- `ROADMAP.md`
