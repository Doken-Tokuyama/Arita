Translation of `012-logic-island-v0.md`; the original is normative. / Traducción de `012-logic-island-v0.md`; el original es el normativo.

# ADR-012 — Logic island F3 v0 (surface sketch)

- **Estado:** **aceptada** (DOC-only)
- **CUT-ID:** `LOGIC-ISLAND-DOC-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO)
- **Relacionados:** ADR-002 (isla propia), ADR-008 (evidence chain), `08-LICENSES-IDNI.md`, `THREAT_MODEL.md`, ROADMAP Fase 3
- **Governance DOC CUT:** Tau/TML inspiration only; zero IDNI; **no crate in the DOC CUT**.
- **IMPL:** CUT `LOGIC-ENGINE-V0` **ACCEPTED** / landed (2026-09-13) — `crates/arita-logic`

## Context

F1/F2 + measure + HIR-CHECK-V0 cubren surface, anti-theater y ownership v0. La **capa 2** de ADR-008 (Logical Island / Evidence Kernel) remains en horizon F3.

ADR-002 ya fija motor **propio**. This ADR sketches el **surface F3 v0** (`spec` / `fact` / `rule` / `query`) y the verdict contract (**UNSAT / query fail → `rejected`**) **without** implementar runtime ni crates.

## Decision (proposal)

### 1. Relation to F1/F2

- F1.1 / F2 **remain** valid; F3 **adds** logic blocks, does not redefine them.
- **Pin GO v0:** **separate logic modules** — un `.arita` of logic (`spec`/`fact`/`rule`/`query`) **no** mezcla `fn`/`main`/`test` in the same file until a CUT later. F2 y F3 coexisten in the repo, no in the same fichero v0.
- Default for AIs remains F1.1 / F2 PACK; F3 only if the task asks for logic / evidence.

### 2. Surface sketch F3 v0 (canonical — draft)

```arita
module demo_logic

spec Reachability {
  // predicate / sort declarations (v0 minimum)
}

fact edge(a, b)
fact edge(b, c)

rule path(X, Y) :- edge(X, Y)
rule path(X, Z) :- edge(X, Y), path(Y, Z)

query path(a, c)   // expect satisfiable / true
query path(c, a)   // expect fail / UNSAT-ish → rejected en measure
```

**Forma (pins v0):**

- **File = logic XOR systems:** no `fn`+`spec` together en v0 (`LOGIC-ISLAND-DOC-20260913`).

**Constructos:**

| Constructo | Rol |
|------------|-----|
| `spec Name { … }` | Contenedor de vocabulario / predicados (v0 puede ser optional o empty) |
| `fact P(args…)` | Hecho base |
| `rule Head :- Body` | Clause (body = conjunction of atoms) |
| `query Atom` | Consulta; resultado alimenta measure |

**Outside v0:** complex negation, aggregation, temporal Tau full, FFI a TML, test UI.

### 3. Semantics / veredictos (alineado ADR-008 + THREAT_MODEL)

| Evento | Veredicto measure / check |
|--------|---------------------------|
| Query **satisfecha** (derivable) | candidato a **accepted** (si el oracle lo exige) |
| Query **fails** / **UNSAT** / contradiction | **`rejected`** |
| Engine absent / timeout / skip | **`inconclusive`** — ****never** promote to accepted |
| “PASS” decorativo without ejecutar el kernel | theater — forbidden |

Estados: `rejected` \| `inconclusive` \| `accepted`. **Nunca** `inconclusive` → `accepted`.

### 4. Pipeline (DOC)

```text
.arita (spec/fact/rule/query)
        ↓ parse/AST (futuro; CUT syntax aparte)
        ↓ lower → Logic IR ARITA (propio)
        ↓ verify (motor propio)
        ↓ UNSAT / query fail → rejected (E03xx draft)
        ↓ (sistemas) emit-Rust sigue ADR-001; isla no sustituye backend
```

### 5. Diagnostics draft (E03xx — no congelados until GO+IMPL)

| Code | English (canonical, draft) |
|------|----------------------------|
| **E0301** | `query failed` |
| **E0302** | `unsatisfiable / contradiction` |
| **E0303** | `unknown predicate or arity` |
| **E0304** | `logic construct not allowed in F3 v0` |

Codes finales = CUT IMPL + tabla EN stable (as E0xxx/E02xx).

### 6. Licencias / anti-IDNI (inamovible)

- Tau/TML inspiration: OK (ADR-002 / `08-LICENSES-IDNI.md`).
- **Forbidden:** copiar, linkar, submodule, o exponer APIs IDNI as runtime ARITA.
- Binding comercial IDNI = **ADR nuevo**, no este.

### 7. Anti-theater

| Forbidden | Why |
|-----------|---------|
| Oracle que imprime “SAT” without motor | Fake evidence |
| Skip de query → accepted | skip ≠ PASS |
| Few-shots with `fact`/`rule` que no corren en kernel | Theater |
| Crates stub `arita-logic` que always Ok | Fake kernel |

### 8. Out of this ADR / this CUT

- **DOC CUT** fue no crate; **LOGIC-ENGINE-V0 landed:** `crates/arita-logic` (zero IDNI).
- No pest/AST F3 until CUT syntax + GO.
- No cambiar F1/F2 surface ni ownership.
- No attestation / content-addressing (horizon ADR-008).

## Checklist GO (Ingeniero) — cerrado

- [x] Surface sketch `spec`/`fact`/`rule`/`query` OK
- [x] Pin: **separate logic modules** en v0 (no mix `fn`+`spec` same file)
- [x] UNSAT / query fail → `rejected` OK
- [x] Tau/TML inspiration only + zero IDNI OK
- [x] No crate en this CUT OK
- [x] Alineado ADR-002 / ADR-008 / THREAT_MODEL OK
- [x] GO `LOGIC-ISLAND-DOC-20260913` → **accepted**; Logic agent **HOLD** until IMPL CUT

## Estado IMPL

- CUT `LOGIC-ENGINE-V0` **ACCEPTED** / **landed** (Ingeniero, 2026-09-13): `crates/arita-logic` (pest/AST + IR + verify).
- `arita-syntax` F1/F2 **untouched**.
- Pins DOC: separate logic modules; UNSAT/query fail → `rejected`; never inconclusive→accepted; zero IDNI.

## Enlaces

- `DOC/ADR/002-isla-logica-own.md`
- `DOC/ADR/008-evidence-architecture.md`
- `DOC/08-LICENSES-IDNI.md`
- `DOC/THREAT_MODEL.md`
- `ROADMAP.md` — Phase 3

## PARK / OUT — Int wrapping & saturating (2026-09-18)

**OUT v0:** facts/rules/queries **do not** model wrapping/saturating `Int` arithmetic nor lower `i64` wrap-release into TML/Tau. The Imperativo wave (E0217, saturating_*) is **orthogonal** and **does not block** this island (zero crate coupling). See `DOC/LOGIC-INT-ARITH-OUT.md`.
