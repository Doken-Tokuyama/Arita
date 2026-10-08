# ADR-290 — IndexMut seguro: asignación compuesta indexada (core 0.10, slice A)

- **Estado:** **CLOSED (Ingeniero, GO CLOSED 2026-10-03; DOC/GATE-CORE10-INDEX-MUT-20261003.md)** · `arita measure` 862/862 accepted (N = 862, k = 6) · historia: v0.3 DRAFT PINS (Arquitecto 2026-10-03, sha `7ea657e65343aff8766b33378a98ef832f3cda55c495287a5d3b3db9f941900f`); v0.3 revisada y APROBADA Y CONGELADA por el Ingeniero el 2026-10-03 (línea «Sello», pins 1–3 aceptados; GO IMPL con k = 6, N = 862, orden Parser → HIR → Codegen, Codegen condicionado a P7); antes, v0.2 (sha `5524d902166c718fd109f44af6e27d5a0b5ec686021f449c450e47eac09fa847`) aprobada con cambios sobre v0.1 (sha `926483b4…`); freeze `DOC/reviews/MEASURE-ADR290-INDEX-MUT-FREEZE-20261003.sha` y addendum de freeze `DOC/reviews/MEASURE-ADR290-INDEX-MUT-FREEZE-20261003.addendum.sha`.
- **Sello:** **v0.3 APROBADO Y CONGELADO (Ingeniero, 2026-10-03; sha previo al sello `7ea657e65343aff8766b33378a98ef832f3cda55c495287a5d3b3db9f941900f`) · GO IMPL con k = 6, N = 862, orden Parser → HIR → Codegen.** El GO de Parser y HIR es inmediato; el paso de Codegen exige antes la medición ligera P7 (`i64::checked_add/sub/mul` como puntero a fn sin lint de rustc/clippy en 2015 y 2021; si avisa, alternativa B del PREP §3.3 con aviso previo). Condiciones de IMPL: (a) P10 (literal máximo de i64 y `known_int` sin plegar a E0217) se verifica ANTES de diseñar la fixture de overflow de IM-2; si no se puede escribir el límite, IM-2 usa `*=` con operandos grandes, sin cambiar semántica; (b) sin E0333 por operador; (c) el emisor de E0006 para operador no soportado es el Parser; HIR solo para compuesto sobre Map/String con el texto exacto actual; (d) ningún cambio a la expresión binaria `a + b` (ADR-045): se registra como B-290-2 (P2); (e) migración IM-6 solo con freeze nuevo + addendum + fixture nuevo, sin reescribir 05-compound ni freezes cerrados; (f) k y N se re-miden al CUT, sin hacer ADR nuevo si IMPL propone un oráculo extra: se reporta al Ingeniero antes.
- **CUT-ID:** `CORE-0.10-INDEX-MUT-20261002`
- **Fecha:** 2026-10-02
- **Autores:** ARITA Arquitecto (pins) · orden de cola decidido por el Ingeniero (287 → 288 → 289 → **este** → Mutex)
- **Padre / contexto:** [ADR-261](261-core-index-sugar-v0.md) · [ADR-266](266-core-map-index-v0.md) · [ADR-282](282-core-map-assign-v0.md) · [ADR-283](283-core-vec-assign-v0.md) · [ADR-284](284-core-scenario-mut-v0.md) · [ADR-285](285-core-ref-mut-v0.md) · [ADR-289](289-core-emit-unused-parens-v0.md) (criterio de emit limpio)
- **Cierra:** el caso compuesto `v[i] op= x` sobre `Vec<Int>`/`List<Int>` (slice A) (CLOSED 2026-10-03). Map compuesto, anidado y campo quedan fuera (OUT / Q1); B-290-1 sigue abierto.
- **Códigos:** **E0333 nuevo** (D2; solo tipo de elemento no soportado; un operador compuesto no soportado ⇒ E0006). E0334–E0339 libres, reservados a slices B/C de este ADR.
- **No reabre:** E0291 · E0340–E0343 · ADR-265 / 270 / 278 / 283.

## Contexto

- ADR-261/266: lectura `v[i]`/`m[k]` = `get` → `Option`; IndexMut quedó OUT.
- ADR-282 (`m[k] = v`, Map<Text|String,Int>, ≡ put), ADR-283 (`v[i] = x`, Vec/List, ≡ `v.set(i,x)?`, solo en `fn -> Result<_, Int>`), ADR-284 (scenario-mut), ADR-285 (ref-mut): CLOSED. Codegen emite helpers (`__arita_vec_set(&mut v,i,x)?`, `m.insert(k,v)`), nunca Rust `IndexMut`/`Index`; los emit-bans exigen 0 IndexMut/Index en el Rust emitido.
- Hoy siguen en E0006 (fuera de gramática F1.1): compuesto `v[0] += 1` (ejemplos/core09/vec-assign/neg/05-compound.arita:7; map-assign/neg/04-compound.arita:8), anidado `v[0][1] = 1` (vec-assign/neg/06-nested.arita), campo `v[0].f = 1` (vec-assign/neg/07-field.arita; TRAPS T09-13). Test HIR: crates/arita-hir/src/lib.rs ~L8682-8687.
- Hecho verificado: la superficie NO tiene asignación compuesta simple (`x += 1`): assign_stmt en arita.pest L216 (solo `=`); index_assign_stmt en L218; regla `assign_stmt = { ident ~ "=" ~ assign_rhs }`, HIR `Assign {name, value}`. Por tanto `v[i] += x` no hereda base alguna: este ADR añade el operador compuesto SOLO sobre places indexados.

## Decisión (slice A)

D1. Gramática: `index_assign_stmt` admite además `op_assign` ∈ {`+=`, `-=`, `*=`} (mismo conjunto de operandos de ident y literal/ident como índice que ADR-283). `x += 1` simple sigue fuera (E0006) → B-290-1.

D2. Alcance de tipos v0: Vec<Int>/List<Int> únicamente. Otros tipos de elemento ⇒ E0333 (nuevo: «compound index-assign: unsupported element type in v0»; **solo** por TIPO de elemento no soportado, es decir no Int). Un OPERADOR compuesto no soportado (`/=`, `%=`, …) ⇒ **E0006** (como hoy: no entra en la gramática de D1, falla en parse), nunca E0333.

D3. Semántica Vec: `v[i] op= x` ≡ leer el elemento en i y `v.set(i, cur op x)?`. Índice fuera de rango o i<0 ⇒ el mismo Err que `set` (se propaga con `?`); solo en `fn -> Result<_, Int>` (fuera: E0344, igual que ADR-283). Literal negativo ⇒ E0319 (igual que ADR-283).

D4. Orden de evaluación: (1) índice, (2) rhs (izquierda→derecha), (3) lectura de cur, (4) `cur op x`, (5) escritura. rhs se evalúa antes de tomar `&mut v`; como `v[j]` es `Option` (ADR-261), un rhs `v[j]` es E0203 y `v.get(j)` no es Int: no hay aliasing con el préstamo mutable.

D5. **CERRADA — D5-ii (Arquitecto, 2026-10-03; Q4 cerrada con el PREP de Codegen `DOC/reviews/PREP_ADR290_CODEGEN_20261003.md`).** El compuesto `v[i] op= x` (`+=`, `-=`, `*=` sobre Vec<Int>/List<Int>) se emite con `checked_add`/`checked_sub`/`checked_mul`; `None` (overflow) ⇒ `Err` propagado con `?`, **igual en debug y release** (los `checked_*` no dependen de `overflow-checks`; PREP §2.2) y **sin panic nuevo**. Índice negativo (no literal), fuera de rango y overflow ⇒ `Err` con `v` **intacto** (la escritura es el último paso, D4.5); solo en `fn -> Result<_, Int>` (D3). Orden de fallo: fuera de rango antes que overflow (hay que leer `cur` para operar). **Declaración explícita:** `v[i] += x` es **MÁS ESTRICTO** que la expresión `a + b` de Int, que hoy emite `(a + b)` sobre `i64` sin checked (panic en debug, wrap en release; PREP §1.4 y §2.1, ADR-045). La expresión binaria **NO cambia** en este slice y este ADR no arregla el agujero de ADR-045; si el rhs `x` es a su vez `a + b`, esa suma interna conserva panic/wrap (D5 cubre solo la operación compuesta, no «toda la sentencia»). **Pin 1 (valor del Err de overflow):** `Err(0)`, el mismo valor que el OOB de `Vec.set` (`DOC/ADR/283-core-vec-assign-v0.md:23` «early-return `Err(0)`» y `:175` «`Err(0): Int` es SoT (270)»; PREP §2.3). **Declarado:** la causa (OOB vs overflow) **no se distingue en el valor del Err**; distinguirla exigiría un ADR aparte.

D6. Reglas de borrow reutilizadas sin cambio: binding no `mut` ⇒ E0202; movido ⇒ E0201; préstamo vivo/compartido ⇒ E0202; tipo del rhs ≠ Int ⇒ E0203; `x[i] op= y` con x Map/String/otra colección ⇒ E0006, como hoy (mismo código, emitido por HIR al rechazar el receptor; ejemplos/core09/map-assign/neg/04-compound sigue E0006); E0314 solo para ident no-colección (p. ej. Int). HIR debe reproducir el texto **exacto** `E0006: construct outside F1.1 (parse failure)` (mismo formato y span que hoy; test HIR `crates/arita-hir/src/lib.rs:8687`) para el compuesto sobre Map, porque con la gramática de D1 el parser ya no lo rechaza; el oráculo `neg-core09-map-assign-compound` debe seguir verde (PREP §4.2.3, P5).

D7. Emit: sin IndexMut/Index/AddAssign sobre índice en el Rust emitido; helper de prelude `__arita_vec_update(v: &mut [i64], i: i64, x: i64, op: fn(i64, i64) -> Option<i64>) -> Result<(), i64>` (nombre y forma: PREP §3.1, alternativa A: `usize::try_from(i)`, una sola búsqueda `get_mut`, `op(*slot, x)`, escritura al final), volcado solo si el módulo usa algún compuesto sobre Vec (como `__arita_vec_set`). Sitio de llamada: `{ let __arita_vi = <i>; let __arita_vx = <x>; __arita_vec_update(&mut <v>, __arita_vi, __arita_vx, i64::checked_add)?; }` (`checked_sub`/`checked_mul` para `-=`/`*=`). Orden de evaluación índice → rhs → lectura → operación → escritura, sin doble evaluación (D4; PREP §3.2). Mismos emit-bans que 282/283 (0 IndexMut/Index; sin `+`/`-`/`*` entre `slot` y `x`, ni `unwrap`/`expect`/`panic!`/`as usize`). La forma con `i64::checked_add` como puntero a fn **debe medirse** (rustc + clippy en 2015 y 2021) antes del GO IMPL (PREP §3.2, P7); si avisa, se vuelve al Arquitecto (alternativa B del PREP §3.3). Código emitido pasa `-D unused_parens -D unused_braces` y clippy según el criterio de ADR-289.

D8. Map NO entra en slice A. Slice B (ADR futuro/ampliación): `m[k] op= y` exige contexto `fn -> Result` y miss de clave ⇒ Err propagado (opción b); NUNCA identidad 0 silenciosa (Q1 RESUELTA, Ingeniero 02-10).

## OUT / HOLD

- `v[i][j] = x`, `v[i].campo = x`, `m[k].push(x)`, `&mut v[i]`, `&mut m[k]` como argumento: siguen E0006/HOLD (necesitan borrow de subelemento + ADR + GO propios).
- String.set (indexa por chars), Mutex, D1 (params `&mut Map/Vec` + receptor loan): HOLD.
- E0314 split: HOLD (backlog ROADMAP L67-68).
- B-290-1 (P3): `x += 1` simple (asignación compuesta sobre ident), tras el PREP de Mutex.
- Slice B: Map compound (Q1 b).

## Códigos

E0333 nuevo (D2; **solo** tipo de elemento no soportado, p. ej. Vec<Text>/Vec<Bool>). Un operador compuesto no soportado ⇒ E0006 (D2), no E0333. E0334–E0339 libres, reservados a slices B/C de este ADR. No se reabren E0291, E0340–E0343 ni ADR-265/270/278/283.

Significado de los códigos usados (según ADR/HIR reales): **E0344** `index assign outside result fn` (contexto; `DOC/ADR/283-core-vec-assign-v0.md:14` y `:161`; `crates/arita-hir/src/lib.rs:597`) · **E0319** `negative set index` (índice literal negativo; `DOC/ADR/270-lang-contract-set.md:25`; HIR `lib.rs:564`) · **E0203** `type mismatch` (tipo del rhs, o `E` de la fn ≠ Int; ADR-283 `:25`; HIR `lib.rs:531`) · **E0006** `construct outside F1.1 (parse failure)` (ADR-283 `:35`; HIR `lib.rs:8687`) · **E0333** (nuevo en este ADR; no existe aún en HIR ni en otro ADR).

**Pin 3 — precedencia dentro de la misma sentencia `v[i] op= x`:** E0344 > E0319 > E0333 > E0203. Es la regla de ADR-283 (E0344 > E0319 > E0203, `:28`) con E0333 insertado entre E0319 y E0203; **no choca** con «gana el primer ofensor en orden fuente», que sigue rigiendo **entre** sentencias (ADR-283 `:28` y `:207`; la precedencia por código solo rige dentro de una misma sentencia).

## Oráculos propuestos (k provisional = 6)

- IM-1 `v[i] += x` / `-=` / `*=` sobre Vec<Int> en `fn -> Result`: resultado correcto (pos).
- IM-2 índice OOB, negativo y **overflow** (`+=`/`-=`/`*=` en los límites de i64) en runtime ⇒ `Err(0)` propagado, `v` intacto, sin panic, igual en debug y release (D5-ii) (pos).
- IM-3 emit-ban: 0 IndexMut/Index/AddAssign-index en el Rust emitido + `rustc`/`cargo check` limpio.
- IM-4 neg E0333: Vec<Text> `+=` en `fn -> Result` (tipo de elemento no soportado); un operador no soportado (`/=`, `%=`, …) ⇒ E0006, no E0333 (D2).
- IM-5 neg: no-mut ⇒ E0202; rhs Text ⇒ E0203; main ⇒ E0344; literal negativo ⇒ E0319; `x[i] op= y` con x Map/String/otra colección ⇒ E0006, como hoy (mismo código, emitido por HIR al rechazar el receptor; ejemplos/core09/map-assign/neg/04-compound sigue E0006); E0314 solo para ident no-colección (p. ej. Int). Precedencia E0344 > E0319 > E0333 > E0203 con un neg por par (Pin 3).
- IM-6 migración: `vec-assign/neg/05-compound` pasa de **negativo (E0006) a POSITIVO** (es `Vec<Int>` en `fn -> Result<Int, Int>`: `ejemplos/core09/vec-assign/neg/05-compound.arita:4-8`) ⇒ **freeze nuevo + addendum + fixture nuevo** (p. ej. un `05-compound-ok.arita`; el nombre lo fija IMPL; 05 no se reescribe y se retira de `neg-core09-vec-assign-compound` solo con addendum del Ingeniero; PREP P8); neg 06-nested y 07-field siguen E0006 sin cambio. La migración de ejemplos/core09/vec-assign/neg/05-compound.arita cambia la sha de un fixture congelado: se hace con NUEVO freeze + addendum (DOC/reviews/…addendum.sha); nunca se reescribe el freeze existente. Idem map-assign/neg/04-compound si cambia (no cambia en slice A: sigue E0006).

N provisional = N_CLOSED previo + k, lo calcula el Ingeniero en GO IMPL (orden de cola: 287 → 288 → 289 → 290 → Mutex). Provisional: 856 (ADR-289 CLOSED) + k = 6 ⇒ **862**; lo recalcula el Ingeniero.

## Preguntas abiertas (Ingeniero)

Q1. **RESUELTA (Ingeniero 02-10):** Map NO entra en slice A. Slice B (ADR futuro/ampliación): `m[k] op= y` exige contexto `fn -> Result` y miss de clave ⇒ Err propagado (opción b); NUNCA identidad 0 silenciosa. Pregunta original: Map compound `m[k] += x`: put no devuelve Result, así que un miss de clave debe ser total sin `?`. Opciones: (a) miss ⇒ identidad (0) solo para `+=`/`-=`, `*=` ⇒ E0333; (b) exigir contexto Result y Err en miss; (c) mantener E0006. Recomendación del Arquitecto: slice A solo Vec, Map en slice B tras decisión.

Q2. **RESUELTA (Ingeniero 02-10):** Int-only en v0. Pregunta original: Admitir Float/otros numéricos en slice B o mantener Int-only.

Q3. **RESUELTA (Ingeniero 02-10):** `x += 1` simple = B-290-1 (P3), tras el PREP de Mutex, no antes. Pregunta original: ¿`x += 1` simple como ADR aparte (B-290-1) antes o después?

Q5. **RESUELTA (Ingeniero 02-10):** oráculos separados IM-1..IM-6 (k=6), ninguno >10 min en Lex. Pregunta original: IM-1..IM-6 en un oráculo o varios (criterio ≤10 min en Lex por run, como ADR-289).

## Proceso

DOC pins → verificación Ingeniero → GO IMPL. La condición «tras ADR-289 CLOSED» está **CUMPLIDA** (ADR-289 CLOSED 2026-10-03). **Sigue SIN GO IMPL** (lo da el Ingeniero tras verificar v0.3 y la medición de P7). Orden de implementación: **Parser → HIR → Codegen** (el AST gana `op`; el codegen no es testeable sin el campo; PREP §4.1, P6). N provisional 862 (856 + k = 6), lo recalcula el Ingeniero. No editar ROADMAP.

## Changelog

- 2026-10-03 — v0.3 DRAFT PINS (Arquitecto): Q4/D5 cerrada → D5-ii (`checked_*`, `None` ⇒ `Err`, igual en debug y release; `v[i] op= x` más estricto que `a + b`, que no cambia); Pin 1: `Err(0)` único, causa no distinguida; Pin 2: operador no soportado ⇒ E0006, E0333 solo por tipo de elemento (D2, Códigos, IM-4); Pin 3: precedencia E0344 > E0319 > E0333 > E0203; D6: HIR reproduce el texto exacto de E0006 para Map; D7: helper `__arita_vec_update`; IM-2/IM-4/IM-5/IM-6 ajustados (05-compound pasa a positivo: freeze nuevo + addendum + fixture nuevo); orden Parser → HIR → Codegen; condición «tras ADR-289 CLOSED» cumplida; N provisional 862; sin GO IMPL. Base: PREP de Codegen `DOC/reviews/PREP_ADR290_CODEGEN_20261003.md` (sha256 `25d0783112f8e8f68ba4019eba943914045cb65b736ec28a597e59309e842127`). (El fichero no tenía sección de changelog previa.)

- 2026-10-03 — Sello (Ingeniero): v0.3 revisada y APROBADA; GO IMPL con k = 6, N = 862, orden Parser → HIR → Codegen, Codegen condicionado a P7; pins 1–3 aceptados tal cual; B-290-2 (P2): semántica de `a + b` sobre Int (panic en debug, wrap en release) a pinear antes de v1.
- 2026-10-03 — CLOSED (GATE `DOC/GATE-CORE10-INDEX-MUT-20261003.md`); cierre en la sección «Cierre» (final del fichero; las secciones de este ADR no están numeradas).

## Cierre (CLOSED 2026-10-03)

Datos tomados de `DOC/GATE-CORE10-INDEX-MUT-20261003.md` (sha256 `2a653cff898e18d191d4b79b611207849432b1eaff617c435c523d5db5d4cc7d`).

- **Veredicto:** GO CLOSED ADR-290 slice A (Ingeniero Rust, 2026-10-03). Medida final **862/862 accepted**, 0 skip; k = 6 (IM-1..IM-6), N previo 856 → N = 862. CUT-ID del GATE `CORE-0.10-INDEX-MUT-20261002` (el mismo del ADR; sin cambio).
- **Oráculos** (ids decididos por el Ingeniero): `core10-index-mut-compound-ok` (IM-1: `+=`, `-=`, `*=` sobre Vec<Int> en `fn -> Result`, build ok y stdout exacto) · `core10-index-mut-err-propagation` (IM-2: índice fuera de rango, negativo y overflow de `+=`/`-=`/`*=` ⇒ `Err(0)` propagado, `v` intacto, sin panic, en debug y release) · `core10-index-mut-emit-ban` (IM-3: helper `__arita_vec_update` + llamadas `i64::checked_*`; 0 IndexMut/Index/*Assign/unwrap/expect/panic!; `unused_parens` y clippy `-D warnings` limpios en 2015 y 2021) · `core10-index-mut-neg-unsupported` (IM-4: `v[0] /= 2`, `v[0] %= 2` y `Vec<Text>` anotado ⇒ **E0006 exacto**) · `core10-index-mut-neg-rules` (IM-5: 10 negs con su código exacto: E0202, E0203, E0344, E0319, E0006 Map/String, E0314 Int, y los pares de precedencia alcanzables E0344>E0319, E0344>E0203, E0319>E0203) · `core10-index-mut-migrated-05` (IM-6).
- **Evidencia** (run exclusivo en Lex, 2026-10-03 10:09 → 12:30, `/tmp/ing-adr290-final`): `cargo fmt --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` 0; `cargo build --release` 0; `cargo test --workspace -- --test-threads=1` 562 passed / 0 failed; `arita measure` exit 0 (termina 11:45), stdout JSON puro, 862 ids únicos, 862 accepted, 0 skip, los 856 previos sin cambios frente a `MEASURE_ADR289_UNUSED_PARENS_EXCLUSIVE_20261003.json` y los 6 nuevos accepted; Miri dentro de measure y de Veyra (verde).
- **Veyra Proof** (`--profile quick`): **ACCEPTED** `20261003T094540Z` (rustfmt, cargo check, clippy, cargo test, cargo-audit, veyra-anti-theater; 0 problemas, 0 skipped). El GATE no cita ningún run REJECTED previo. Herramientas: rustc/cargo 1.97.1, clippy 0.1.97; `~/.cargo/bin` (clippy-driver) en el PATH.
- **Artefacto de medición:** `DOC/reviews/MEASURE_ADR290_INDEX_MUT_EXCLUSIVE_20261003.json` (sha256 `97ae00c8776f…`). **Freeze:** `DOC/reviews/MEASURE-ADR290-INDEX-MUT-FREEZE-20261003.sha` (1029 líneas: 6 de cabecera + 1023 verificables con `shasum -a 256 -c`; sha256 `2eae386b83b0…`) y su addendum `DOC/reviews/MEASURE-ADR290-INDEX-MUT-FREEZE-20261003.addendum.sha` (el GATE lo cita sin ruta; la ruta es la del fichero existente).
- **Shas de código** (idénticas antes y después del run): `crates/arita-codegen/src/lib.rs` `c9b3e7d6…` → `e8df9366dc74…`; `crates/arita-hir/src/lib.rs` `7983cecc…` → `d0571e12419d…`; `crates/arita-syntax/src/lib.rs` `16b4191e…` → `187cff0e4bb0…`; `crates/arita-syntax/src/arita.pest` `569b15d1…` → `7c28aa3c627a…`; `crates/arita-cli/src/measure.rs` `4a04ab1e…` → `c4dc13a73339…`; `main.rs` `ed0820b4…` y `package.rs` `5fee2beb…` sin cambios. ADR-290 `c3adf806…` (v0.3 sellado, antes de este edit de cierre).

### Decisiones del Ingeniero incorporadas al cierre (03-10; ver también D5–D6, Códigos e IM-4..IM-6)

- **D5-ii:** `v[i] op= x` usa `checked_add`/`checked_sub`/`checked_mul`; overflow ⇒ `Err(0)`, igual en debug y release. **Es más estricto que `a + b`**, que no cambia en este slice (ADR-045).
- **Err(0):** el mismo valor que el OOB de `Vec.set`; la causa (OOB vs overflow) no se distingue en el valor del Err.
- **D6:** compuesto sobre Map/String/Bytes ⇒ **E0006**, con el prefijo del ADR + ` @start..end` (lo emite HIR); el resto de no-colecciones ⇒ **E0314** (`n[0] += 1` pasa de E0006 a E0314). Un operador no soportado (`/=`, `%=`) ⇒ E0006 emitido por el Parser.
- **IM-4 (prevalece sobre el texto de v0.3, que hablaba de un neg E0333):** IM-4 mide E0006 exacto; **E0333 no es alcanzable desde fuente** (la gramática es Int-only y la búsqueda del Parser no ofrece vía real a un Vec/List no Int; `Vec<Text>` falla en el parser con E0006), queda como defensa en profundidad probada solo por tests HIR, sin fingir un neg E0333. Por la misma razón los pares de precedencia con E0333 de Pin 3 no son alcanzables desde fuente; la precedencia E0344 > E0319 > E0333 > E0203 sigue vigente (GATE, «Decisiones y semántica»).
- **IM-6 / 05-compound:** `ejemplos/core09/vec-assign/neg/05-compound.arita` (congelado, sin modificar) pasa de E0006 a válido; se retira de `neg-core09-vec-assign-compound` (3 → 2 casos) mediante addendum del freeze; positivo nuevo `core10/index-mut/05-compound-ok.arita` (existe en `ejemplos/core10/index-mut/`). `ejemplos/core09/vec-assign/evidence.json` (neg_compound: E0006) queda documentalmente obsoleto; no se reescribe.

### Backlog abierto

- **B-290-2 (P2):** `a + b` Int sin checked: hoy `(a + b)` sobre `i64` (panic en debug, wrap en release; ADR-045); debe pinearse antes de v1.
- **B-290-1 (P3):** `x += 1` simple. **B-289-1 / B-289-2 (P3)** (otros lints de rustc; los 15 workspaces suplementarios sin oráculo propio). Son los que declara el GATE («Abierto»).

### Orden siguiente

ADR-291 MUTEX-REJECT (Mutex real a v1.1). El GATE indica además que Docs alinea las cifras a N = 862.

Nota: sha del ADR tras este cierre: lo registra el addendum del Ingeniero.
