# ADR-293 — Core UNDECLARED-CALL v0: llamada a función no declarada (B-286-7, P1)

- **Estado:** **DRAFT PINS v0.1 (Arquitecto, 2026-10-04; decisiones del Ingeniero incorporadas; sin GO IMPL).** Pendiente la PREP de Measure (§7). No declara PASS, CLOSED ni N/N medido.
- **CUT-ID:** `CORE-0.10-UNDECLARED-CALL-20261003`
- **Fecha:** 2026-10-04
- **Autores:** ARITA Arquitecto (pins) · decisiones del Ingeniero 2026-10-04 · hechos de Parser en `DOC/reviews/PREP_B286_7_20261003.md` (en adelante **PREP**) y `DOC/reviews/ADR-293-PARSER-SWEEP-20261004.md` (en adelante **BARRIDO**)
- **Padre / contexto:** [ADR-286](286-core-0.10-errores-fase1.md) (B-286-7, B-286-11) · [ADR-291](291-core-mutex-reject-v0.md) (neg `mutex_new`; DRAFT, ver D7) · [ADR-292](292-core-int-arith-runtime-v0.md) (CLOSED, N = 866 según `DOC/GATE-CORE10-INT-ARITH-RUNTIME-20261003.md`) · PREP B-286-7 · BARRIDO del Parser
- **Cierra (al CLOSED):** **B-286-7** (P1) y la migración del neg `neg-core03-compose-mutex-hold` desde el fallo accidental E0100 a E0347.
- **Código:** **E0347** (libre en `crates/`, ADR y `ejemplos/`, §2) — mensaje EN canónico: ``E0347: call to undeclared function `NAME` `` (`NAME` = nombre de la llamada). **Sin span.**
- **No reabre:** B-286-11 (spans en `Call`/`HirCall`; sigue abierto, P2) · E0206/E0313/E0320/E0321 y su orden en el brazo `Call` · E0241/E0203 de `spawn`/`join` · E0007 · E0340–E0345 · ADR-291 (su texto) · ADR-292 · ADR-289 · Codegen.

## 1. Decisión y límites

**D1 — Código y mensaje.** Una llamada libre a una función que no está declarada en el módulo, ni importada por `use`, ni es builtin ⇒ **E0347** con el texto exacto ``E0347: call to undeclared function `NAME` ``. Sin span: `Call`/`HirCall` no llevan span y B-286-11 sigue abierto (`DOC/ADR/286-core-0.10-errores-fase1.md` L262); el diagnóstico no lo reabre.

**D2 — Punto de comprobación.** En HIR, en el brazo `HirExpr::Call` de `eval_expr` (`crates/arita-hir/src/lib.rs` L743–L884), dentro del `match c.callee` de ADR-244, **en el hueco `_ => {}` (L865) que queda tras** E0320 (`busy_spin`/`hang_forever`, L772), E0313 (`timeout`/`delay`/`cancel*`, L778–L846), E0206 (`http_*`, L853–L858) y E0321 (`reqwest`, L860–L864), y **ANTES de evaluar los argumentos** (`for arg in &c.args`, L875–L877) y del chequeo `async_fns` (L867). Heredan así su prioridad los brazos anteriores y el retorno temprano de `spawn`/`join` (L744–L765).

**D3 — Regla.** Se rechaza si el callee **no** cumple ninguna exención: (1) contiene `::` (`Vec::new`, `List::new`, `Map::new`, `host::*`); (2) es `print`; (3) es `DEFERRED_SHAPE_MARKER` (`crates/arita-syntax/src/lib.rs` L1745); (4) está en `BUILTIN_FN_NAMES` (HIR L5146); (5) está en `fn_rets`, es decir declarada en el módulo (L5217); (6) está importada por `use` (D4).

**D4 — `imports` es OBLIGATORIO.** Campo nuevo `imports: Vec<String>` en `HirModule` (L61), poblado en `lower_ast` (L5323) desde `module.uses[].item`; el `CheckCtx` lo recibe como `HashSet` y `check` (L5177) lo asigna en los tres sitios donde construye un `CheckCtx` (dos bucles sobre `hir.functions` y el de `tests`). El BARRIDO demuestra su necesidad: **sin** la exención de imports cambian **16** programas; **con** ella cambia **exactamente 1** (`ejemplos/core03/client-compose/neg/02-mutex-hold.arita`, ok → E0347), **0 roturas**, sobre **867** `.arita`. Tocar `HirModule` obliga a editar **13** literales de test (§3).

**D5 — Codegen no cambia.** La llamada se rechaza en HIR antes de emitir: ni un byte de Rust emitido cambia, y el neg migrado deja de llegar a rustc/cargo.

**D6 — Migración única.** `neg-core03-compose-mutex-hold` (id y fixture congelados, sha256 `4ea5e12123c7ae22e1b4af62827f9e1a45a8def29d0a84aa44c373b7da222d79`) pasa de E0100 accidental a **E0347** (llamada `mutex_new`), con freeze nuevo y addendum que escribe el Ingeniero (§5).

**D7 — Interacción con ADR-291.** `mutex_new(0)` es una llamada libre: no contiene `Mutex`, `Arc` ni `.lock()`, así que **no** pertenece al alcance de E0346 de ADR-291 (cuyo propio §2.2 lo reconoce: «`mutex_new` no es una forma nueva del alcance cerrado D2»). Con ADR-293 antes que ADR-291 (orden de la cola de Lex, GATE de ADR-292), el neg migra a **E0347 y NO a E0346**; ADR-291 (v0.1, pendiente) pierde su «Migración única» y conserva sus oráculos propios (tipo, `Arc`, constructor, `lock`) sobre fuentes nuevas. La corrección de ADR-291 (§5, §2.4, k y N) la hace su autor; este ADR no la edita. Los números de N de ADR-291 los recalcula el Ingeniero: aquí no se fijan.

**D8 — Fuera de alcance.** (a) B-286-11 (spans en `Call`); (b) funciones de módulos importados (no se resuelve su existencia ni su firma: eso lo hace el CLI con E0404/E0405/E0406/E0331); (c) variables no ligadas (`Path` no ligado ⇒ `Int`, PREP L60 y L224); (d) cualquier cambio de Codegen.

## 2. Evidencia (PREP + BARRIDO + código)

- **Recorrido actual de `mutex_new(0)`** (PREP L23–L44): el parser acepta `user_call`; HIR no tiene brazo para `mutex_new` (cae en `_ => {}`) y `type_of_expr` lo tipa `Int` por el fallback de `fn_rets` (hoy L1305–L1311); Codegen emite la llamada; cargo falla tarde y la CLI lo envuelve como E0100 (`async/tokio`). El oráculo `run_core03_compose_mutex_hold_oracle` comprueba solo la subcadena `E0100` (`crates/arita-cli/src/measure.rs` L18368–L18411; registro en L18542). **(corregido)**: la fila B-286-7 de ADR-286 (L258) y ADR-291 citan HIR L1275–L1282 / L1276–L1281, y el PREP cita measure L17704–L17745; las posiciones actuales son las de este párrafo.
- **Código libre** (PREP L62): `rg` sobre `crates/`, `DOC/ADR`, `ejemplos/` no da definiciones de E0347; solo aparece en notas de `DOC/reviews/` (PREP B-286-7, PREP B292_2, PREP S2 must-use, y una **sugerencia opcional** no adoptada de guard escapado en `PREP_MUTEX_PARSER_20261003.md` L229). E0346 es propuesta de ADR-291; E0333–E0339 los reserva ADR-290.
- **Corpus (PREP L12–L22):** 860 `.arita` en el PREP; 150 nombres de llamada libre únicos, de los que 12 sin declarar; 68 de las 73 llamadas sin declarar son de `ejemplos/f3/` (isla lógica Datalog, no pasan por parser/HIR); las 5 restantes son `reqwest_get` ×4 (E0321, sin cambio) y `mutex_new` ×1.
- **BARRIDO (ejecución real de parse + `lower_ast` + HIR check, en copia `/tmp/arita_adr293`):** 867 `.arita` (los 860 del PREP más 7 de `core10/int-overflow`). Base: 530 `ok` / 337 con diagnóstico; con parche: 529 / 338. **Cambia 1 fichero** (`02-mutex-hold`: `ok` → ``E0347: call to undeclared function `mutex_new` ``); **0 roturas** (los 516 `ok` fuera de `neg` siguen `ok`); los 4 negs `reqwest` siguen en E0321; los 12 de `f3/` no cambian. **Ablación** (sin la exención de imports): 16 cambios en vez de 1, los 15 extra son ficheros que llaman a fns importadas por `use`.
- **Confirmaciones del BARRIDO:** `assert f(1) == 2` en `test` con `f` no declarada sí pasa por `eval_expr` y da E0347 (con `f` declarada, `ok`); `g(h(1))` da E0347 sobre `g` (callee antes de evaluar argumentos); `cargo test -p arita-hir -p arita-syntax` en la copia: 174 + 78 passed, 0 failed, original y parcheado; instrumentado, 0 impactos del guard en los 174 tests de HIR.
- **Parche de referencia:** `/tmp/arita_adr293/adr293_hir_final.diff` (192 líneas, sha256 `15a9ef142973ec58941500b02cc7256bdd9d0fe9eb6341ef88cc871b5991ac63`; idéntico a `adr293_hir.diff`). Es evidencia de viabilidad en una copia, **no** la implementación: el árbol real no se ha tocado.
- **Referencias de código verificadas por lectura** (`crates/arita-hir/src/lib.rs`, sha256 `d0571e12419dae491ec7ce5dcfba99ad247c045793fc3c1f6c53abf1ee575807`, 9630 líneas): `HirModule` L61; `CheckCtx.fn_rets` L417 y `CheckCtx::new` L434; brazo `Call` L743–L884; E0320 L772, E0313 L778–L846, E0206 `http_*` L853–L858, E0321 `reqwest` L860–L864, `_ => {}` L865, `async_fns` L867, evaluación de argumentos L875–L877; `type_of_expr` genérico L1305–L1311; `BUILTIN_FN_NAMES` L5146; `check` L5177; `fn_rets` L5217; `lower_ast` L5323. Literales de test de `HirModule`: 20 líneas con `HirModule {` = 1 struct + 1 firma y 1 literal de `lower_ast` + 4 firmas de helpers + **13 literales de test** (L5641, L5704, L7413, L7587, L7737, L7783, L7812, L7835, L7902, L7950, L7988, L8035, L8088); coincide con el BARRIDO.

## 3. Cambio mínimo previsto (a verificar en IMPL)

- **`arita-hir`:** `HirModule.imports`; `CheckCtx.imports`; el brazo de exención y error de D2/D3; relleno en `lower_ast`; asignación en los 3 sitios de `check`; **13 literales de test** con `imports: vec![]` (el PREP decía 20: contaba también firmas y estructura; **(corregido)** con el BARRIDO y la lectura de §2).
- **Tests cargo adicionales** (no cuentan en N; PREP L216 y confirmaciones del BARRIDO): E0347 con fn importada exenta (`imports` poblado); `print`/`Vec::new`/`host::*` exentos; precedencia frente a E0007, `spawn(ghost())` (E0241) y `reqwest_*` (E0321); `assert f(1) == 2` en `test` (E0347).
- **Measure:** migrar el oráculo `neg-core03-compose-mutex-hold` (hoy exige la subcadena `E0100`) y registrar los 5 oráculos de §4; lo hace Measure/Ingeniero, no este ADR.
- **Sin cambios:** `arita-syntax`, Codegen, `arita-cli` (salvo el oráculo de `measure.rs`), `package.rs`.

## 4. Oráculos propuestos (k = 5)

Ids y fuentes son los del PREP §4.4 (fuentes de ~6 líneas, `[L]`: no ejecutadas); los ids de oráculo y las rutas de fixture quedan **fijados por el Sello del Ingeniero** (Q1, abajo). Cada neg: stderr con el texto exacto ``E0347: call to undeclared function `ghost` ``, exit 1, stdout vacío y sin emisión de Rust.

1. **`neg-core10-undeclared-call-stmt`** (`ejemplos/core10/undeclared-call/neg/01-stmt.arita`): `main` con `ghost(1)` como sentencia seguida de `print("after")`.
2. **`neg-core10-undeclared-call-let`** (`ejemplos/core10/undeclared-call/neg/02-let.arita`): `let n: Int = ghost(1)` y `print(n)`.
3. **`neg-core10-undeclared-call-print-arg`** (`ejemplos/core10/undeclared-call/neg/03-print-arg.arita`): `print(ghost(1))`.
4. **`neg-core10-undeclared-call-in-result-fn`** (`ejemplos/core10/undeclared-call/neg/04-in-result-fn.arita`): la llamada `let r: Int = ghost(n)` dentro de `fn f(n: Int) -> Result<Int, Int>`, no en `main`.
5. **`core10-declared-call-ok`** (`ejemplos/core10/undeclared-call/05-declared-ok.arita`, fuera de `neg/`; control positivo, sin prefijo `pos-` según el Sello; **(corregido)** el PREP lo llamaba `pos-core10-declared-call-control`, misma forma que (2) con `fn ghost(n: Int) -> Int { n }` declarada): construye verde; stdout `1` [derivado de la fuente, no ejecutado].

**k = 5. N:** numerado contra N = 866 (ADR-292 CLOSED) ⇒ **871 (866 + 5)**, que es el valor que recoge el GATE de ADR-292 para esta cola; **lo recalcula el Ingeniero al GO**. La **migración** de `neg-core03-compose-mutex-hold` conserva su id y no suma a N. Reglas: skip ≠ PASS; ningún oráculo cuenta si el build no se ejecutó. Los negs existentes de `reqwest` (E0321), `http_*` (E0206) y `busy_spin` (E0320) sirven de guardas de precedencia sin cambios.

## 5. Migración y backlog

**Migración de `neg-core03-compose-mutex-hold`.** Fixture `ejemplos/core03/client-compose/neg/02-mutex-hold.arita` (sha256 `4ea5e12123c7ae22e1b4af62827f9e1a45a8def29d0a84aa44c373b7da222d79`, verificado hoy): E0100 ⇒ E0347. Requiere **freeze nuevo + addendum escritos por el Ingeniero**; **ningún freeze existente se reescribe**. ADR-291 (§2.4) enumera cuatro freezes que contienen esa sha: `MEASURE-ADR284-FREEZE-20260927.sha:175`, `MEASURE-ADR285-FREEZE-20260927.sha:175`, `MEASURE-ADR286-FREEZE-20260927.sha:177` y `MEASURE-ADR286-S1B-FREEZE-20261002.sha:189` (verificados). **(corregido)** `rg` de esa sha en `DOC/reviews/*.sha` da además `INGENIERO-ADR283-FREEZE-20260927.sha:160`, `MEASURE-ADR287-PKG-MEMBER-FREEZE-20261002.sha:202`, `MEASURE-ADR288-EMIT-CLIPPY-FREEZE-20261003.sha:205`, `MEASURE-ADR289-EMIT-UNUSED-PARENS-FREEZE-20261003.sha:210`, `MEASURE-ADR290-INDEX-MUT-FREEZE-20261003.sha:217` y `MEASURE-ADR292-INT-ARITH-RUNTIME-FREEZE-20261003.sha:223`: **diez** en total; ninguno se reescribe. El oráculo de `measure.rs` se cambia a E0347 solo con evidencia real.

**Backlog.**
- **B-286-7 (P1):** se cierra al CLOSED de este ADR (cierre documental con addendum en ADR-286 a cargo del Ingeniero).
- **B-286-11 (P2):** spans en `Call`/`HirCall`; **sigue abierto**.
- **B-293-1 (P3, propuesta):** variables no ligadas siguen tipadas `Int` en silencio (PREP L60 y L224; no re-verificado por lectura aquí).
- **B-293-2 (P3, propuesta, [H]):** las llamadas a fns importadas siguen tipándose `Int` por el fallback de `fn_rets` en el flujo de fichero suelto con `use` (PREP L104); D4 solo las exime, no las resuelve.
- **B-293-3 (P3):** en v0 todo callee con `::` queda exento (también un `modulo::f` inexistente); su existencia la resuelve el CLI (E0404/E0405/E0406/E0331). Acotar la exención de `::` a una lista cerrada en un ADR de métodos asociados.

## 6. Riesgos

1. **Imports:** sin la exención, el barrido da 16 cambios en vez de 1 (D4); tocar `HirModule` toca 13 literales de test.
2. **Ownership del neg con ADR-291:** un fixture solo puede esperar un código (D7); si se invirtiera el orden, ADR-291 necesitaría un check por nombre `mutex_*` previo a esta regla.
3. **Cobertura de la medida:** el BARRIDO usa `use` aproximado en el «modo B» (sin comprobar `pub`, aproxima `package.rs` L500); no cubre fuentes `.arita` compuestas con `format!` en tests ni los fixtures que Measure copia en `/tmp` (PREP L124–L130). Las cifras de §2 son de una copia, no de un GATE.
4. **Futuros builtins:** un builtin libre nuevo, closures o fn como valor deben entrar en la lista de exenciones en el mismo ADR que los introduzca (regla de ADR-286 §0.1d (c), PREP L223).
5. **Isla lógica:** `ejemplos/f3/` (Datalog, `arita logic`) no pasa por parser/HIR (`E0006` en `spec`); la regla no la afecta [PREP L16].

## 7. GO, orden y medición

- PREP de Measure sin BLOCKER; implementación en `arita-hir` (+ oráculos y fixtures de Measure); Codegen y Parser sin cambios.
- **GO IMPL** lo da el Ingeniero. **Orden:** ADR-293 va ANTES que ADR-291 (cola de Lex; **(corregido)** el GATE de ADR-292 decía ADR-293 → ADR-291 → S2 MUST-USE; el Sello fija ADR-293 → ADR-294 → ADR-291 → S2); una carga pesada a la vez.
- Medición con evidencia real (measure exclusivo, `cargo test`, fmt, clippy, Veyra); prohibido inventar PASS / N/N / CLOSED. Cierre con GATE propio `DOC/GATE-CORE10-UNDECLARED-CALL-<fecha>.md`.

## 8. Preguntas del Ingeniero — RESUELTAS (Sello, 2026-10-04)

- **Q1 — RESUELTA (Sello):** fixtures en `ejemplos/core10/undeclared-call/neg/01-stmt.arita`, `02-let.arita`, `03-print-arg.arita` y `04-in-result-fn.arita`, y control en `ejemplos/core10/undeclared-call/05-declared-ok.arita` (fuera de `neg/`); ids de oráculo `neg-core10-undeclared-call-{stmt,let,print-arg,in-result-fn}` y, para el control, `core10-declared-call-ok` (sin prefijo `pos-`).
- **Q2 — RESUELTA (Sello):** en v0 todo callee con `::` queda exento (también un `modulo::f` inexistente); su existencia la resuelve el CLI (E0404/E0405/E0406/E0331). Abre **B-293-3 (P3)** (§5).
- **Q3 — RESUELTA (Sello):** aceptable en v0: la exención por nombre de item es deliberada; la distinción de ruta queda con B-293-2.
- **Q4 — RESUELTA (Sello):** el Arquitecto corrige ADR-291 ya (sin esperar al GO de este ADR): quita su «Migración única» y fija k = 4; su N lo recalcula el Ingeniero al GO IMPL con los N reales de la cola. **Hecho:** ADR-291 v0.2 (sin «Migración única», k = 4 fijo, sin cifras de N).
- Sin preguntas abiertas.

## 9. Changelog

- 2026-10-04 — v0.1 DRAFT PINS (Arquitecto): decisiones del Ingeniero 2026-10-04: E0347 con mensaje ``call to undeclared function `NAME` `` sin span; punto de comprobación en el hueco de `eval_expr` tras E0320/E0313/E0206/E0321 y antes de los argumentos; `HirModule.imports` obligatorio (barrido: 16 → 1 cambio, 867 `.arita`, 0 roturas); k = 5 (4 negs + control); migración de `neg-core03-compose-mutex-hold` a E0347 (diez freezes que la contienen, ninguno reescrito); interacción con ADR-291 (el neg no migra a E0346); N = 871 provisional, recalcula el Ingeniero; sin GO IMPL.
- 2026-10-04 — v0.1 (backlog, Arquitecto): añadido **B-293-3 (P3)** (acotar la exención de `::` a una lista cerrada en un ADR de métodos asociados) y Q2 de §8 marcada RESUELTA por el Ingeniero (todo callee con `::` exento en v0). Sin cambios en D1–D8, k ni N; el «Sello del Ingeniero» no se toca.
- 2026-10-04 — v0.1 (reconciliación con el Sello, Arquitecto): ids de oráculo (`neg-core10-undeclared-call-*`; control `core10-declared-call-ok`, no `pos-core10-declared-call-control`) y rutas de fixture (`ejemplos/core10/undeclared-call/neg/{01-stmt,02-let,03-print-arg,04-in-result-fn}.arita`; control `05-declared-ok.arita` fuera de `neg/`) alineados con las respuestas selladas; Q1–Q4 de §8 marcadas RESUELTAS; fórmula de B-293-3 igual a la del Sello; orden de la cola en §7 → 293 → 294 → 291 → S2. Solo por encima del Sello; el «Sello del Ingeniero» no se toca. Sin cambios en D1–D8, k ni N.

## Sello del Ingeniero (2026-10-04 12:45)
Sello del v0.1 de sha256 `db6d8b5338b8faf910b8a31b92475716d87e7a7d340efa747387d202fb640469`: pins D1–D8 y k = 5 aprobados; **GO IMPL pendiente** (cola de Lex: ADR-293 es el siguiente, antes de ADR-294, ADR-291 y S2). Respuestas a §8:
- **Q1:** fixtures en `ejemplos/core10/undeclared-call/neg/01-stmt.arita`, `02-let.arita`, `03-print-arg.arita`, `04-in-result-fn.arita` y control en `ejemplos/core10/undeclared-call/05-declared-ok.arita`; ids de oráculo `neg-core10-undeclared-call-{stmt,let,print-arg,in-result-fn}` y, para el control, `core10-declared-call-ok` (patrón de los positivos recientes, sin prefijo `pos-`).
- **Q2:** en v0 todo callee con `::` queda exento (también un `modulo::f` inexistente); su existencia la resuelve el CLI (E0404/E0405/E0406/E0331). Se abre B-293-3 (P3): acotar la exención de `::` a una lista cerrada en un ADR de métodos asociados.
- **Q3:** aceptable en v0: la exención por nombre de item es deliberada; la distinción de ruta queda con B-293-2.
- **Q4:** el Arquitecto corrige ADR-291 ya (sin esperar al GO de este ADR): quita su «Migración única» y fija k = 4; su N lo recalculo yo al GO IMPL con los N reales de la cola.
