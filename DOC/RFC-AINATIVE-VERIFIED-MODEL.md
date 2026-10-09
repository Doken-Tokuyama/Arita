# RFC — ARITA AI-native Language → Safe Rust (v0.1)

- **Estado:** **propuesta** (rev. 3 — tesis <person> 2026-09-19: *lenguaje real*, no plataforma)
- **ID:** `RFC-AINATIVE-LANGUAGE-20260919`
- **Supersede:** rev. 2 (perfiles/proporcional) en *énfasis de producto*; perfiles y ownership siguen vigentes como tooling
- **HOLD:** reinicio de CUT/measure **solo tras aviso a <person> y su GO**

## 0. Formulación canónica

> **ARITA** es un lenguaje **AI-native de propósito general** que **compila a Rust**: la IA escribe ARITA como lenguaje primario; ARITA rechaza ambigüedad, humo y operaciones no verificables; emite **proyectos Rust reales** (build / test / clippy / binario / lib / WASM / servicio) que se mantienen **sin depender de la IA**.

### No es

| Anti-producto | Por qué no |
|---------------|------------|
| Plataforma para controlar agentes (MCP/ACP-first) | Tooling de edición; no es el lenguaje |
| DSL de contratos / verificación formal-first | Contratos opcionales y proporcionales al riesgo |
| Wrapper de Cargo / asistente que edita Rust | El artefacto nativo es `.arita`, no “parches sobre Rust” |
| Transpilador de pseudocódigo libre | Sin semántica fija → vuelve el humo |

### Sí es

```text
IA escribe ARITA nativamente
        ↓
 lexer + parser ARITA
        ↓
 AST → HIR tipado → AIR/CFG semántico
        ↓
 checker: tipos, ownership, efectos, concurrencia, anti-fake
        ↓
 emisor determinista
        ↓
 Rust real (workspace Cargo)
        ↓
 cargo build / test / clippy / deploy…
```

## 1. AI-native (diseño contra fallos de LLM)

Pocas formas equivalentes · sintaxis canónica · semántica local · APIs tipadas/versionadas · diagnósticos estructurados reparables · sin “compila pero sorprende” · proyecto visible en manifiesto · un camino preferente por problema frecuente.

| Rust tal cual | Problema para IA | ARITA |
|---------------|------------------|-------|
| Lifetimes / traits densos | Inventa o clone-fix | Ownership canónico + emit Rust correcto |
| `unwrap` / `panic!` / `[]` parcial | Demo falsa | Superficie sin ellos; fallible/total |
| crates.io abierto | API fantasma | Bindings curados + manifiesto tipado |
| `unsafe`/FFI libre | Escape de garantías | Solo paquetes `native`/`ffi` auditados |

## 2. Lenguaje nuevo (rust-like), no dialecto

Sintaxis familiar; **semántica deliberadamente más pequeña, total y canónica**. Backend = Rust safe + interoperabilidad. No heredar macros/lifetimes/`Index` panic como superficie.

## 3. Anti-fake (cadena obligatoria)

1. Parse + resolve (símbolos/bindings reales)  
2. Type + semantic check  
3. Lower IR / AIR  
4. Emit Rust determinista  
5. `cargo` + Clippy  
6. Tests / **scenarios de aceptación** ejecutables  
7. Policies (unsafe/panic/efectos/deps)  
8. **Manifiesto de evidencia** (hash source↔emit↔resultado)

`scenario` / acceptance = especificación **ejecutable** (no `requires`/`ensures` formales por defecto).

## 4. Arquitectura del compilador

AST (fiel al texto) → Resolver → HIR (azúcar fuera, DefId) → Typed HIR → **AIR/CFG** (moves, borrows, async, effects) → policy + test compiler + **Rust emitter estructurado** (no concatenar strings).

`rustc` = **segunda barrera** de ownership/memoria.

## 5. Ownership v0.1 (canónico para IA)

Owned por defecto · move salvo Copy · `borrow` / `borrow mut` · sin refs escapando en structs/return al inicio · `share` / actor / `with_lock` · mutación con alcance léxico · sin `Arc<Mutex<T>>` expuesto.

## 6. Evidencia (CLI/CI primero; UI después)

`target/arita/evidence/<source-hash>.json` con niveles: `parsed` … `tested` / `bounded_verified` / `proved` / `blocked` / `unknown`.  
Prohibido etiquetar DONE si hay stub, aceptación sin test, hash mismatch, dep no declarada, o “verified” cuando solo hay `tested`.

## 7. Core 0.1 (vertical, no self-host primero)

Módulos · record/enum/match · genéricos sencillos · Text/Bytes/números checked · List/Map · Option/Result · ownership simple · files/JSON/CLI · scenarios · emit `#![forbid(unsafe_code)]` · bindings curados · **un** programa de referencia no trivial (CLI/daemon HTTP).

**Criterio de éxito:** tarea a una IA **sin** enseñarle Rust → escribe ARITA → binario Rust → escenarios pasan → si falla, diagnóstico dice qué falta.

## 8. Relación con rev. 2 / ADR-225–230

- Perfiles `safe|service|sandboxed|high-assurance|ffi` siguen como **policy de compilación**.  
- R0–R8 / E0310… = reglas de superficie; **GO reinicio** <person>: cerrar R4 → Core 0.1 (ADR-232).  
- Insert / sugar `[]` / Mutex: contratos de **lenguaje** fallible/total tras reinicio ordenado.

## 9. Checklist

- [x] Tesis lenguaje-first documentada (rev. 3)  
- [x] Arquitecto alinea ROADMAP + faces (2026-09-19)
- [ ] Docs firma índice/THREAT_MODEL  
- [x] Aviso/GO <person> reinicio (R4 → Core 0.1)  
- [x] Repair oracle DOC (REPAIR-ORACLE.md + ADR-231 propuesta)
- [x] Pins Core 0.1 → ADR-232
- [x] R4 CLOSED en el gate local **583/583**
- [x] Core 0.1 CLOSED en el gate local **603/603**

## 10. Repair oracle (delta <person> 2026-09-19)

El compilador es un **oráculo estructurado** para corrección autónoma acotada — no un string de error pegado al LLM.

Normativo: [`REPAIR-ORACLE.md`](REPAIR-ORACLE.md) · ADR-231.

- Bucle: objetivo verificable → cambio mínimo → checks ARITA/Rust → clasificar → repair/rollback/blocked  
- Pirámide 0–9 (parser → … → runtime); éxito barato **no** tapa fallo caro  
- Diagnósticos JSON (`ARI-*`, spans, `allowed_repairs` / `forbidden_repairs`)  
- CLI futura: `arita repair` — **HOLD IMPL** hasta reinicio + GO  
- **No fake success:** sin reparación autónoma válida ⇒ estado honesto (`blocked_by_ambiguity`, etc.)

## 11. Core 0.2 (perfil `service`)

Pins: [ADR-233](ADR/233-core-0.2-pins.md). Prereq Core 0.1 CLOSED. Vertical = HTTP daemon + async explícito.

- [x] Pins Core 0.2 → ADR-233
- [ ] Core 0.2 CLOSED en el gate local
