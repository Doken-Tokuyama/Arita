# ARITA Repair Oracle — compiler-guided autonomous correction

- **Estado:** **v0.1.2 punto 6: diagnósticos estructurados, sugerencias deterministas y `arita fix`** (rev. 1.2 — memoria + presupuestos + envelope + estancamiento)
- **Producto:** el compilador es un **oráculo estructurado**, no un mensaje de error pegado al LLM
- **Relacionados:** RFC rev. 3 · PRODUCT-VISION · evidencia/scenarios · ADR-231
- **HOLD levantado** (GO <person> 2026-10-09 18:26, vía Ingeniero); v0.1.2 punto 6: diagnósticos estructurados, sugerencias deterministas y `arita fix` (antes: **HOLD IMPL** `arita repair` / CUT hasta GO reinicio <person>)

## 0. Tesis

El agente propone un **cambio mínimo**; el compilador y validadores producen **hechos**; un **Repair Controller** decide:

`pass` | `repair` | `rollback` | `blocked` | `needs_human`

“Compila” ≠ “correcto”. Un loop LLM↔compilador ayuda (p.ej. RustAssistant ~74% en errores de compilación OSS), pero **no** cierra la tarea sin scenarios/evidencia sobre el artefacto final.

\[
\text{No hay reparación autónoma} \Rightarrow \text{no hay fake success}
\]



## 0.1 Automejora (qué sí / qué no)

Hay dos “automejoras”; ARITA empieza por la **primera**:

1. **Memoria externa de reparación** — auditable, reversible; no modifica pesos del modelo.
2. **Finetuning del LLM** — caro; sólo con miles de pares golden. Antes amplifica ruido.

El sistema mejora sobre todo el **compilador, las reglas y el Repair Controller**, no el modelo opaco.

## 0.2 Memoria de fallos (registro inmutable)

Cada fallo/reparación se guarda estructurado (no “prompt→respuesta” libre):

\[
\text{diagnóstico} + \text{contexto semántico} +
\text{estrategia} + \text{patch} + \text{resultado medido}
\]

Campos mínimos: `repair_id`, versiones lenguaje/compilador/toolchain, `policy_profile`, `task_kind`, diagnóstico (`stage`/`code`/`category`/`semantic_rule`), `context_fingerprint` (HIR shape, types, effects, binding versions), `hypothesis`, `repair_template`, `patch_shape`, `validation_plan`, `outcome`, `quality` (`minimal_diff`, policy/dep delta, acceptance_preserved).

### Recuperación de estrategias

1. Normalizar a código estable (`ARI-OWN-002`…)  
2. Extraer contexto (HIR/AIR, tipos, efectos, perfil, bindings)  
3. Recuperar repairs previos compatibles  
4. Rankear por éxito, minimidad, regresiones, coste  
5. Dar al LLM **sólo 1–3** estrategias permitidas  
6. Sandbox + validar  
7. Registrar resultado **también si falla**

Ranking inicial transparente:

\[
\mathrm{score}(s) =
0.40 \cdot \mathrm{success\_rate}
+ 0.20 \cdot \mathrm{context\_similarity}
+ 0.20 \cdot \mathrm{minimality}
+ 0.10 \cdot \mathrm{test\_preservation}
- 0.10 \cdot \mathrm{regression\_rate}
\]

Claves simbólicas primero (no embeddings opacos que autoricen repairs):

```text
diagnostic_code + semantic_rule + HIR node kind
+ language_version + types + profile + permitted repair class
```

### Biblioteca de plantillas (no memoria libre)

| Código | Regla | Plantillas válidas |
|--------|-------|--------------------|
| `ARI-RES-014` | API inexistente | binding real / símbolo / pedir dep |
| `ARI-TYP-021` | tipo dominio | parse/constructor / adapter / firma |
| `ARI-ERR-005` | `Result` ignorado | `?` / `match` / recover tipado |
| `ARI-TOT-003` | operación parcial | `get()?` / índice refinado |
| `ARI-OWN-002` | borrow conflict | acortar scope / owned local / reordenar / tx |
| `ARI-ASY-011` | guard × await | cerrar scope / actor / canal |
| `ARI-EFF-004` | capability ausente | API sin efecto / pedir permiso / rediseñar |
| `ARI-POL-001` | unsafe/panic escape | **rechazar**; API segura o bloquear |

### Promoción de aprendizaje

| Nivel | Requisitos | Uso |
|-------|------------|-----|
| `observed` | pasó `cargo check` | referencia débil |
| `tested` | tests relevantes | baja prioridad |
| `trusted` | tests + properties + policy + diff mínimo | sugerible auto |
| `golden` | revisión humana + regresión + repetición | plantilla preferida |
| `rejected` | compiló pero rompió tests/policy/semántica | **nunca** sugerir |

Cada `golden` → test de regresión ARITA + regla/diagnóstico más preciso + doc + repair determinista si aplica.

## 1. Anti-bucle (prohibido)

```text
IA genera → cargo check falla → IA prueba cambios aleatorios hasta que “desaparezca”
```

## 2. Bucle correcto

```text
Objetivo verificable
   ↓
Plan y cambio mínimo
   ↓
Parse / typecheck ARITA
   ↓
Emisión Rust determinista
   ↓
Compilador, lints, tests y verificadores
   ↓
Clasificación del fallo (causa raíz)
   ↓
Reparación acotada o rollback
   ↓
Nueva evidencia sobre revisión inmutable
```

Cada intento **debe** asociar:

| Campo | Rol |
|-------|-----|
| Tarea concreta | qué se pretendía |
| Hash base | `sha256(source + lock + policy + emitted-rust)` |
| Diff pequeño reversible | 1 causa, no reescritura masiva |
| Diagnósticos estructurados | ARI-* JSON |
| Hipótesis de causa | antes del diff |
| Comprobaciones objetivo | plan de verify |
| Presupuesto de intentos | p.ej. max 3 / misma causa |
| Decisión final | pass / repair / rollback / blocked / needs_human |

## 3. Pirámide de corrección (barato → caro)

Reparar primero con el feedback más determinista; subir de nivel **sólo** si el anterior pasa.

| Nivel | Herramienta | Qué descubre | Reacción |
|------|-------------|--------------|----------|
| 0 | Parser ARITA | gramática / AST | sólo la construcción sintáctica |
| 1 | Resolver | imports, símbolos, APIs inexistentes | catálogo real; **no** inventar nombre |
| 2 | Type checker | tipos, `Result`, match incompleto | contrato / conversión / manejo tipado |
| 3 | Ownership / efectos | move, borrow, capability | reestructurar scope; no clone/permiso sin policy |
| 4 | Emit + `cargo check` | lowering, traits, async, borrow real | corregir **IR/emisor**; no parchear Rust a mano |
| 5 | Clippy / policy | patrones peligrosos | repair permitida o rechazar diseño |
| 6 | Unit / integración / acceptance | regresión observable | corregir producción; **no** degradar expected |
| 7 | Properties / fuzz | bordes / input hostil | invariantes, validación, límites |
| 8 | Kani / Verus / Creusot | propiedades acotadas/deductivas | modelo/código; **no** `assume` escape |
| 9 | Runtime / canary | operacional | traza, reproduce, regresión |

**Regla:** éxito de nivel 4 **no** tapa fallo de nivel 6.

### Prioridad de cola

`P0` soundness/unsafe/policy → `P1` parse/resolve/types/ownership → `P2` build emit → `P3` acceptance → `P4` fuzz/formal → `P5` lints.

## 4. Diagnóstico estructurado (ejemplo)

No enviar sólo el texto de `rustc`. Normalizar a ARITA:

```json
{
  "revision": "sha256:base",
  "stage": "ownership",
  "diagnostic": {
    "code": "ARI-OWN-002",
    "category": "borrow_conflict",
    "severity": "error",
    "primary_span": {
      "file": "routes.arita",
      "start_line": 47,
      "start_column": 5,
      "end_line": 47,
      "end_column": 31
    },
    "message": "No puedes mutar `routes` mientras existe un préstamo de lectura activo.",
    "cause": {
      "kind": "shared_borrow",
      "origin_span": {
        "file": "routes.arita",
        "start_line": 39,
        "start_column": 18
      }
    },
    "semantic_rule": "exclusive_mutation",
    "allowed_repairs": [
      "reduce_borrow_scope",
      "copy_required_fields_before_mutation",
      "reorder_read_then_write",
      "use_transactional_update"
    ],
    "forbidden_repairs": [
      "unsafe_escape",
      "suppress_diagnostic",
      "implicit_unbounded_clone",
      "change_test_expectation"
    ]
  }
}
```

El agente recibe: **regla violada · transformaciones permitidas · prohibidas · sitio · evidencia** — no “haz que compile”.



## 4.1 Envelope de diagnósticos (canal agente)

Dos canales: (1) humano corto con snippets; (2) **JSON/CBOR tipado** — el agente **no** parsea ANSI de terminal.

### Envelope

`schema_version: arita.diagnostics.v1` · `revision` (source/policy/lock/compiler hashes) · `stage` · `status` · `summary` (errors, warnings, blocked_by_policy, cascade_errors_suppressed) · `diagnostics[]` · `repair_context` · `next_action` (`repair_allowed` | `needs_dependency_approval` | `needs_human` | …).

### Diagnóstico individual (campos obligatorios)

Además de §4: `id`, `blocking`, spans byte+línea, `related[]`, `semantic` (rule, hir_node, types, control_flow), `cause_class`, `allowed_repair_classes` / `forbidden_repair_classes`, `examples[]` (forma canónica), `validation_required[]`.

Cascadas: `cascade_errors_suppressed` evita gastar intentos en síntomas.

### Evidencia con claims autorizados

`schema_version: arita.evidence.v1` · checks con status/duration/bounds ·  
`claims_permitted` (p.ej. `compiled`, `tested`, `bounded_verified_parser`) ·  
`claims_forbidden` (p.ej. `fully_verified`) — el agente **sólo** puede afirmar claims permitidos.

## 5. Taxonomía de reparaciones (conjunto cerrado)

| Clase | Causa habitual | Repair segura | Anti-patrón |
|-------|----------------|---------------|-------------|
| Símbolo no resuelto | API inventada / typo / dep ausente | catálogo/bindings; símbolo real o pedir dep | inventar nombre plausible |
| Tipo incompatible | dominio / firma | conversión validada; adaptar API | `as`, stringify/parse ciego |
| `Result` no tratado | fallo ignorado | `?` / `match` / recover tipado | `unwrap` / `expect` / default silencioso |
| `Option` ausente | valor no garantizado | `match` / `ok_or` / fallback justificado | asumir `Some` |
| Match no exhaustivo | variant olvidado | ramas explícitas + neg tests | `_ => default()` que oculta |
| Overflow / división | precondición ausente | checked + error/límite | wrap silencioso |
| Borrow / move | aliasing / vida | reducir scope; rediseñar API | clonar arbitrariamente |
| Trait / bound | interop mal elegido | binding compatible / adapter declarado | bound genérico “por suerte” |
| Async / `Send` | guard sobre await | cerrar guard; actor/canal | `spawn_local` como parche general |
| Test fallido | lógica o requisito ambiguo | corregir producción + regresión | cambiar expected sin justificar |
| Fuzz crash | input no previsto | validar / acotar / invariante | filtrar corpus a escondidas |
| Kani counterexample | propiedad falsa / borde | reproducir como test; arreglar | bajar `unwind` / `assume` arbitrario |
| Dep / policy | fuera de allowlist | solicitud explícita + revisión | editar manifest en silencio |

## 6. Fases del controlador

### 6.1 Reproducir antes de editar

```text
revision = sha256(source + arita.lock + policy + emitted-rust)
diagnostic_set = verify(revision)
```

Si no reproduce → entorno/caché/flake/desalineación; **no** editar.

### 6.2 Localizar causa raíz

No agrupar 100 errores. Cascadas típicas: parse bloquea resolve; símbolos bloquean tipos; tipos producen borrow falsos; emitter produce diagnósticos Rust que **no** existen en ARITA; test puede fallar por especificación, no por código.

### 6.3 Hipótesis antes del diff

```json
{
  "diagnostic": "ARI-OWN-002",
  "hypothesis": "El borrow compartido de `routes` sobrevive más de lo necesario; sólo hace falta `route_id`.",
  "proposed_change": "Extraer `route_id` owned y terminar el borrow antes de `routes.insert`.",
  "expected_effect": "Eliminar conflicto mutable/compartido sin clonar la colección.",
  "affected_invariants": [
    "route_id identifica la ruta",
    "insert conserva unicidad de ID"
  ],
  "verification_plan": [
    "arita check routes.arita",
    "cargo check",
    "cargo test -p routes",
    "property test duplicate_ids"
  ]
}
```

El LLM propone; el controlador **acepta o rechaza** según repairs permitidas.

### 6.4 Un solo diff semántico

Límites:

- 1–3 archivos de producción / intento
- Un módulo semántico
- **No** cambiar a la vez producción y criterios de aceptación bloqueados
- **No** policy / dependencia / capability sin aprobación
- **No** borrar tests / reducir cobertura para “arreglar”
- Rechazar escapes: `unsafe`, `unwrap`, `expect`, `todo!`, `unimplemented!`, `#[allow]`, `cfg` que oculte rutas

### 6.5 Revalidar desde abajo

```text
ARITA parse → resolve → type → ownership/effects
  → emit Rust → cargo check → target tests
  (+ property / Kani si aplica)
```

Resultados enlazados al **hash post-parche**.

### 6.6 Parar y escalar → humano

Detener si: >N intentos misma causa (p.ej. 3) · cambia la especificación · amplía permisos/deps · dos repairs semánticamente distintas sin evidencia · ambigüedad de negocio · degrada límites/arquitectura · toca FFI/crypto/auth/migración/despliegue · repair prohibida.

Estado honesto: `blocked_by_ambiguity` — **nunca** “implementado”.

## 7. Feedback estructural ARITA (ejemplos)

### API alucinada (`ARI-RES-014`)

`http.fetch_json` inexistente → listar binding real + candidatos tipados (`http.get`, `json.decode`) + repair sugerida. **Prohibido** adivinar otro método.

### Acceso parcial (`ARI-TOT-003`)

`users[index]` → exigir `users.get(index)?` o índice refinado. Enseña la forma canónica (alineado ADR-227).

### Tarea sin dueño (`ARI-TASK-007`)

`spawn` suelto en `service-safe` → `await` / `group.spawn` / `supervisor.spawn`; prohibido detach.

## 8. Determinista antes de LLM

| Patrón | Auto-repair posible | Condición |
|--------|---------------------|-----------|
| Import no usado | eliminar | no reexport público |
| Formato | `arita fmt` | siempre |
| Match con variant nuevo | stub que **bloquea** compilación | nunca marcar resuelto |
| Literal → tipo inequívoco | constructor tipado | sin overflow |
| `?` omitido obvio | sugerir/apply | firma ya admite el error |
| Borrow temporal innecesario | acortar scope | AIR prueba equivalencia local |
| Clippy estilo | `--fix` en sandbox | diff revisable; sin `allow` nuevo |

LLM sólo para decisión **semántica**.

## 9. Compilador como profesor

Base de conocimiento por código:

```text
diagnostic code
  → regla semántica violada
  → formas válidas de programa
  → ejemplos mínimos
  → restricciones de policy
  → tests/gates obligatorios
```

Ejemplo `ARI-ERR-005` (`Result` no consumido): formas válidas `?` / `match` / `recover`; no válidas `unwrap` / `default` / `let _ =`; exige caso de error en tests.

## 10. Controles post-parche

| Eje | Preguntas |
|-----|-----------|
| Tarea | ¿acceptance intactos? ¿expected tocados? ¿bypass? ¿API pública cambiada? |
| Políticas | ¿unsafe/panic/unwrap/dep/capability/timeout/tamaño? ¿`allow`/`cfg` oculto? |
| Evidencia | ¿sobre hash final? ¿tests ejecutados de verdad? ¿Kani bounds iguales o mayores? ¿nuevos supuestos listados? |

## 11. Arquitectura del Repair Controller

```text
        Tarea + especificación
                  │
                  ▼
           Planner LLM
                  │
                  ▼
           Patch candidate
                  │
   ┌──────────────┴──────────────┐
   │ ARITA Repair Controller      │
   │ - revision store             │
   │ - policy / permission        │
   │ - diagnostic normalizer      │
   │ - root-cause grouper         │
   │ - repair-rule selector       │
   │ - diff budgeter              │
   │ - validation planner         │
   │ - rollback engine            │
   │ - evidence recorder          │
   └──────────────┬──────────────┘
        │         │         │
        ▼         ▼         ▼
  ARITA check  Cargo/Clippy  Tests/Kani/fuzz
        │         │         │
        └──── evidence + diagnostics ────┘
                       │
                       ▼
         pass / repair / rollback / blocked
```



## 11.1 Arquitectura ampliada (rev. 1.2)

```text
Tarea / SPEC / aceptación
        ↓
Planificador LLM → patch candidato
        ↓
┌─────────────────────────────────────────────┐
│ Repair Controller                            │
│ Revision store · Policy gate · Diff budget   │
│ Diagnostic normalizer → Root-cause grouper   │
│ → Strategy retrieval ← Repair-memory store   │
└─────────────────────────────────────────────┘
   ↓              ↓              ↓
ARITA check   Rust/Cargo     Tests/Fuzz/Kani
   └──────── Evidence aggregator ────────┘
                    ↓
     PASS / RETRY / ROLLBACK / BLOCKED
```

Componentes **no** deben: auto-certificarse (LLM), editar fuera de scope, sobrescribir último buen estado, aceptar warnings como éxito, pasar logs sin estructura, autorizar escapes, alterar policy, declarar done sin pruebas, convertir `tested`→`proved`, reiniciar el loop en silencio.

### Máquina de estados

```text
IDLE → PLAN → PATCH_PROPOSED → POLICY_CHECK
  ├─ reject → BLOCKED
  └─ allow → SANDBOX_VALIDATE
       ├─ pass → ACCEPTED
       ├─ fail_repairable → DIAGNOSE → RETRIEVE_STRATEGY → PATCH_PROPOSED
       ├─ fail_nonrepairable → BLOCKED
       └─ environment_failure → RETRY_ENVIRONMENT
```

`ACCEPTED` exige **todos** los gates del perfil. `cargo check` verde ≠ `ACCEPTED` si falta acceptance.

### Pseudocódigo (núcleo)

```text
while may_continue:
  evidence = validate(candidate, required_checks)
  if all_required_pass → accept
  diagnosis = normalize_and_group(failures)
  if needs_human or is_stuck or risk_over_budget → block
  strategies = retrieve_allowed(diagnosis, context, policy, memory)
  proposal = llm_minimal_patch(..., strategies, diff_budget)
  if !policy_allows → record_rejected; continue
  candidate = apply_in_sandbox(proposal)
→ block_on_budget
```

Lo crítico: `may_continue`, `policy_allows`, `all_required_checks_pass`, `is_stuck` — no el `while`.

## 12. CLI objetivo: `arita repair`

```bash
arita repair \
  --task TASK-142 \
  --profile service-safe \
  --max-attempts 3 \
  --scope crates/proxy-routes \
  --require acceptance,types,ownership,cargo,test \
  --deny unsafe,panic,dependency-change,policy-change
```

### Salida pass (ejemplo)

```text
Attempt 3: Status: pass
Evidence: ARITA check PASS · emit 0 unsafe · cargo check PASS ·
  clippy 0 warnings · acceptance 7/7 · properties 10k PASS
Residual: URL parser fuzzed 5m; no formal proof of normalization.
```

### Salida blocked (ejemplo)

```text
Status: BLOCKED
Reason: el requisito no especifica si http:// es válido o debe exigirse HTTPS.
No se aplicó ningún cambio irreversible.
```

## 13. Regla de oro

\[
\text{Cambio aceptado} =
\text{causa explicada}
\land \text{reparación permitida}
\land \text{requisitos preservados}
\land \text{evidencia nueva sobre el artefacto final}
\]



## 15. Presupuestos, progreso y estancamiento

No basta `max_iterations=10`. Presupuesto **multidimensional**:

| Presupuesto | Ejemplo inicial |
|-------------|-----------------|
| Iteraciones | 3 / causa raíz; 8 / tarea |
| Diff | ≤200 líneas; ≤3 archivos prod / intento |
| Tokens / coste modelo | tope fijo / tarea |
| Wall clock | 20 min local; 60 min CI |
| CPU/mem verificadores | 4c/8GiB; 10 min / checker |
| Ejecución tests | 10k properties; fuzz 5 min PR / 1 h nightly |
| Riesgo | 0 sin aprobación: red, secretos, migrations, FFI |
| Regresión | no aumentar errores netos |
| Estancamiento | ≤2 parches sin reducir causa raíz |

Al tocar techo: entregar estado + razón; **nunca** auto-subir límite ni reiniciar en silencio.

### Vector de progreso

\[
P = (E_{\mathrm{root}}, E_{\mathrm{policy}}, E_{\mathrm{compile}}, E_{\mathrm{acceptance}}, E_{\mathrm{regression}}, D_{\mathrm{diff}})
\]

Progreso lexicográfico: baja root → no sube policy → no baja aceptación → no introduce regresiones → diff en presupuesto.

Ciclo `A→B→A` o mismo `root_cause_fingerprint` → `BLOCKED_STAGNATION` (hash: revision + root_cause + validation_summary + policy_delta).

### Estados de salida

| Estado | Significado |
|--------|-------------|
| `accepted` | todos los gates del perfil sobre revisión final |
| `partial` | compila pero falta evidencia; **no** “terminado” |
| `blocked_ambiguity` | requisito/semántica insuficiente |
| `blocked_policy` | única repair viola seguridad |
| `blocked_budget` | tiempo/intentos/coste agotados |
| `blocked_stagnation` | ciclo / sin progreso |
| `blocked_environment` | toolchain/red/creds |
| `needs_human_review` | deps, permisos, FFI, migración, API pública |
| `rollback` | regresión; volver a último sano |

Handoff humano: pregunta concreta · intentos · último sano · diagnóstico · opciones A/B/C · sin tocar acceptance/policy.

## 16. Orden de implementación (repair)

1. Diagnósticos JSON estables (códigos, causa, spans, repairs, checks)  
2. Snapshots + sandbox + rollback  
3. Controller con presupuesto + estancamiento  
4. Biblioteca manual 20–30 templates  
5. Memoria con ranking por evidencia  
6. LLM sólo para decisión semántica no mecánica  
7. Promoción golden → reglas/tests/deterministic  
8. Finetuning **sólo** con miles de pares golden  

\[
\text{Autonomía útil} =
\text{modelo} + \text{compilador estructurado} +
\text{memoria validada} + \text{gates}
- \text{escapes} - \text{reintentos sin límite}
\]

## 14. Checklist DOC

- [x] Oráculo estructurado (no paste de rustc)
- [x] Pirámide 0–9 + P0–P5
- [x] JSON diagnóstico + hipótesis
- [x] Taxonomía + anti-patrones
- [x] Diff budget + escalate
- [x] Ejemplos ARI-RES / TOT / TASK
- [x] Determinista antes de LLM
- [x] Controller + CLI `arita repair`
- [x] Memoria de repairs + ranking + templates + promoción
- [x] Envelope diagnostics.v1 + evidence claims
- [x] Presupuestos / progreso / estancamiento / exit states
- [ ] IMPL post-GO reinicio (ADR-231)
