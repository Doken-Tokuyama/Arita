# ADR-291 — Core Mutex-reject v0: `Mutex`/`Arc`/`.lock()` fuera del surface (Core 0.10)

- **Estado:** **DRAFT PINS v0.2 (Arquitecto, 2026-10-04; decisiones del Ingeniero incorporadas; pendiente GO; v0.1 pedido tras PREP de B-286-7 → superado por ADR-293).** ADR-290 está CLOSED (GATE `DOC/GATE-CORE10-INDEX-MUT-20261003.md`) y ADR-292 CLOSED (GATE `DOC/GATE-CORE10-INT-ARITH-RUNTIME-20261003.md`); la verificación y el GO de este ADR los da el Ingeniero. Este texto solo fija una propuesta comprobable; no declara PASS, N/N ni CLOSED. **Orden de la cola (Ingeniero, 2026-10-04): ADR-293 → ADR-294 → ADR-291 → S2 MUST-USE.**
- **CUT-ID:** `CORE-0.10-MUTEX-REJECT-20261003` (provisional; lo confirma el Orquestador)
- **Fecha:** 2026-10-03
- **Autores:** **ARITA Arquitecto**; alcance y orden pendientes de confirmación del Ingeniero
- **Padre / contexto:** [ADR-229](229-lang-contract-mutex.md) (Mutex PARK/E0312), [ADR-286](286-core-0.10-errores-fase1.md) (B-286-7 y B-286-12), [ADR-039](039-e0242-borrow-across-await.md) (E0242), [ADR-290](290-core-index-mut-v0.md) (CLOSED), [ADR-293](293-core-undeclared-call-v0.md) (B-286-7 y la migración del neg `neg-core03-compose-mutex-hold`, E0347; va ANTES que este ADR), [Parser IMPL r2](../reviews/ADR-291-PARSER-IMPL-DRAFT-20261004-r2.md) (parche del Parser validado en copia; §2.5)
- **Cierra (al CLOSED):** el hueco de diagnóstico propio para la superficie Mutex-reject. **La migración del neg `neg-core03-compose-mutex-hold` pertenece a ADR-293 (E0347)**, no a este ADR.
- **Código propuesto:** **E0346** — mensaje EN canónico: **`mutex concurrency is not available in this surface`**.
- **No reabre:** E0242 `borrow held across await`, E0312 en su contrato histórico, E0340–E0345, ADR-229/286/290, ni el código de emisión. E0313 no se reutiliza.

## 1. Decisión propuesta y límites

**D1 — Naturaleza.** En v0, `Mutex`, `Arc<Mutex<..>>` y el método `.lock()` **no existen en el surface de ARITA**: se rechazan con E0346, antes de emitir Rust. La salida es accionable y estable; no se acepta como diagnóstico de Mutex el E0100 accidental de una función inexistente (esa llamada libre, p. ej. `mutex_new(0)`, pertenece a ADR-293, E0347).

El texto canónico de E0346 es exactamente:

```text
E0346: mutex concurrency is not available in this surface
```

El diagnóstico se ancla en el identificador de tipo (`Mutex`/`Arc`) o en el selector de método `lock`, con el span más pequeño disponible. La forma exacta de obtener el span de `MethodCall` queda como Q1 (§8).

**D2 — Superficie cerrada de v0.** Entra únicamente:

1. `Mutex` como tipo o nombre de construcción, incluido `Mutex<T>` cuando el parser pueda reconocerlo.
2. `Arc`, y la forma compuesta `Arc<Mutex<T>>` cuando el parser pueda reconocerla.
3. `.lock()` sobre un receptor.

`RwLock` queda **fuera**: el PREP del Parser no lo trata y no hay evidencia de una regla o contrato actual para él.

**D3 — Fuera de alcance.** Mutex funcional, `Guard` funcional, `unlock`, guard-RAII, `thread_local`, locks async, sharing entre tasks, `spawn` con argumentos, política de poisoning, `Arc` emitido y cualquier cambio de Codegen. El Mutex real se recomienda para v1.1, no para este slice.

**D4 — Alternativas futuras.** El PREP del Parser deja dos formas posibles para una v1 posterior: (a) rechazo propio con código ARITA; (b) tipos opacos `Named` (`Mutex`/`Guard`) con un neg E0346. Este ADR recomienda **(a)** para v0: no abre una semántica parcial ni crea un guard cuyo lifetime aún no puede acotarse. La alternativa (b) queda como diseño de v1.1, no como pin de este ADR.

**D6 — NO (B-286-7 vive en ADR-293).** Este ADR no trata ni cierra B-286-7 (llamada a función no declarada tipada `Int`). `mutex_new(0)` y cualquier constructor libre no declarado se rechazan con **E0347** por ADR-293, **no** con E0346; este ADR no migra ningún neg existente.

## 2. Hechos verificados: qué llega hoy al parser y al HIR

### 2.1 Tipos genéricos

- `ty_named = { ident }` en `crates/arita-syntax/src/arita.pest:74` no admite `<...>`.
- Las reglas genéricas son cerradas: `ty_vec_int` L63, `ty_list_int` L65, `ty_map_text_int` L66, `ty_result` L69 y `ty_option` L72; `let_stmt` usa la lista cerrada en L112–115.
- Por ello `let m: Mutex<Int> = ...` y `Arc<Mutex<Int>>` mueren en parse como E0006 genérico (`construct outside F1.1 (parse failure)`) antes de HIR: `DOC/reviews/PREP_MUTEX_PARSER_20261003.md:40–56`.
- Un parámetro o retorno `Mutex` sin `<...>` sí pasa por `ty_named`: `arita.pest:84`, `:276–277` **(corregido: antes `:274–277`)**; el lowering produce `Type::Named` (`crates/arita-syntax/src/lib.rs:2140`, `:2467–2469`).

### 2.2 `.lock()` y constructores asociados

- `method_call_other = { ident ~ "." ~ ident ~ "(" ... }` en `arita.pest:123`; `m.lock()` baja a `Expr::MethodCall` (`DOC/reviews/PREP_MUTEX_PARSER_20261003.md:64–72`).
- No hay encadenado: `m.lock().unwrap()`, `*m.lock()` y `m.lock().len()` no son expresables hoy (`PREP_MUTEX_PARSER_20261003.md:64–72, 108–119`).
- `Mutex.new(0)`/`Arc.new(0)` tienen la forma de constructor asociado que el HIR intenta resolver por nombre (`PREP_MUTEX_PARSER_20261003.md:71, 96–106`). Hoy terminan en E0206 porque el nombre no está ligado o el método no está en la whitelist.
- `Type::Named` existe en AST/HIR, pero no hay validación de que el nombre exista: `PREP_MUTEX_PARSER_20261003.md:58–62, 96–106`.
- Una llamada libre no declarada como `mutex_new(0)` llega a HIR y devuelve `Int` por el comportamiento B-286-7 (hoy `crates/arita-hir/src/lib.rs:1305–1311`; **(corregido)**: la cita original L1276–L1281 estaba desplazada): después termina en E0100 tardío. `mutex_new` no es una forma del alcance cerrado D2; su tratamiento (E0347) pertenece a ADR-293.

### 2.3 Await, préstamos y guard

- E0242 solo cubre hoy dos arms de `HirExpr::Await` (`crates/arita-hir/src/lib.rs:881–907` **(corregido: antes `:852–878`)**); su contrato es `borrow held across await`, ADR-039 L23–31.
- Los préstamos HIR actuales proceden de `&`/`&mut` o del receptor temporal; un guard devuelto por `lock` sería un valor `Named`, no un `Loan` (`PREP_MUTEX_PARSER_20261003.md:142–148`).
- No hay scopes HIR push/pop para cuerpos de `if`, `while` o `match`; los préstamos viven hasta el final de la función (`PREP_MUTEX_PARSER_20261003.md:74–87`). Por eso guard-RAII y E0242 quedan fuera de v0.
- ADR-039 L59 mantiene Mutex×await en PARK; ADR-286 B-286-12 mantiene guard-RAII y `thread_local` como backlog P3 (`DOC/ADR/286-core-0.10-errores-fase1.md:263`).

### 2.4 Accidente actual del neg existente

`ejemplos/core03/client-compose/neg/02-mutex-hold.arita:1–8` contiene:

```arita
// CUT CORE-0.3-CLIENT-COMPOSE-20260920 — neg Mutex HOLD must reject
// Oracle: neg-core03-compose-mutex-hold. Do NOT unpark Mutex.
module core03_neg_mutex

async fn main() -> Io<()> {
  let _m: Int = mutex_new(0)
  print("should-not")
}
```

La evidencia del oráculo dice E0100, no un diagnóstico Mutex propio: `DOC/GATE-CORE03-CLIENT-COMPOSE-20260920.md:8–13`; el detalle de Measure está en `DOC/reviews/MEASURE_ADR274_SCENARIO_IO_20260926.json:2431–2434`. El PREP explica el recorrido: parse acepta `mutex_new`, HIR acepta la función inexistente como `Int`, Codegen la emite y rustc falla tarde (`PREP_MUTEX_PARSER_20261003.md:123–140`).

**La migración de `neg-core03-compose-mutex-hold` pertenece a ADR-293 (E0347)**, no a este ADR (decisión del Ingeniero, 2026-10-04): el fixture (`mutex_new(0)`) no contiene `Mutex`, `Arc` ni `.lock()`, así que no migra a E0346. Este ADR no lo toca; el fixture sigue congelado con la sha `4ea5e12123c7ae22e1b4af62827f9e1a45a8def29d0a84aa44c373b7da222d79` y sus freezes se enumeran en ADR-293 §5 (ninguno se reescribe).

### 2.5 Referencia de IMPL del Parser: `ADR-291-PARSER-IMPL-DRAFT-20261004-r2.md`

Fuente: `DOC/reviews/ADR-291-PARSER-IMPL-DRAFT-20261004-r2.md` (sha256 `fa35c7a89920961dd38d3c1b98c99393891bf9cea6d4689e609caa3e36fc5e8a`, 232 líneas; en adelante **r2**). Estado según r2 L3: «VALIDADO EN COPIA /tmp/arita_adr291 (parse + lower + HIR, tests de syntax/hir, clippy, fmt). Nada aplicado al árbol real». Las citas «r2 Ln» son líneas de r2, leídas hoy; las de código se re-verificaron hoy sobre el árbol (hir `d0571e12…`, syntax `187cff0e…`, pest `7c28aa3c…`, sin cambios).

- **Punto de intercepción: `arita-syntax`, no HIR** (r2 L105–L111). `Expr::MethodCall` (`crates/arita-syntax/src/lib.rs` L241) y `Call` (L287) no guardan span; `CheckError::Coded` de HIR no tiene campo span; un E0346 anclado al identificador solo puede emitirse donde existen los `Pair` de pest: el lowering o `classify_pest_error` (L518). `ty_named = { ident }` (`arita.pest` L74) hace que el span del par sea exactamente el identificador. `lower_method_call_other` (L927) se invoca desde L1262, L1608, L1993, L2550 y L3065; `lower_host_call` (L1899) es delegado por `lower_host_call_try` (L2983). Sitios que construyen `Type::Named`: L742, L791, L2140, L2187 y L2469. `arita.pest`, HIR y Codegen no cambian.
- **Qué se detecta** (r2 L114–L121): `Mutex`/`Arc` como tipo sin `<>` (let, param, ret, argumento de Result/Option) en un `lower_ty_named` nuevo; `Mutex<..>`/`Arc<..>` genéricos y `x.lock().m()`/`*x.lock()` por el camino de fallo de pest (escaneo léxico de la línea del fallo, tras E0005 y antes de E0007); `Mutex.new(..)`/`Arc.new(..)` en `lower_method_call_other` (ancla: el receptor); `<recv>.lock(..)` de cualquier receptor (ancla: el selector `lock`); `host.lock(..)` en `lower_host_call`.
- **Mensaje:** `E0346: mutex concurrency is not available in this surface`; el `Display` de `ParseError::Coded` añade ` @start..end` (syntax L343–L355; r2 L125), así que el stderr real es p. ej. `E0346: mutex concurrency is not available in this surface @60..65`.
- **Span de `MethodCall` (Q4):** ninguno tras el lowering (r2 L127–L128); los spans viven solo en los `Pair` (`recv`, `method_pair`) dentro de `lower_method_call_other`. Por eso HIR no puede anclar este código.
- **Precedencia** (r2 L130–L136 y L45): E0346 es de fase parse y gana a cualquier error de HIR (E0007, **E0347 de ADR-293**, E0203, E0206, E0343/E0344, E0340); observado en copia: `Mutex.new(0)` con `undeclared(1)` antes o después da E0346 en ambos órdenes. `mutex_new(0)` libre nunca da E0346 (con ADR-293, E0347).
- **Validación en copia** (r2 L18–L27, no es medida de GATE): `cargo fmt --check` y `clippy -D warnings` rc 0; `cargo test -p arita-syntax adr291` 7 passed y `-p arita-hir adr291` 5 passed; suites completas 85 (syntax) y 179 (hir) passed, 0 failed; con ADR-293 aplicado, mismos resultados. Barrido de 867 `.arita` base vs base+ADR-291: 0 cambios (r2 L40). Diffs finales (r2 L49–L50): `adr291_parser_final.diff` (189 l., sha256 `9c8fc38b756646e136dc089365dd948a345e886f0555a83f8f97ad103b664c37`) y `adr291_tests_final.diff` (269 l., sha256 `f5e3117d0351b6d8cd5443ea6240e163d3b3e87c85396ab2427fb945e513ab22`), ambos en `/tmp/arita_adr291/`.
- **Fuentes y spans de los 4 oráculos** (r2 L29–L37 y L160–L165): `/tmp/arita_adr291/oracles/01-type.arita` (`@60..65`), `02-arc.arita` (`@36..39`), `03-constructor.arita` (`@66..71`), `04-lock.arita` (`@85..89`); spans predichos = observados en copia. Recalculados hoy sobre esas fuentes: coinciden los cuatro (en `04-lock` el selector `.lock(` está en 85..89; la primera aparición de `lock` en el fichero es el nombre del módulo). Son fuentes nuevas; **no** el fixture congelado `mutex_new(0)`.

**Hallazgos (r2 vs código vs este ADR):**
1. **HIR → syntax.** El v0.1/v0 de este ADR situaba constructor y lock en HIR (§4, §5); r2 L211 lo contradice y L105–L111 lo demuestra por código. Corregido en §4 y §5 de esta v0.2.
2. **Citas desfasadas del propio ADR** (r2 L207): hir `852–878` → hoy **L881–L907** (brazo `Await`, E0242 en L887 y L899); `arita.pest:274–277` → hoy **L276–L277** (`fn_ret_ok`/`fn_ret_bad`). **(corregido)** en §2.1 y §2.3. Las citas a `PREP_MUTEX_PARSER_20261003.md:NN` de este ADR valen para el original (275 l.), no para `-r2` (491 l.) (r2 L208).
3. **Orden y N en r2** (L69 y L232): «ADR-292 → ADR-293 → ADR-291», con cifras de N, omite ADR-294. El orden vigente es 293 → 294 → 291 → S2 (Ingeniero, 2026-10-04) y N lo recalcula el Ingeniero al GO: no se cita aquí.
4. **r2 L2549 → L2550.** r2 §3.1 cita L2549 como una de las llamadas a `lower_method_call_other`; la llamada está en L2550 (L2549 es `Rule::method_call_other => {`). Menor.
5. **Tamaño de los diffs.** r2 §3 (L100–L101) da 188/245 líneas; r2 R8 (L49–L50) da 189/269 tras R3. Prevalece R8.
6. **`RwLock` (Q7).** La fuente de r2 con `fn f(a: RwLock)` falla por E0006 de forma de parámetro, no por RwLock; r2 la sustituye por `let r: RwLock = 0` (L15). No cambia Q7.
7. **ADR-293 no es rustfmt-clean** (r2 L46): el parche `adr293_hir_final.diff` deja 13 líneas `imports: vec![],  // ADR-293 test literal fix` con dos espacios antes de `//` (verificado hoy en el diff) y hace fallar `cargo fmt --check`; r2 aporta `/tmp/arita_adr291/adr293_fmtfix.diff` (sha256 `9f7e0d07324dd2dae9d8bbb47fbee6d31deed567d5d2317fea795c5e4fa4a50c`). Afecta a la IMPL de ADR-293; este ADR no lo edita.
8. **r2 L139** habla de «~20 literales de test» de ADR-293; ADR-293 §3 y su barrido dan 13 literales de test.
9. **Mensaje con sufijo.** El texto exacto de E0346 queda como prefijo; el stderr real añade ` @a..b`. El oráculo de Measure debe comparar por prefijo o por el span calculado (r2 L204).

## 3. Código E0346

**D5 — elección.** Se propone E0346 porque `rg` en `crates/`, `DOC/` y `ejemplos/` no encuentra E0346–E0360 (`DOC/reviews/PREP_MUTEX_PARSER_20261003.md:212–231`). **(corregido, 2026-10-04)** E0347 queda asignado a ADR-293 (llamada a función no declarada); E0346 sigue libre y es el código de este ADR.

- E0312 no se reutiliza: ADR-229 L20–24 lo define como `Mutex not available in this profile/surface` y lo liga a `neg-e0312-mutex`; su checklist sigue abierto en L46–48. Reutilizarlo mantendría una contradicción cuando Mutex tenga un perfil futuro.
- E0313 tampoco se reutiliza: ADR-229 L32–35 lo reserva para hold-across-await, y el PREP documenta que E0313 ya está usado por timeout/delay/cancel (`PREP_MUTEX_PARSER_20261003.md:227–231`).
- E0340–E0345 están usados o reservados; E0333–E0339 están reservados por ADR-290 (`DOC/ADR/290-core-index-mut-v0.md:9–10, 46–50`).

Mensaje EN canónico propuesto: `mutex concurrency is not available in this surface`. No se mezcla con E0242: E0242 diagnostica un préstamo vivo que cruza `await`, mientras E0346 rechaza la superficie antes de que exista un guard.

## 4. Punto mínimo de rechazo

**Recomendación del Arquitecto (v0.2, Q1 resuelta: elección del Parser, sin ampliar lo que acepta):** todas las formas se rechazan en **`arita-syntax`** (fase parse), con el mismo E0346 anclado al identificador (§2.5). **(corregido)** El texto anterior mandaba las formas `Named`/`MethodCall` a HIR; r2 demuestra por código que HIR no puede anclar el span.

1. Lowering de syntax: `Mutex`/`Arc` como tipo sin `<>`; `Mutex.new`/`Arc.new` (ancla: el receptor); `.lock()` de cualquier receptor y `host.lock()` (ancla: el selector `lock`).
2. Camino de fallo de pest (`classify_pest_error`): `Mutex<...>`/`Arc<...>` y `.lock().x`/`*x.lock()`, por escaneo léxico de la línea del fallo, antes del E0006 genérico.
3. HIR y Codegen no cambian: con el rechazo en parse no se emite Rust; un `Named("Mutex")` en un parámetro no llega a HIR (r2 L145).

Alternativa no recomendada para v0: dejar `Named` opacos y medir solo un caso E0346. Eso permitiría que otras formas pasaran hasta E0206/E0100 y no cumpliría el rechazo cerrado de D2.

## 5. Oráculos propuestos

**k = 4 FIJO** (Ingeniero, 2026-10-04). **N lo recalcula el Ingeniero al GO**; no se declara medida en este ADR.

- `neg-core10-mutex-type`: forma de tipo `Mutex<Int>` en `let`; expected E0346, exit 1, stdout vacío, span del identificador. Fuente nueva propuesta `ejemplos/core10/mutex-reject/01-type.arita` (sha256-16 `aa3b508fcd13f0bc`, `@60..65`).
- `neg-core10-mutex-arc`: forma `Arc<Mutex<Int>>` en un parámetro; expected E0346, exit 1, stdout vacío, span del primer identificador reservado. Fuente nueva propuesta `ejemplos/core10/mutex-reject/02-arc.arita` (`878609be7249096e`, `@36..39`).
- `neg-core10-mutex-constructor`: `Mutex.new(0)`; expected E0346 en el lowering de syntax (fase parse), antes de E0206. Fuente nueva propuesta `ejemplos/core10/mutex-reject/03-constructor.arita` (`b04c9256e356625a`, `@66..71`).
- `neg-core10-mutex-lock`: `n.lock()` con cualquier receptor; expected E0346 en el selector `lock` (fase parse), antes de E0206. Fuente nueva propuesta `ejemplos/core10/mutex-reject/04-lock.arita` (`497e04cbdda93a64`, `@85..89`).

Las cuatro fuentes son **nuevas** (PROPUESTA de r2 L157–L167; `ejemplos/core10/mutex-reject/` no existe hoy y no se crea en este ADR) y **no** el fixture congelado `mutex_new(0)`. Cada neg debe comprobar el prefijo exacto `E0346: mutex concurrency is not available in this surface` (el stderr real añade ` @a..b`, §2.5), exit 1, stdout vacío y ausencia de emisión. No se propone positivo funcional en v0. Ids, rutas y shas son propuesta hasta el freeze del Ingeniero.

**Migración del neg histórico:** la migración de `neg-core03-compose-mutex-hold` pertenece a **ADR-293 (E0347)**; este ADR no migra ningún neg existente. Los 4 oráculos de rechazo de arriba quedan.

## 6. GO, orden y medición

- ADR-290 y ADR-292 CLOSED son condición previa cumplida (`DOC/ADR/290-core-index-mut-v0.md:3–4, 37–39`, `DOC/GATE-CORE10-INDEX-MUT-20261003.md`, `DOC/GATE-CORE10-INT-ARITH-RUNTIME-20261003.md`); ADR-293 y ADR-294 van antes (orden 293 → 294 → 291 → S2, Ingeniero 2026-10-04).
- GO IMPL queda bloqueado hasta GO explícito del Ingeniero.
- Existen el PREP Parser (viabilidad estática) y el borrador de IMPL r2, validado en copia (§2.5; no medida de GATE); cualquier PREP Measure futuro debe cerrar sin BLOCKER antes de GO. La IMPL del Parser aplica los diffs finales de r2 (r2 L53) cuando ADR-293 esté aplicado.
- Orden propuesto: Parser/lowering → HIR → tests/Measure. Codegen no cambia si el rechazo queda antes de emisión.
- El fixture congelado y los freezes cerrados no se tocan en este DRAFT.

## 7. Fuera de alcance y backlog

- `RwLock`: fuera; no aparece tratado en el PREP.
- Mutex funcional, `Arc<Mutex<T>>` compartido, guards, `unlock`, `thread_local`, locks async y guard que cruza `await`: v1.1/futuros.
- **B-291-1 (P2, propuesta):** Mutex real, `Arc<Mutex<T>>`, guard-RAII, poisoning y política de sharing para v1.1.
- **B-286-7 (P1, existente):** llamada a función inexistente tipada como `Int`; **vive en ADR-293 (E0347)** (`DOC/ADR/286-core-0.10-errores-fase1.md:258`); este ADR no la cierra (D6).
- **B-286-12 (P3, existente):** firmas con guard-RAII y `thread_local` (`DOC/ADR/286-core-0.10-errores-fase1.md:263`). La integración guard/borrow con E0242 queda abierta y no se fija en v0.

## 8. Preguntas del Ingeniero — RESUELTAS (2026-10-04)

- **Q1 — RESUELTA:** la interceptación de `Mutex<T>`/`Arc<...>` (parser antes del E0006 o capa de lowering con lookahead) la elige el Parser, **sin ampliar lo que el parser acepta**.
- **Q2 — RESUELTA:** `Mutex.new` y `Arc.new` entran **DENTRO** del conjunto rechazado (E0346).
- **Q3 — RESUELTA:** `.lock()` se rechaza **léxicamente para cualquier receptor**; el selector `lock` queda **reservado** (una futura API no-Mutex no podrá usar `.lock()` sin un ADR que lo libere). Queda documentado aquí.
- **Q4 — RESUELTA:** el span exacto de `MethodCall` lo documenta el Parser, **sin reabrir B-286-11**.
- **Q5 — RESUELTA:** **k = 4 fijo**; N lo recalcula el Ingeniero al GO.
- **Q6 — RESUELTA:** este ADR **no tiene migración propia**; la del neg histórico pertenece a ADR-293 (E0347).
- **Q7 — RESUELTA:** `RwLock` queda **fuera**.
- Sin preguntas abiertas.

## 9. Changelog

- 2026-10-03 — v0 DRAFT PINS (Arquitecto): propuesta Mutex-reject; E0346 libre; alcance cerrado Mutex/Arc/lock; k provisional 4 (N provisional retirada en v0.2); migración del neg core03 (retirada en v0.2: pasa a ADR-293); sin GO IMPL, sin cambios de código ni de Codegen.
- 2026-10-04 — v0.2 DRAFT PINS (Arquitecto): decisiones del Ingeniero: el neg `neg-core03-compose-mutex-hold` (`mutex_new(0)`) no contiene Mutex/Arc/`.lock()` y **no migra a E0346**: migra a E0347 por ADR-293 (llamada a función no declarada); retirada la «Migración única» y toda cifra de N (N lo recalcula el Ingeniero al GO); k = 4 fijo; D6 NO (B-286-7 vive en ADR-293); Q1–Q7 RESUELTAS (Q1 elección del Parser sin ampliar lo que acepta; Q2 `Mutex.new`/`Arc.new` dentro; Q3 `.lock()` léxico para cualquier receptor, selector reservado; Q4 span documentado por el Parser sin reabrir B-286-11; Q5 k = 4; Q6 sin migración propia; Q7 RwLock fuera); orden 293 → 294 → 291 → S2; v0.1 pedido tras PREP de B-286-7, superado por ADR-293. Sin GO IMPL, sin cambios de código.
- 2026-10-04 — v0.2 (ampliación, Arquitecto): **esta v0.2 equivale a la v0.1 sin migración que pidió el Orquestador tras el PREP de B-286-7.** Incorporada la referencia de IMPL del Parser `ADR-291-PARSER-IMPL-DRAFT-20261004-r2.md` (§2.5): intercepción en `arita-syntax` (no HIR), span por `Pair`, fuentes nuevas y spans de los 4 oráculos (propuesta), E0346 gana a E0347 por fase, hallazgos r2 vs código vs ADR (9); citas corregidas (`arita.pest` L276–277, HIR `Await` L881–907). Sin cargo propio, sin ROADMAP, sin sellos; sin GO IMPL.

## Sello del Ingeniero (2026-10-04)

Sellado sobre la versión v0.2 con sha256 `774267483b798c0a5fe7f7615927bae31c25503a448d07adf37c5494f15ade15` (168 líneas). Esta sección se añade al final; nada por encima se ha modificado.

**Decisión: v0.2 ACEPTADA como contrato de pins para ADR-291.** El GO IMPL se da por escrito tras ADR-294 CLOSED (cumplido: GATE `DOC/GATE-CORE10-KNOWN-INT-SCOPE-20261004.md`, N=878).

1. **Alcance confirmado.** E0346 `mutex concurrency is not available in this surface` en `arita-syntax` (fase parse), sin cambios en HIR de producción, Codegen ni `arita.pest`; superficie cerrada de D2 (`Mutex`, `Arc`, `.lock()` de cualquier receptor, con el selector `lock` reservado). `RwLock` fuera. Sin migración propia: la de `neg-core03-compose-mutex-hold` ya se cerró con ADR-293 (E0347).
2. **Oráculos k = 4 y N.** Los cuatro negs de §5 con fuentes nuevas en `ejemplos/core10/mutex-reject/`. Cifra de la cola: 878 → **882**, y S2 suma k(S2) = 7 después (889). Los ids y rutas de §5 quedan aceptados; las fuentes exactas, y por tanto los spans `@a..b`, los fija la corrida real, no esta propuesta.
3. **Comparación del diagnóstico.** El oráculo exige el prefijo exacto `E0346: mutex concurrency is not available in this surface`, único código E, exit 1, stdout vacío, sin Rust emitido y un control que construye con 0 diagnósticos. El sufijo ` @a..b` del stderr es obligatorio y debe ser un span no vacío dentro de la fuente; no se fija en el id.
4. **Precedencia.** E0346 (parse) gana a E0347 (HIR) en ambos órdenes, medido por el Parser sobre la cadena 293 → 294 → 291. Un test unitario debe fijarlo.
5. **Riesgo del selector reservado.** El barrido de 867 `.arita` no cambia con 291 (0 cambios por 291); Measure debe confirmar en su PREP que ningún oráculo, fixture ni ejemplo existente usa `.lock(` ni los identificadores `Mutex`/`Arc`. Cualquier coincidencia es BLOCKER.
6. **Parches.** Solo valen los diffs finales del Parser con sha256 (`adr291_parser_final.diff` `9c8fc38b…` y `adr291_tests_final.diff` `f5e3117d…`), aplicados al árbol real por el Ingeniero tras el GATE de 294; `cargo fmt --check` y clippy `-D warnings` deben pasar en la corrida. Cero tests existentes modificados.
7. **Cierre.** Mismo gate que 293/294: fmt 0, clippy 0, build release, `cargo test --workspace -- --test-threads=1`, measure N/N sin skip, Veyra ACCEPTED, GATE propio `DOC/GATE-CORE10-MUTEX-REJECT-<fecha>.md`, freeze y addendum nuevos escritos por mí. Ningún freeze ni sello anterior se reescribe; este ADR no se edita tras el Sello.
8. **Backlog.** B-291-1 (P2, Mutex real en v1.1). El Mutex funcional queda fuera de v1.
