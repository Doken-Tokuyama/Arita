# ADR-012 — Isla lógica F3 v0 (surface sketch)

- **Estado:** **aceptada** (DOC-only)
- **CUT-ID:** `LOGIC-ISLAND-DOC-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft) + Ingeniero Rust (GO)
- **Relacionados:** ADR-002 (isla propia), ADR-008 (evidence chain), `08-LICENSES-IDNI.md`, `THREAT_MODEL.md`, ROADMAP Fase 3
- **Gobernanza DOC CUT:** Tau/TML inspiración only; cero IDNI; **no crate en el DOC CUT**.
- **IMPL:** CUT `LOGIC-ENGINE-V0` **ACCEPTED** / landed (2026-09-13) — `crates/arita-logic`

## Contexto

F1/F2 + measure + HIR-CHECK-V0 cubren surface, anti-theater y ownership v0. La **capa 2** de ADR-008 (Logical Island / Evidence Kernel) sigue en horizonte F3.

ADR-002 ya fija motor **propio**. Este ADR esboza el **surface F3 v0** (`spec` / `fact` / `rule` / `query`) y el contrato de veredicto (**UNSAT / query fail → `rejected`**) **sin** implementar runtime ni crates.

## Decisión propuesta

### 1. Relación con F1/F2

- F1.1 / F2 **siguen** válidos; F3 **añade** bloques lógicos, no los redefine.
- **Pin GO v0:** **módulos lógicos separados** — un `.arita` de lógica (`spec`/`fact`/`rule`/`query`) **no** mezcla `fn`/`main`/`test` en el mismo archivo hasta un CUT posterior. F2 y F3 coexisten en el repo, no en el mismo fichero v0.
- Default para IAs sigue siendo F1.1 / F2 PACK; F3 solo si la tarea pide lógica / evidencia.

### 2. Surface sketch F3 v0 (canónico — draft)

```arita
module demo_logic

spec Reachability {
  // declaraciones de predicados / sortes (v0 mínimo)
}

fact edge(a, b)
fact edge(b, c)

rule path(X, Y) :- edge(X, Y)
rule path(X, Z) :- edge(X, Y), path(Y, Z)

query path(a, c)   // expect satisfiable / true
query path(c, a)   // expect fail / UNSAT-ish → rejected en measure
```

**Forma (pins v0):**

- **Archivo = lógica XOR sistemas:** no `fn`+`spec` juntos en v0 (`LOGIC-ISLAND-DOC-20260913`).

**Constructos:**

| Constructo | Rol |
|------------|-----|
| `spec Name { … }` | Contenedor de vocabulario / predicados (v0 puede ser opcional o vacío) |
| `fact P(args…)` | Hecho base |
| `rule Head :- Body` | Cláusula (cuerpo = conjunción de átomos) |
| `query Atom` | Consulta; resultado alimenta measure |

**Fuera de v0:** negación compleja, agregación, temporal Tau completo, FFI a TML, UI de prueba.

### 3. Semántica / veredictos (alineado ADR-008 + THREAT_MODEL)

| Evento | Veredicto measure / check |
|--------|---------------------------|
| Query **satisfecha** (derivable) | candidato a **accepted** (si el oráculo lo exige) |
| Query **falla** / **UNSAT** / contradicción | **`rejected`** |
| Motor ausente / timeout / skip | **`inconclusive`** — **nunca** promover a accepted |
| “PASS” decorativo sin ejecutar el kernel | theater — prohibido |

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

### 5. Diagnósticos draft (E03xx — no congelados hasta GO+IMPL)

| Code | English (canonical, draft) |
|------|----------------------------|
| **E0301** | `query failed` |
| **E0302** | `unsatisfiable / contradiction` |
| **E0303** | `unknown predicate or arity` |
| **E0304** | `logic construct not allowed in F3 v0` |

Códigos finales = CUT IMPL + tabla EN estable (como E0xxx/E02xx).

### 6. Licencias / anti-IDNI (inamovible)

- Inspiración Tau/TML: OK (ADR-002 / `08-LICENSES-IDNI.md`).
- **Prohibido:** copiar, linkar, submodule, o exponer APIs IDNI como runtime ARITA.
- Binding comercial IDNI = **ADR nuevo**, no este.

### 7. Anti-theater

| Prohibido | Por qué |
|-----------|---------|
| Oráculo que imprime “SAT” sin motor | Fake evidence |
| Skip de query → accepted | skip ≠ PASS |
| Few-shots con `fact`/`rule` que no corren en kernel | Theater |
| Crates stub `arita-logic` que siempre Ok | Fake kernel |

### 8. Fuera de este ADR / este CUT

- **DOC CUT** fue sin crate; **LOGIC-ENGINE-V0 landed:** `crates/arita-logic` (cero IDNI).
- No pest/AST F3 hasta CUT syntax + GO.
- No cambiar F1/F2 surface ni ownership.
- No attestation / content-addressing (horizonte ADR-008).

## Checklist GO (Ingeniero) — cerrado

- [x] Surface sketch `spec`/`fact`/`rule`/`query` OK
- [x] Pin: **módulos lógicos separados** en v0 (no mix `fn`+`spec` mismo file)
- [x] UNSAT / query fail → `rejected` OK
- [x] Tau/TML inspiración only + cero IDNI OK
- [x] No crate en este CUT OK
- [x] Alineado ADR-002 / ADR-008 / THREAT_MODEL OK
- [x] GO `LOGIC-ISLAND-DOC-20260913` → **aceptada**; Logic agent **HOLD** hasta IMPL CUT

## Estado IMPL

- CUT `LOGIC-ENGINE-V0` **ACCEPTED** / **landed** (Ingeniero, 2026-09-13): `crates/arita-logic` (pest/AST + IR + verify).
- `arita-syntax` F1/F2 **untouched**.
- Pins DOC: módulos lógicos separados; UNSAT/query fail → `rejected`; never inconclusive→accepted; cero IDNI.

## Enlaces

- `DOC/ADR/002-isla-logica-propia.md`
- `DOC/ADR/008-evidence-architecture.md`
- `DOC/08-LICENSES-IDNI.md`
- `DOC/THREAT_MODEL.md`
- `ROADMAP.md` — Fase 3

## PARK / OUT — Int wrapping & saturating (2026-09-18)

**OUT v0:** facts/rules/queries **no** modelan aritmética `Int` wrapping/saturating ni bajan `i64` wrap release a TML/Tau. La oleada Imperativo (E0217, saturating_*) es **ortogonal** y **no bloquea** esta isla (cero acoplamiento de crates). Ver `DOC/LOGIC-INT-ARITH-OUT.md`.
