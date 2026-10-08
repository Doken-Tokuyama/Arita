# ADR-294 — Core KNOWN-INT-SCOPE v0: alcance de `known_int` en límites de rama, bucle y bloque (B-292-2, P1, entra en v1)

- **Estado:** **DRAFT PINS v0.2 (Arquitecto, 2026-10-04; decisiones del Ingeniero incorporadas; respuestas Q1–Q5 incorporadas; pendiente Sello del Ingeniero).** Sin GO IMPL; pendiente la PREP de Measure (§7). No declara PASS, CLOSED ni N/N medido.
- **CUT-ID:** `CORE-0.10-KNOWN-INT-SCOPE-20261004`
- **Fecha:** 2026-10-04
- **Autores:** ARITA Arquitecto (pins) · decisiones del Ingeniero 2026-10-04 · hechos de Parser/HIR en `DOC/reviews/PREP_B292_2_20261004.md` (sha256 `b6cf58f84b488d82b3653e1e28b7a85db871ec2baa5004326515e9bb6c73d473`, 383 líneas; en adelante **PREP**) y 17 casos en `/tmp/arita_b292/cases/` (fuera del árbol)
- **Padre / contexto:** [ADR-292](292-core-int-arith-runtime-v0.md) (D5 / B-292-2; la etiqueta «hipótesis» queda **SUPERADA**: el falso positivo está **MEDIDO** por el PREP, casos fp1–fp4 y fp7; B-292-2 **sube de P2 a P1**) · [ADR-045](045-e0217-int-overflow.md) (E0217) · [ADR-044](044-e0216-int-div0.md) (E0216) · [ADR-290](290-core-index-mut-v0.md) (B-290-2) · [ADR-293](293-core-undeclared-call-v0.md) (orden: 293 → 294) · PREP Parser
- **Cierra (al CLOSED):** **B-292-2** (P1, HIR): falsos positivos de E0217/E0216 por `known_int` no sensible al flujo.
- **Código:** **ninguno nuevo.** E0217 (`integer overflow`) y E0216 (`integer division by zero`) conservan texto y significado; solo se eliminan falsos positivos.
- **No reabre:** el texto de E0216/E0217 · ADR-292 (CLOSED; se precisa por addendum) · `check_body` y el arm `Let` (reserva para S2 MUST-USE) · Parser y Codegen (no cambian) · los falsos negativos fn1–fn4 (B-294-2).

## 1. Decisión y límites

**D1 — Opción A (mínima).** Invalidar `known_int` en los **límites de rama** `if`/`match` (y `if let`/`while let`), de **bucle** y de **ámbito de bloque**. Sin análisis de flujo: la opción B (mapa `known` por camino, unión por intersección, punto fijo en bucles, `pop` de ámbito) queda **rechazada para v1** (PREP §7: +120–200 l., 15–20 tests, toca `check_body` y todos los arms).

**D2 — UN helper único dentro de `check_stmt`.** Descripción en términos del PREP (§7, opción A):
1. `clobbered(body)`: recorre los cuerpos anidados (plantilla: `stmt_uses_ident` / `body_uses_ident`) y devuelve los nombres con `Assign` o `Let` en **cualquier profundidad**.
2. Antes de **cada rama hermana** (`then`/`else`, cada brazo de `match`): se restaura el **snapshot** de `known_int` previo a la sentencia para esos nombres (`Vec<(String, Option<i64>)>`).
3. Antes de un **cuerpo de bucle**: `known_int = None` para esos nombres (efecto «2.ª vuelta»).
4. **Tras la sentencia**: `known_int = None` para esos nombres (fusión conservadora).
5. **Cobertura de `match` (Q4, Ingeniero 2026-10-04: SÍ):** el helper cubre el `match` con binding de patrón (Result/Option/enum) y **todos** los puntos de llamada del arm `Match`, incluido el brazo enum que llama `check_stmt` por sentencia (HIR ~L4127; puntos L3925, L4006, L4084 y L4127). La restauración de `known_int` no sustituye ni modifica la restauración del binding de patrón existente; el orden exacto lo fija IMPL y lo fija al menos un unit test `adr294_match_pattern_binding_*` (§3).

**D3 — Lo que NO se toca.** `check_body` y el arm `Let` (incluido `bindings.insert`) **no se modifican**: sus puntos son los ganchos de S2 MUST-USE (arm `Let` tras `bindings.insert`, arm `Expr` y cola de `check_body`; PREP §8). La opción A edita solo los arms de control de flujo de `check_stmt` (`If`, `IfLet`, `While`, `WhileLet`, `Match`) y el helper.

**D4 — Sin código E nuevo.** E0217 sigue válido **en recto**: c5 (`let mut a = MAX; a + 1`) y c6 (`a = a + 1`) conservan E0217. También lo conserva cualquier sentencia de control que **no** asigne ni haga `let` del nombre (el nombre no está en `clobbered`).

**D5 — Falsos negativos fn1–fn4 NO se arreglan.** Siguen sin E0217 y el desbordamiento ya hace **panic 101** en runtime por ADR-292 (D1). Quedan como tests HIR de control «sigue Ok» y como backlog B-294-2.

**D6 — Orden y alcance de IMPL.** Solo `arita-hir`. Orden de IMPL en Lex: **ADR-293 → ADR-294 → ADR-291 → S2 MUST-USE** (Ingeniero). Sin cambios de Codegen ni de Parser: ni un byte de Rust emitido cambia para los programas que hoy compilan.

## 2. Evidencia (PREP + código)

- **Mecanismo** (PREP §3, verificado por lectura en `crates/arita-hir/src/lib.rs`, sha256 `d0571e12419dae491ec7ce5dcfba99ad247c045793fc3c1f6c53abf1ee575807`, 9630 líneas, sin cambios desde el PREP): `bindings` es un único `HashMap` plano por función; `check_body` (L4141–L4146) es un `for s in body { check_stmt(s)? }` sin abrir ni cerrar ámbito; `If` (L3696–L3711) comprueba `then` y `else` sobre el **mismo** mapa; los bucles se comprueban **una vez**, en orden fuente; un `let` interior hace `bindings.insert` y **no** se elimina al salir del bloque; solo los bindings de patrón de `if let`/`while let`/`match` se restauran (L3739–L3750, L3802–L3812, brazos de `match`). La reasignación **recta** es correcta (`Assign`, L3838–L3855: literal ⇒ `Some(n)`, otro RHS ⇒ `None`). El defecto está en la **fusión de caminos**, el **bucle** y el **ámbito**.
- **Falso positivo MEDIDO** (PREP §4.1; `arita parse` con el binario `target/debug/arita` ya construido, no reconstruido, no verificado por hash de fuentes): fp1, fp2, fp3, fp4 ⇒ `E0217: integer overflow`; fp7 ⇒ `E0216: integer division by zero`; los cinco son programas válidos sin ruta de ejecución que desborde ni divida por 0. Además fpw5 y fpw6 ⇒ `E0217` (**dependientes del camino**: con `n = 3` fpw5 desborda en runtime; fpw6 desborda tras dos vueltas; ver §4 y Q2).
- **Falsos negativos medidos** (PREP §4.2): fn1–fn4 dan `ok` en `arita parse` y desbordan en runtime (panic 101 [L] por ADR-292).
- **Controles** (PREP §4.3): c1–c4 `ok` hoy; c5 y c6 `E0217` hoy. Los 17 ficheros de `/tmp/arita_b292/cases/` son **idénticos, bloque a bloque**, a las fuentes del PREP (comprobado hoy).
- **Corpus** (PREP §5, léxico, no ejecutado hoy): 867 `.arita`, 16 asignaciones `ident = …` en 9 ficheros, **0** con RHS literal entero; 0 sombras `let x: Int = <lit>` anidadas sobre un `x` exterior; **0 positivos afectados, 0 falsos positivos en el corpus**. Riesgo para v1 según el PREP: **MEDIO** (probabilidad 0/867; severidad: programa válido rechazado sin escape salvo reescribir).
- **Todos los consumidores de `fold_i64`** quedan contaminados por el mismo defecto: 39 líneas / 42 ocurrencias de `self.fold_i64` (re-verificado hoy), entre ellas E0217 `+ - *` (L724–L739), E0217 MIN÷−1 (L3315, L3339) y E0216 (p. ej. L2832). Por eso el arreglo es en `known_int`, no en E0217.
- **Referencias de código verificadas por lectura:** `BindingState` L375–L380 (`known_int` L380); `fold_i64` L455–L463; E0217 L724–L739; `Let` L3594, cálculo de `known_int` L3652–L3655 y `bindings.insert` L3666–L3680; `Expr` L3683; `If` L3696–L3711; `IfLet` L3713–L3757; `WhileLet` L3759–L3818; `While` L3820–L3830; `Assign` L3838–L3855; `Match` L3876–L4137; `check_body` L4141–L4146; `stmt_uses_ident` L4363; `body_uses_ident` L4419; altas con `known_int: None` en L3739, L3802, L3912, L3993 y L5078. Puntos de llamada a `check_body`/`check_stmt` dentro de los arms de control: L3707, L3709, L3748, L3755, L3811, L3828, L3925, L4006, L4084 y L4127 (el brazo `match` de enum comprueba por sentencia con `check_stmt`).

## 3. Cambio mínimo previsto (a verificar en IMPL)

- **`arita-hir`, `check_stmt`:** el helper de D2 y su uso en los arms `If`, `IfLet`, `While`, `WhileLet`, `Match` (los tres tipos de scrutinee del `Match`: Result/Option, Bool/Int y enum). **Estimación del PREP, no medida:** +45–60 líneas de producción.
- **Tests HIR (unit, fuera de los oráculos de Measure):** ~10 tests `adr294_*` (el PREP los llamaba `adr292b_*`; **(corregido)** al id de este ADR), ~150 l. según el PREP, con las fuentes en `const` **fuera** de `#[test]` (B-286-5):
  - fp1, fp2, fp3, fp4, fp7, fpw5, fpw6 ⇒ `Ok` (fpw5 y fpw6 son **solo unit**, no corpus: Q2);
  - c1–c6 como controles (c1–c4 `Ok`, **solo unit**, no se promueven a oráculos: Q3; c5 y c6 `E0217`);
  - fn1–fn4 ⇒ «sigue Ok» (controles del falso negativo conocido);
  - **al menos uno** `adr294_match_pattern_binding_*` (Q4): `match` con binding de patrón (Result/Option/enum) y brazo enum con `check_stmt` por sentencia, comprobando que un `Assign` en un brazo no contamina al hermano ni sobrevive tras el `match`.
  
  El reparto exacto en funciones de test (17 fuentes, ~10 tests) lo decide IMPL. **0 tests existentes modificados** (PREP §7: los tests HIR de E0217 `e0217_*` L6121–L6236 son rectos; ningún test HIR construye `HirStmt::Assign`; `adr283_eq_set_negative_lit_and_known_int_e0319` L8579 es recto).
- **Measure:** registrar los 7 oráculos de §4: fp1, fp2, fp3, fp4 y fp7 (fixtures nuevos en `ejemplos/core10/known-int/`), c6 (fixture **nuevo** en `ejemplos/core10/known-int/neg/`) y c5, que **REUSA** el fixture existente `ejemplos/f2/neg/e0217-runtime-max-plus.arita` sin modificarlo ni duplicarlo (solo se cablea en measure, mismo contrato E0217, exit 1); lo hace Measure/Ingeniero.
- **Sin cambios:** `arita-syntax`, Codegen, `arita-cli` (salvo el registro de oráculos), `package.rs`.
- **Validación tras el gate (PREP §9.4, no ejecutada):** `cargo test -p arita-hir adr294`, `cargo test -p arita-hir`, `cargo clippy -p arita-hir --all-targets -- -D warnings`, `cargo fmt --check`, y `arita parse` de los casos esperando Ok en fp1–fp4, fp7, fpw5, fpw6, c1–c4 y `E0217` en c5 y c6.

## 4. Oráculos (k = 7)

**Ids y rutas aceptados por el Ingeniero (Q1, 2026-10-04):** positivos en `ejemplos/core10/known-int/`, c6 en `ejemplos/core10/known-int/neg/`, c5 reusa un fixture existente (ver abajo). Los **nombres de fichero** dentro de esos directorios son los de los casos del PREP (`fp1-else-bleed.arita`, …) y los confirma IMPL. Las fuentes son las de los casos del PREP (`module b292`; el nombre de módulo puede cambiar al crear los fixtures). **El PREP solo validó con `arita parse`: que estos positivos compilen y ejecuten con rustc/cargo [NO MEDIDO]** (riesgo en §6).

**Positivos (deben COMPILAR y ejecutar; exit 0).** Stdout exacto derivado de la fuente (`print` escribe una línea por llamada), **[no ejecutado]**:

1. `core10-known-int-fp1-else-bleed` — caso `fp1-else-bleed`. `n = 3` ⇒ solo se ejecuta el `then`. stdout: `ok`.
2. `core10-known-int-fp2-match-arm` — caso `fp2-match-arm-bleed`. `flag = true` ⇒ solo el brazo `true`. stdout: `ok`.
3. `core10-known-int-fp3-loop-carried` — caso `fp3-loop-carried`. En la vuelta `i == 1`, `a` vale 0 ⇒ `r = 1`. stdout: `1` / `ok`.
4. `core10-known-int-fp4-shadow-leak` — caso `fp4-shadow-leak`. Imprime la sombra interior, luego `a + 1` con `a = 0`. stdout: `9223372036854775807` / `1` / `ok`.
5. `core10-known-int-fp7-e0216-else` — caso `fp7-e0216-else-bleed`. `n = 3` ⇒ solo el `then` (`d = 0`); el `else` con `x.div_euclid(d)` no se ejecuta. stdout: `ok`.

**Negs de control en recto (conservan su error).** Cada neg: texto exacto `E0217: integer overflow`, exit 1, stdout vacío, sin emisión de Rust:

6. `neg-core10-known-int-c5-straight` — **REUSA el fixture existente `ejemplos/f2/neg/e0217-runtime-max-plus.arita`** (`let a: Int = MAX; let x: Int = a + 1`; sha256 `34c6bf8f63673596b7b323721aa2406e11e330c2d370577e962887b3bf9a27b5`, verificado hoy), **sin modificarlo ni duplicarlo**; hoy no está cableado en measure (su propio comentario lo dice) y este ADR solo lo cablea, con el mismo contrato E0217 y exit 1. Equivale al caso `c5-true-positive` del PREP (la fuente del PREP difiere en `print(r)`; mismo programa en lo que importa). Su sha se registra en el freeze nuevo (ya figura en freezes anteriores, p. ej. `MEASURE-ADR292-INT-ARITH-RUNTIME-FREEZE-20261003.sha:957`; ninguno se reescribe).
7. `neg-core10-known-int-c6-assign-overflow` — caso `c6-assign-overflow-expr` (`a = a + 1` con `a = MAX`); fixture **NUEVO** en `ejemplos/core10/known-int/neg/`.

**k = 7 CONFIRMADO (Ingeniero, 2026-10-04)**: el número exacto de oráculos medibles enumerados arriba (5 positivos + 2 negs). Los controles c1–c4 **no** son oráculos (Q3: son unit tests); fpw5 y fpw6 **no** son oráculos (Q2: unit tests). **N de la cola — cifras del Ingeniero (2026-10-04), citadas como tales:** 866 → 871 (ADR-293, k = 5) → **878 (este ADR, k = 7)** → 882 (ADR-291, k = 4) → +k(S2); orden 293 → 294 → 291 → S2; el Ingeniero recalcula al GO. Reglas: skip ≠ PASS; ningún oráculo cuenta si el build no se ejecutó. El freeze y el addendum los escribe el Ingeniero; ningún freeze existente se reescribe.

## 5. Backlog

- **B-292-2 (P2 → P1):** se cierra al CLOSED de este ADR (cierre documental con addendum en ADR-292 a cargo del Ingeniero).
- **B-294-1 (P2, propuesta):** sombra heredada de `ty`/`mutable`/`moved` por un `let` interior. La opción A arregla `known_int` pero un `let` interior sigue reemplazando de forma permanente el `BindingState` entero del exterior (PREP §3.5 y §7). `moved` es una **sospecha por lectura, no medida** [H].
- **B-294-2 (P3, propuesta):** fn1–fn4 — E0217 sensible al flujo (opción B) para detectar en compilación el desbordamiento alcanzable por camino; hoy lo cubre el panic 101 de ADR-292.
- **B-292-1 y demás backlog de ADR-292:** sin cambios.

## 6. Riesgos

1. **Pérdida de detección (esperada):** tras una sentencia que asigna o hace `let` de un nombre en una rama, el nombre deja de ser foldable; E0217/E0216 dejan de saltar en casos «definitivos» que solo se producían por camino (fpw5, fpw6 ya no dan E0217; PREP §7). Coherente con ADR-045 («same-scope») y cubierto en runtime por ADR-292.
2. **Colisión con S2 MUST-USE:** mitigada por D3 (no tocar `check_body` ni `Let`); se espera desplazamiento de líneas, no conflicto de contexto (PREP §8).
3. **`Match` con varias formas (Q4 resuelta: SÍ):** el arm `Match` tiene varias formas de comprobación (Result/Option, Bool/Int, enum) con cuatro puntos de llamada separados (L3925, L4006, L4084, L4127); el helper debe cubrirlos todos, incluido el binding de patrón y el brazo enum por sentencia; lo fija al menos un test `adr294_match_pattern_binding_*`.
4. **Compilación de los positivos [H, no medido]:** el PREP solo corrió `arita parse`. El Rust emitido para fp1–fp4 y fp7 podría producir avisos de rustc (p. ej. `unused_assignments`) o activar el lint por defecto `arithmetic_overflow`/`unconditional_panic` sobre valores propagados como constantes; si Measure compila sin `-D warnings` solo importa lo segundo. Debe resolverlo la PREP de Measure antes del GO.
5. **El binario del PREP:** las mediciones `[M]` usan un `target/debug/arita` existente cuya correspondencia exacta con el HIR actual no se verificó por hash (el mtime es compatible).
6. **Corpus:** 0/867 afectados (léxico, PREP §5); el riesgo de regresión sobre el corpus se mide en IMPL con measure y `cargo test`.
7. **Closures / `spawn` con cuerpo (Q5):** hoy no existen (`rg -i 'closure|lambda'` sin coincidencias en `arita-hir` ni en `arita.pest`; `spawn` toma una llamada, no un cuerpo). **Nota para el ADR que los introduzca:** deberá extender `clobbered` a esos cuerpos; de lo contrario un `Assign` dentro de ellos no invalidaría `known_int` y reaparecería el defecto que este ADR corrige.

## 7. GO, orden y medición

- PREP de Measure sin BLOCKER; implementación en `arita-hir` (+ oráculos y fixtures de Measure); Codegen y Parser sin cambios.
- **GO IMPL** lo da el Ingeniero. **Orden:** ADR-293 → ADR-294 → ADR-291 → S2 MUST-USE; una carga pesada a la vez. **(corregido)** El PREP (§9.2) recomendaba el slice «tras ADR-291»; prevalece el orden del Ingeniero.
- Medición con evidencia real (measure exclusivo, `cargo test`, fmt, clippy, Veyra); prohibido inventar PASS / N/N / CLOSED. Cierre con GATE propio `DOC/GATE-CORE10-KNOWN-INT-SCOPE-<fecha>.md`.

## 8. Preguntas del Ingeniero — RESUELTAS (2026-10-04)

- **Q1 — RESUELTA:** ids y rutas aceptados: positivos `core10-known-int-fp1-else-bleed`, `-fp2-match-arm`, `-fp3-loop-carried`, `-fp4-shadow-leak` y `-fp7-e0216-else` en `ejemplos/core10/known-int/`; negs c5 y c6. **c5 REUSA** el fixture existente `ejemplos/f2/neg/e0217-runtime-max-plus.arita` sin modificarlo ni duplicarlo (solo se cablea en measure; mismo contrato E0217, exit 1; su sha se registra en el freeze); **c6 fixture NUEVO** en `ejemplos/core10/known-int/neg/` (§4).
- **Q2 — RESUELTA:** fpw5 y fpw6 son **tests unitarios**, no corpus.
- **Q3 — RESUELTA:** c1–c4 **NO** se promueven: unit tests.
- **Q4 — RESUELTA (SÍ):** el helper cubre `match` con binding de patrón (Result/Option/enum) y el brazo enum que llama `check_stmt` por sentencia (HIR ~L4127); al menos un unit test `adr294_match_pattern_binding_*` (D2.5, §3).
- **Q5 — RESUELTA:** sin closures/`spawn` con cuerpo hoy; queda nota de riesgo para el ADR que los introduzca (§6.7).
- Sin preguntas abiertas.

## 9. Changelog

- 2026-10-04 — v0.1 DRAFT PINS (Arquitecto): decisiones del Ingeniero 2026-10-04: opción A (invalidar `known_int` en límites de rama/bucle/bloque, un helper único en `check_stmt`, sin tocar `check_body` ni `Let`); sin código E nuevo; fn1–fn4 a backlog (B-294-2) y B-294-1 propuesto; B-292-2 P1 (falso positivo MEDIDO, etiqueta «hipótesis» de ADR-292 D5 superada); k = 7 propuesto (5 positivos + 2 negs), N = 878 provisional (871 + 7), recalcula el Ingeniero; orden 293 → 294 → 291 → S2; sin GO IMPL.
- 2026-10-04 — v0.2 DRAFT PINS (Arquitecto): respuestas del Ingeniero Q1–Q5 incorporadas: Q1 ids/rutas aceptados (positivos en `ejemplos/core10/known-int/`; c5 REUSA `ejemplos/f2/neg/e0217-runtime-max-plus.arita` sin modificar ni duplicar, c6 fixture nuevo en `ejemplos/core10/known-int/neg/`); Q2 fpw5/fpw6 solo unit; Q3 c1–c4 solo unit; Q4 el helper cubre `match` con binding de patrón y el brazo enum por sentencia (D2.5; test `adr294_match_pattern_binding_*`); Q5 nota de riesgo para closures/`spawn` (§6.7); k = 7 confirmado; N de la cola (cifras del Ingeniero): 866 → 871 (293) → 878 (294) → 882 (291) → +k(S2). Pendiente el Sello del Ingeniero (no se añade aquí); sin GO IMPL.

## Sello del Ingeniero (2026-10-04)

Sellado sobre la versión v0.2 con sha256 `33c10f540994005633857b1899934ca9d5f2b86359c7815dc104407fb900d64c` (109 líneas). Esta sección se añade al final; nada por encima se ha modificado.

**Decisión: v0.2 ACEPTADA como contrato de pins para ADR-294. Sin GO IMPL todavía.** El GO IMPL se da por escrito tras ADR-293 CLOSED y la PREP de Measure sin BLOCKER (orden 293 → 294 → 291 → S2; una carga pesada a la vez en Lex).

1. **Alcance (opción A) confirmado.** Un único helper dentro de `check_stmt`, usado solo en los arms `If`, `IfLet`, `While`, `WhileLet` y `Match`. No se toca `check_body`, el arm `Let` ni `Assign`. Sin código E nuevo. La opción B queda rechazada para v1.
2. **Oráculos k = 7 confirmados**: cinco positivos (fp1, fp2, fp3, fp4, fp7) en `ejemplos/core10/known-int/` y dos negs con E0217 exacto y exit 1 (c5 reutiliza `ejemplos/f2/neg/e0217-runtime-max-plus.arita` sin modificarlo ni duplicarlo; c6 es un fixture nuevo en `ejemplos/core10/known-int/neg/`). Cifras de la cola: 866 → 871 (293) → 878 (este ADR) → 882 (291) → +k(S2).
3. **Condición de aceptación de los positivos (riesgo §6.4).** Antes de contar un positivo como oráculo, Measure debe compilar y ejecutar el Rust emitido con rustc y fijar el stdout real; el stdout derivado a mano de §4 no vale como evidencia. Si rustc rechaza o avisa (p. ej. `arithmetic_overflow` o `unconditional_panic` sobre valores propagados como constante), es un BLOCKER que vuelve al Arquitecto. No se relaja el contrato ni se saca el caso del conjunto sin mi decisión por escrito.
4. **Cobertura de `match`.** El unit test `adr294_match_pattern_binding_*` es obligatorio y debe cubrir los cuatro puntos de llamada del arm `Match` (Result/Option, Bool/Int y el brazo enum que llama `check_stmt` por sentencia). Un `Assign` en un brazo no puede contaminar al hermano ni sobrevivir tras el `match`.
5. **Tests unitarios sin tocar nada existente.** fpw5, fpw6, c1–c4 y fn1–fn4 quedan solo como unit tests. Cero tests existentes modificados; las fuentes van en `const` fuera de `#[test]`. El parche debe pasar `cargo fmt --check` y `cargo clippy -- -D warnings` antes de aplicarse al árbol real.
6. **Parser.** El diff de la validación en `/tmp` se rehace sobre este v0.2 (k = 7). Solo vale el diff final que el Parser entregue con su sha256, validado en cadena 293 → 294 → 291 sobre una copia limpia.
7. **Cierre.** El gate exige fmt 0, clippy `-D warnings` 0, build release, `cargo test --workspace -- --test-threads=1`, measure N/N sin skip, Veyra ACCEPTED y GATE propio `DOC/GATE-CORE10-KNOWN-INT-SCOPE-<fecha>.md`. El freeze y el addendum los escribo yo; ningún freeze ni sello anterior se reescribe. El cierre documental de B-292-2 se hace con un addendum en ADR-292 por mi parte, sin editar su Sello.
8. **Backlog confirmado.** B-294-1 (P3: sombra heredada de `ty`/`mutable`/`moved`; `moved` es una sospecha no medida) y B-294-2 (P3: falsos negativos fn1–fn4, cubiertos en ejecución por el panic 101 de ADR-292). B-292-2 sube a P1 y se cierra con este ADR.
9. **Nota para el futuro.** Si se introducen closures o `spawn` con cuerpo, su ADR debe extender `clobbered` a esos cuerpos (§6.7).
