# ADR-289 — Core emit unused-parens v0: sin paréntesis redundantes en el Rust emitido (B-283-1 + `capacity()` + criterio «cero warnings de rustc en los positivos»)

- **Estado:** **CLOSED (Ingeniero, GO CLOSED 2026-10-03; DOC/GATE-CORE10-EMIT-UNUSED-PARENS-20261003.md)** · `arita measure` 856/856 accepted (N = 856, k = 3) · historia: v0.1 APROBADO Y CONGELADO (Ingeniero 2026-10-02, sha `f10574bfe7085de81ddc47ce7bb391ff8c0ca7996370bc28cd3d299fcbe6fe9f`), Addendum 1 (§11, 2026-10-03), GO IMPL tras ADR-288 CLOSED; freeze `DOC/reviews/MEASURE-ADR289-EMIT-UNUSED-PARENS-FREEZE-20261003.sha` y addendum de freeze `DOC/reviews/MEASURE-ADR289-EMIT-UNUSED-PARENS-FREEZE-20261003.addendum.sha`.
- **CUT-ID:** `CORE-EMIT-UNUSED-PARENS-20261001` (provisional; lo confirma el Orquestador)
- **Fecha:** 2026-10-01
- **Autores:** ARITA Arquitecto (pins) · alcance y orden decididos por el Ingeniero 01-10 (S1b → PKG-MEMBER 287 → EMIT-CLIPPY-B282 288 → **este** → IndexMut → Mutex); pedido original de <person> (unused_parens en el Rust emitido)
- **Padre / contexto:** [ADR-282](282-core-map-assign-v0.md) (B-282-2, `strip_one_outer_parens`) · [ADR-288](288-core-emit-clippy-b282-v0.md) (B-282 medido; deja B-283-1 aquí) · [ADR-285](285-core-ref-mut-v0.md) y [ADR-286](286-core-0.10-errores-fase1.md) (freezes con `ejemplos/core09/ref-mut/evidence.json`) · inventario estático de Codegen `DOC/reviews/PREP_UNUSED_PARENS_INVENTARIO_20261001.md` (md5 `7d9aca74b16d4d37e11fc8edb124a183`, verificado al redactar; en adelante **INV**)
- **Cierra:** **B-283-1** · caso `let c: i64 = (v.capacity() as i64);` de `ejemplos/f2/184-shrink-to-fit-vec.arita` · criterio «cero warnings de rustc en el Rust emitido de los positivos» (alcance acotado a cero `unused_parens`, §11) (CLOSED 2026-10-03).
- **Códigos:** **ninguno** (warnings del Rust emitido, no diagnósticos).
- **No reabre:** ADR-282/283/284/285/286/287/288 · E0291 / E0340–E0343 · ADR-265 / 270 / 278 / 283. Los GATE 283/284/285 no se reescriben (addendum propio, §7).

## 1. Hechos (de INV; estático, sin conteo real todavía)

INV es un inventario **estático**: no hay conteo de warnings de rustc/clippy hasta que el PREP dinámico (§6) lo produzca. Hoy el número de positivos que avisan es **desconocido**, no cero. Sobre `crates/arita-codegen/src/lib.rs` (md5 `57fc2a6c…` en INV):

- **(a) Binary:** un productor, `({l} {o} {r})` (L2204), y **23 sitios de consumo** donde el resultado cae directo en un contexto que lintea (let init, asignación, cond `if`/`while`, scrutinee `match`/`if let`/`while let`, brazo y cola de bloque, args de `Ok`/`Err`/`Some` y de `emit_user_call`, y 6 `let __x = {a};` de `swap`/`get`/`swap_remove`/`remove`/`is_char_boundary`/`is_multiple_of`).
- **(b) Borrow:** `(*&mut x)` / `(*&x)`, 2 productores (L2210/L2212), mismos contextos.
- **(c) Casts parentizados:** 11 productores `(x.f() as i64)` (L3275–3565: `count_ones`…`trailing_ones`, `abs_diff`, `len`, `capacity`, `floor/ceil_char_boundary`).
- **(d) Argumentos directos:** 57 sitios (`41` de un argumento, `5` de dos, `11` del puente `arita_host_http`) que lintean solo si el argumento ya viene parentizado; los 15 de la forma `&{a}` **no** lintean.
- **(e) Macros:** `println!("{}", {e})` (L2147) y `assert_eq!({l}, {r})` (L1830): **NO VERIFICADO** si rustc linta (lectura estática: probablemente no, `format_args!` y el `match (&(a), &(b))` de `assert_eq!`).
- **12 sitios que NO se tocan** por precedencia (INV §2: `resize/try_reserve*/reserve/shrink_to(({a}).max(0) as usize)`, `checked_pow/shl/shr`, `wrapping_shl/shr` con `({a}) as u32`, `floor/ceil_char_boundary`), más los `({ms} as u64)` (L2156/2377/2392/2404) y `{n} as usize` (L2923).
- Ya arreglado y que se conserva: `strip_one_outer_parens` en `let n: Int = x.len()` (L1820) y en `IndexAssign` (Vec L1889–1890, Map L1900–1901).
- Test de codegen con cadena esperada que **cambiaría**: `lib.rs` L3772, L4026 (solo si cambia `println!`, ver D4), L3835, L3936, L3937, L4121. Siguen valiendo: L4744, L4614, L5187–5196.
- `measure.rs`: **no existe** hoy un oráculo que recorra los positivos con rustc/clippy `-D warnings`; el único emit-clippy es `core09-map-assign-emit-clippy` (≈L9705–9813), y `run_ejemplo_oracle` (≈L4637–4660) compila y ejecuta sin `-D warnings`. Universo: 398 `EJEMPLO_ORACLES` positivos (538 `.arita` fuera de `*/neg/*`, 295 en `*/neg/*`).
- **Punto crítico:** `ejemplos/core09/ref-mut/evidence.json` declara hashes del Rust **emitido** (`lib_rs_sha256`, `main_rs_sha256`, `edge_lib_rs_sha256`, `edge_main_rs_sha256`) que el oráculo `core09-ref-mut-evidence` recompone (`measure.rs` ≈L11563–11682); `evidence.json` está hasheado en `MEASURE-ADR285-FREEZE-20260927.sha` L381 y `MEASURE-ADR286-FREEZE-20260927.sha` L383. Ese ejemplo emite `println!("{}", (m.len() as i64))` (`bin/main.arita:26`, `edge/bin/main.arita:26`).

## 2. Decisiones (pins)

**D1 — Mecanismo: contexto de emisión.** Se introduce `ExprCtx { Operand, Bare, Head }` y `emit_expr_ctx(e, ctx)`. `emit_expr(e)` queda como alias con `Operand` (comportamiento actual, byte a byte). Solo `Binary` (L2204), `Borrow` (L2210/2212) y los 11 casts de (c) cambian: con `Operand` emiten como hoy (con paréntesis); con `Bare` o `Head` emiten **sin el par exterior**. Los hijos de un `Binary` se emiten **siempre** con `Operand`. Cada sitio de consumo se migra **explícitamente**; un sitio no migrado no cambia. Se descarta como mecanismo general el parche por cadenas `strip_one_outer_parens` en cada consumo (frágil: olvida los guards de D3); `strip_one_outer_parens` y sus tests se conservan.

**D2 — Alcance de la migración.** `Bare` en: let init, asignación, args de función/método (familias (a) y (d)), `Ok/Err/Some(..)`, args de `emit_user_call`, `let __x = {a};` (6), brazo de `match`, cola de bloque y de fn. `Head` en: cond de `if`/`while` y scrutinee de `match`/`if let`/`while let` (los 3 caminos: stmt, `Expr::Match`, `emit_match_stmt_as_expr`). **No se migran:** los 12 sitios de precedencia de INV §2, los `({ms} as u64)` y `{n} as usize`, y los 15 `&{a}`.

**D3 — Guards obligatorios (sin ellos no hay GO IMPL).**
1. Un `Binary`/cast que sea **operando** (cualquier lado) de otro `Binary`, `Cast` o receptor de método se emite siempre con `Operand` (con paréntesis).
2. Un cast que sea operando izquierdo de `<` o `<<` **conserva** el paréntesis (`(v.len() as i64) < 3`; sin él no parsea).
3. `Head` **conserva** el paréntesis si la expresión contiene un struct literal exterior (`contains_exterior_struct_lit` en rustc); también se conserva ante cualquier construcción que rustc no considera linteable.
4. Dobles paréntesis `((x + 1)).max(0)` son resultado de los 12 sitios no migrados; no son objetivo.
5. El operando de un **unario** (`-(a + b)`, `!(a && b)`), el operando izquierdo de **`as`** (`(a + b) as i64`) y el de **indexación/rango** (`(a + b)[i]`, `(a + b)..c`) **conservan** el paréntesis (`Operand`).
6. Valores de `return (..)` / `break (..)`: el PREP dinámico decide si rustc los linta; si lintean pasan a `Bare`, si no se quedan como están.
Cada guard tiene su fixture (§5, PU-2) y su test cargo (§4). El guard 5 tiene además un test cargo propio para el unario.

**D4 — `println!` / `assert_eq!` (familia (e)) y los hashes de ref-mut.** Resuelto por el Ingeniero (Q1): manda el criterio de <person> (cero warnings de `unused_parens` en el emit) sobre la comodidad del freeze; **no se saca nada del alcance por evitar un re-freeze, solo por no lintear.** (a) Si el PREP dinámico muestra que rustc **no** linta los argumentos de `println!`/`assert_eq!`, se quedan **`Operand`** y los cuatro hashes de `ejemplos/core09/ref-mut/evidence.json` deben quedar **idénticos**, con `core09-ref-mut-evidence` verde **sin modificar `evidence.json`** (criterio de cierre PU-4). (b) Si rustc **sí** los linta, o si cambia cualquier hash de ref-mut por otra migración, se **migran** y se **re-congela `evidence.json` con addendum en el freeze NUEVO de este slice**, registrando el sha antiguo y el nuevo; los freezes 285/286 (`MEASURE-ADR285-FREEZE-20260927.sha` L381, `MEASURE-ADR286-FREEZE-20260927.sha` L383) **no se reescriben**. No hay BLOCKER pendiente del Ingeniero para este punto.

**D5 — `capacity()`.** El cast de `capacity` (L3539) se migra como los demás de (c); `ejemplos/f2/184-shrink-to-fit-vec.arita` pasa a emitir `let c: i64 = v.capacity() as i64;`. `std-shrink-to-fit-vec` (measure.rs ≈L1916) debe seguir aceptado con la misma salida.

**D6 — Sin cambio semántico.** El programa observable (stdout, exit) de todos los positivos es idéntico antes y después. Cualquier diferencia de stdout en un oráculo existente es BLOCKER. Ninguna firma pública del crate cambia salvo la interna `emit_expr_ctx`.

## 3. Criterio y mecanismo de medición

Criterio del Ingeniero: «cero warnings de rustc en el Rust emitido de los positivos, sin cambiar semántica». Pins (Q2 resuelto): se miden `-D unused_parens -D unused_braces` sobre **todos** los positivos; el alcance se amplía a `-D warnings` completo **solo si** el PREP dinámico muestra el corpus limpio. Los otros lints que aparezcan se registran como **B-289-n (P2**: son warnings reales en código emitido) y **no bloquean** este slice. Mecanismo, por clase de positivo (el PREP dinámico debe **clasificar cada positivo** en una de las dos): (i) **`rustc`** sobre el `.rs` emitido (edición 2015 sin `--edition`, como `arita build`, y 2021 para el camino Cargo), `--emit=metadata`, para los positivos de un solo fichero sin dependencias; (ii) **`cargo check`** en un crate scratch con `RUSTFLAGS="-D unused_parens -D unused_braces"` y target dir compartido, para los positivos que emiten un crate Cargo con dependencias (async/tokio, `[deps]`, host-bridges, puente http), que `rustc` suelto no compila. **Ningún positivo se excluye en silencio:** si no se puede medir ⇒ **inconclusive y cuenta como no accepted**; falta de `rustc`/`cargo` ⇒ inconclusive, nunca accepted; skip ≠ PASS.

## 4. Tests cargo (codegen, `crates/arita-codegen/src/lib.rs`; nombres y cuenta los fija IMPL)

Se migran (se cambia el string esperado, se conserva el nombre): L3835 `let sum: i64 = 1 + 2;` · L3936 `while i < 2 {` · L3937 `i = i + 1;` · L4121 `a + b` como cola de fn. L3772 y L4026 (`println!`) **no cambian** si D4(a). Se añaden: un test por guard de D3 (cast izquierdo de `<`; struct literal bajo `Head`; hijo de `Binary`; operando de unario, de `as` y de indexación); un test de cada sitio no migrado de INV §2 (la cadena emitida no cambia); un test de `capacity` (D5). Siguen intactos L4744, L4614, L5187–5196 y los dos `b282_emit_clippy`. Criterio de cierre: `cargo test --workspace` verde, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, miri-workspace verde si estaba verde en el CLOSED previo.

## 5. Oráculos `measure` (propuestos; N exacto lo fija el Ingeniero)

- **PU-1 (verde, corpus):** para **cada** positivo de `EJEMPLO_ORACLES` (398 hoy), el Rust emitido se mide según su clase de §3 (`rustc` o `cargo check`) con `-D unused_parens -D unused_braces`, en las dos ediciones donde aplique; **N positivos revisados == longitud actual de la tabla** (el oráculo falla si la tabla crece y no se recorre). Cubre además los positivos que **no** están en `EJEMPLO_ORACLES` pero sí en `ejemplos/` (538 `.arita` fuera de `*/neg/*` frente a 398 oráculos): el PREP dinámico dice cuántos son y por qué no tienen oráculo; los que no tienen stdout esperado **no se miden como oráculo pero se listan**. Un solo oráculo por defecto (Q3 resuelto); se parte solo si el tiempo **medido** en el PREP supera **10 min en Lex**; si se parte, el invariante «N revisados == longitud de la tabla» se comprueba sobre la **suma de las partes** y k se recalcula al GO IMPL.
- **PU-2 (verde, guards y no-regresión de precedencia):** fixtures nuevos mínimos en `ejemplos/core10/unused-parens/`: cast a la izquierda de `<`, struct literal bajo `if`/`match`, `.max(0) as usize` (los 12 sitios no migrados), `checked_pow(.. as u32)`, operando de unario (`-(a + b)`, `!(a && b)`), operando izquierdo de `as` y de indexación; cada uno **compila, no avisa y imprime el stdout exacto** (los ids y la salida los fija IMPL con los programas del test cargo correspondiente).
- **PU-3 (verde, `capacity`):** `ejemplos/f2/184-shrink-to-fit-vec.arita` emitido contiene `let c: i64 = v.capacity() as i64;`, sin aviso, misma salida que hoy.
- **PU-4 (regresión):** `core09-ref-mut-evidence` sigue aceptado **sin cambios en `evidence.json`** (D4(a)); no es un oráculo nuevo, es criterio de cierre explícito.

k provisional = 3 (PU-1…PU-3). **N no se fija aquí:** N = N_CLOSED previo + k, al GO IMPL. Provisional con el orden vigente: 850/851 (PKG-MEMBER) → 853 (288) → **856** (este). Reglas: skip ≠ PASS; ningún oráculo cuenta si rustc no se ejecutó. Los oráculos que son corpus no deben modificar el repo real (manifiesto sha256 antes/después).

## 6. PREP (bloquea GO IMPL)

1. **PREP dinámico de Codegen** (INV §5, con Lex libre y tras S1b CLOSED): para cada positivo, `rustc`/`cargo check` con `-W unused_parens -W unused_braces` (y `-W warnings` para contar otros lints) sobre el emit real, en las dos ediciones; **clasificar cada positivo en `rustc` o `cargo check`**; tabla positivo → clase → lint → contexto → línea; agregada por familia (a)–(e). Debe resolver: ¿lintea `println!`/`assert_eq!`? ¿lintean `return (..)`/`break (..)` (D3.6)? ¿cuántos positivos avisan hoy? ¿hay warnings de otros lints (alcance de §3, B-289-n)? ¿cuántos de los 538 `.arita` no tienen oráculo y por qué?
2. **PREP de Measure:** que ningún oráculo existente verifique la cadena de paréntesis del Rust emitido (INV: 0 aserciones en `measure.rs`/`tests/`, por patrones), y qué oráculos recomponen hashes de Rust emitido (hoy solo `core09-ref-mut-evidence`; no hay otro `evidence.json` con esas claves).
3. Medir el tiempo de PU-1 sobre el corpus real antes de fijarlo como oráculo; si pasa de lo razonable para el gate, se parte (Q3). No se estima aquí.

## 7. Cierre y GATE

Al CLOSED: addendum propio `DOC/GATE-CORE10-EMIT-UNUSED-PARENS-<fecha>.md` que marca B-283-1 y `capacity()` arreglados; GATE-283/284/285 y REF-MUT/VEC-ASSIGN/SCENARIO-MUT no se reescriben. Si se re-congela `evidence.json` (D4 b), el addendum lo registra con el sha antiguo y el nuevo. El addendum lista además la consecuencia para los tests migrados (L3835, L3936, L3937, L4121 y los que el PREP añada).

## 8. Preguntas del borrador — resueltas por el Ingeniero (01-10)

- **Q1 (D4):** manda el criterio de <person>; si `println!`/`assert_eq!` lintean o cambia un hash de ref-mut: se migran y se re-congela `evidence.json` con addendum en el freeze nuevo de este slice; si no lintean, `Operand` y hashes idénticos.
- **Q2 (§3):** `-D unused_parens -D unused_braces`; `-D warnings` completo solo si el PREP muestra el corpus limpio; los otros lints, B-289-n (P2), no bloquean.
- **Q3 (PU-1):** oráculo único; se parte solo si el tiempo medido supera 10 min en Lex.

## 9. Slice y GO

Slice único `EMIT-UNUSED-PARENS` (tamaño M: cambia el emit de codegen, ~100 sitios, 6 tests migrados, 3 oráculos). Secuencia: DOC (este ADR) → verificación del Ingeniero → PREP dinámico + PREP Measure (§6) → respuesta Q1–Q3 → **GO IMPL** del Ingeniero con ADR-288 CLOSED → implementación (Codegen/Orquestador) → Lex → CLOSED con evidencia real. Prohibido inventar PASS / N/N / CLOSED / conteos.

## 10. Changelog

- 2026-10-01 — v0 DRAFT PINS (Arquitecto). Basado en INV (estático, md5 `7d9aca74…`); sin conteo dinámico ni ejecución propia; el mecanismo y los guards son diseño, no resultado medido.
- 2026-10-01 — v0.1 DRAFT PINS (Arquitecto): resueltas Q1–Q3 del Ingeniero; PREP/clases `rustc`/`cargo check` (§3, §6); guards 5 y 6 en D3; PU-1 cubre positivos sin oráculo y fija umbral de 10 min; §7 con tests migrados. Pendiente de aprobación del Ingeniero.
- 2026-10-03 — Addendum 1 (Arquitecto, decisiones del Ingeniero tras el PREP dinámico de Codegen): §11; criterio acotado a cero unused_parens; B-289-1 P3; alcance dentro/fuera; sin reescribir el texto vigente.
- 2026-10-03 — Nota: la línea de changelog v0.1 (2026-10-01) que dice «Pendiente de aprobación del Ingeniero» queda superada: el ADR está APROBADO Y CONGELADO (Estado, L3; Ingeniero 2026-10-02).
- 2026-10-03 — CLOSED (GATE `DOC/GATE-CORE10-EMIT-UNUSED-PARENS-20261003.md`); cierre en §12.

## 11. Addendum 1 (2026-10-03, decisiones del Ingeniero tras el PREP dinámico)

No reescribe el texto vigente (L1–L88); donde contradiga, prevalece este addendum solo en lo indicado.

- **Base:** PREP dinámico de Codegen `DOC/reviews/PREP_ADR289_UNUSED_PARENS_DYNAMIC_20261002.md` (sha256 `fa1cd82865970ff5f6386edfbaafb54739772a3f130a8518a49fb0ff97490863`, md5 `27d2d5826ddd9c890e698679d19322d9`, 161 líneas). Propuesta de partida: `DOC/reviews/ADR289-ADDENDUM-PROPOSAL-20261002.md` (sha256 `1c44c1ef2fcbbe2f45481ad51e09ba067a10846a4b937181afa0032bd3390fbe`).
- **Criterio acotado:** «cero warnings de rustc» (título, «Cierra» L8, §3 L50) queda **ACOTADO a cero `unused_parens`** en el Rust emitido de los positivos medidos; **no** se pretende cero warnings de rustc. Los otros lints (`unused_variables`, `dead_code`, `unused_mut`; 35 positivos con otros lints, PREP L47) pasan a **B-289-1 (P3)**. Datos del PREP: 1791 avisos `unused_parens` en 93 positivos en 2021 (L23); 278 positivos a 0 hoy + 85 solo con `unused_parens` = **363/398 a cero tras el arreglo** (derivado de L47).
- **Alcance DENTRO:** `Binary` en let/asignación/`if`/`while`/cola de fn; casts; `Borrow`; y además argumentos de función, de método, `Ok(..)`, scrutinee de `match` y `return`, con regresión mínima sobre los casos mínimos del PREP (L100 `fn_arg_binary`, L105 `match_scrutinee`, L106–L107 `method_arg_*`, L108 `ok_wrapper`, L113 `return_binary`). **FUERA:** argumentos de macro (`println!`, `assert_eq!`, `vec!`, …), que no lintean (PREP L117). Este addendum prevalece sobre D2 (L31, que tenía esos contextos en `Bare`) en lo que contradiga; los guards 1–6 de D3 (L33–L40) siguen vigentes sobre cuándo un paréntesis NO es redundante.
- **Evidence:** no se re-congela `ejemplos/core09/ref-mut/evidence.json`; si el emisor cambia el Rust de una evidencia congelada, Codegen lo reporta **antes** y se resuelve con addendum (los hashes `main_rs_sha256` / `edge_main_rs_sha256` dependen de los `println!` de ref-mut, PREP L122–L132). PU-4 sigue siendo la regresión de `core09-ref-mut-evidence`.
- **k/N:** k provisional = 3 (PU-1..PU-3) + regresión mínima de los contextos añadidos (PU-4 sigue siendo regresión); N provisional **856** (N de ADR-288 CLOSED = 853, +3); el Ingeniero recalcula al GO IMPL.
- **Ediciones medidas** (propuesta del Arquitecto, sin objeción hasta ahora): los 398 positivos en 2021; 2015 donde compile (366; los 32 con `async`/`.await` de `cargo` no son medibles en 2015, PREP L22–L25). Corrige el desajuste con §3 (L50), donde 2015 se medía en los positivos de `rustc` y 2021 solo en el camino Cargo, mientras el PREP mide los 398 en 2021.
- **GO IMPL:** la condición (1) del Estado (ADR-288 CLOSED) se cumple el 2026-10-03; siguen (2) PREP de Measure sin BLOCKER y (3) GO IMPL del Ingeniero. (El PREP dinámico de Codegen, parte de (2), ya está entregado: `PREP_ADR289_UNUSED_PARENS_DYNAMIC_20261002.md`.)
- **Cierre:** GATE propio `DOC/GATE-CORE10-EMIT-UNUSED-PARENS-<fecha>.md` (sin cambio respecto a §7).

## 12. Cierre (CLOSED 2026-10-03)

Datos tomados de `DOC/GATE-CORE10-EMIT-UNUSED-PARENS-20261003.md` (sha256 `1f00e51a26d27c3c1c8c00903c540fc0c5f87ba6c28c27c851b585d6c3a2a7f0`).

- **Veredicto:** GO CLOSED ADR-289 (Ingeniero Rust, 2026-10-03). Medida final **856/856 accepted**, 0 skip; k = 3 (PU-1..PU-3), N previo 853 → N = 856.
- **Oráculos nuevos:** `core10-unused-parens-corpus` (PU-1: 398 positivos del corpus; 363 de clase `rustc` en ediciones 2015 y 2021 y 35 de clase `cargo` en 2021; 0 `unused_parens`) · `core10-unused-parens-guards` (PU-2: 2 fixtures de `ejemplos/core10/unused-parens/`, build ok, stdout exacto, formas fijadas en el emit, `rustc -W unused_parens` limpio en 2015 y 2021) · `core10-unused-parens-capacity` (PU-3: `ejemplos/f2/184-shrink-to-fit-vec.arita`, emit `let c: i64 = v.capacity() as i64;`). **PU-4 (regresión):** `core09-ref-mut-evidence` existente, accepted, `evidence.json` intacto; **sin id nuevo**. Los 853 previos sin cambios frente a `MEASURE_ADR288_EMIT_CLIPPY_EXCLUSIVE_20261003.json`.
- **Evidencia** (run exclusivo en Lex, 2026-10-03 05:47 → 09:00, `/tmp/ing-adr289-final`): `cargo fmt --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` 0; `cargo build --release` 0; `cargo test --workspace -- --test-threads=1` 520 passed / 0 failed (el test de `arita-cli` que lanza miri real tarda ~4500 s); `arita measure` exit 0 (termina 08:16), stdout JSON puro, 856 ids únicos, 856 accepted, 0 skip; Miri dentro de measure y de Veyra (verde).
- **Veyra Proof** (`--profile quick`): **ACCEPTED** `20261003T061643Z` (rustfmt, cargo check, clippy, cargo test, cargo-audit, veyra-anti-theater; 0 problemas, 0 skipped). El GATE no cita ningún run REJECTED previo. Herramientas: rustc/cargo 1.97.1, clippy 0.1.97; `~/.cargo/bin` (clippy-driver) en el PATH.
- **Alcance medido:** `unused_parens` 1791 → 0 (2015/2021 y workspaces); 93 de 398 positivos cambian el Rust emitido, solo paréntesis.
- **Artefacto de medición:** `DOC/reviews/MEASURE_ADR289_UNUSED_PARENS_EXCLUSIVE_20261003.json` (sha256 `2252d84bcfd1…`). **Freeze:** `DOC/reviews/MEASURE-ADR289-EMIT-UNUSED-PARENS-FREEZE-20261003.sha` (1004 líneas: 5 de cabecera + 999 verificables con `shasum -a 256 -c`; sha256 `85ce0df5f33d…`) y su addendum `DOC/reviews/MEASURE-ADR289-EMIT-UNUSED-PARENS-FREEZE-20261003.addendum.sha` (el GATE lo cita sin ruta; la ruta es la del fichero existente).
- **Shas de código:** `crates/arita-codegen/src/lib.rs` `9e654825…` → `c9b3e7d6ee38…`; `crates/arita-cli/src/measure.rs` `159a07ad…` → `4a04ab1e0920…`; `main.rs` `ed0820b4…` y `package.rs` `5fee2beb…` sin cambios. Fixtures: `01-contexts` `2d162426…`, `02-precedence-sites` `8cf56a98…`. ADR-288 `0a962e12…` y ADR-289 `6fda6661…` (antes de este edit de cierre).
- **Limitaciones declaradas:** otros lints fuera de alcance (B-289-1, P3). Los 15 workspaces suplementarios no tienen oráculo propio (B-289-2, P3); los 11 que cambian los verificó Codegen (cargo build 11/11; clippy idéntico salvo la desaparición de `unused_parens`; 10 conservan lints preexistentes).

### Decisiones del Ingeniero incorporadas al cierre (03-10, vía Orquestador; ver también §11)

- Ids de oráculos `core10-unused-parens-*` (`-corpus`, `-guards`, `-capacity`).
- k = 3 y N = 856 definitivos; PU-4 = `core09-ref-mut-evidence` existente, sin id.
- Los contextos nuevos (argumentos de función/método/`Ok(..)`/`match`/`return`) se cubren como **unit tests de Codegen**, no como oráculos (el GATE no menciona estos unit tests; sí el addendum del freeze: au1..au10, 77 tests).
- PU-2 reformulado en consecuencia: fixtures de `ejemplos/core10/unused-parens/` (`01-contexts`, `02-precedence-sites`) para los contextos medidos por oráculo; contextos nuevos por unit tests.
- Ediciones medidas: 2015 donde `arita build` la usa (363 positivos) y 2021 en los 35 de `cargo`; (corregido, según el GATE: PU-1 mide además los 363 de clase `rustc` en 2021, es decir 363 en 2015 y 2021 y 35 en 2021).
- Los 15 workspaces suplementarios → **B-289-2 (P3)**.
- PU-1 tolerado hasta 15 min (decisión del Ingeniero; el GATE no registra el tiempo de PU-1).

### Backlog abierto

- **B-289-1 (P3):** otros lints de rustc en positivos (`unused_variables`, `dead_code`, `unused_mut`; 35 positivos según el PREP, L47).
- **B-289-2 (P3):** los 15 workspaces suplementarios sin oráculo propio (16 `unused_parens` en 2021 según el PREP, L152; tras el cierre 0 según el GATE).

### Orden siguiente

ADR-290 (IndexMut) → Mutex. El GATE indica para ADR-290 un PREP de Codegen en solo lectura (overflow Int, helper read+set) y GO IMPL cuando el PREP cierre Q4/D5. El export se regenera con N = 856 cuando lo pida el Ingeniero.

Nota: sha del ADR tras este cierre: lo registra el addendum del Ingeniero.
