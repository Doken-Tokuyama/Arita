# ADR-287 — Core package-member v0: `arita build <input>` solo compila un `input` que es miembro del `[workspace]` (B-286-10)

- **Estado:** **CLOSED (Ingeniero, GO CLOSED 2026-10-02; DOC/GATE-CORE10-PKG-MEMBER-20261002.md)** · `arita measure` 851/851 accepted (N = 851, k = 7) · historia: PINS APROBADOS (Ingeniero 01-10), v0.3 `b80cec57b6b9c94626d9c61cf8a2f53758bdf853a16b842280a4938a288ba1ec` verificada, IMPL tras S1b CLOSED; freeze `DOC/reviews/MEASURE-ADR287-PKG-MEMBER-FREEZE-20261002.sha`.
- **CUT-ID:** `CORE-0.10-PKG-MEMBER` (el del GATE y del freeze; el de apertura fue `CORE-PKG-MEMBER-20261001`)
- **Fecha:** 2026-10-01
- **Autores:** ARITA Arquitecto (pins) · prioridad y orden decididos por el Ingeniero 01-10 (S1b → B-286-10 → unused_parens → IndexMut → Mutex)
- **Padre / contexto:** [ADR-255](255-core-package-manifest-v0.md) · comentario de `main.rs` (~L263, «ADR-263: resolve runnable crates/bin_* artifact») — **referencia a verificar**: `263-core-ref-coll-v0.md` es REF-COLL y no respalda la resolución del artefacto `bin_*` (errata del Ingeniero 01-10) · [ADR-286](286-core-0.10-errores-fase1.md) fila **B-286-10 (P1)** (hallazgo de Ingeniero/Codegen 27-09; datos en `crates/arita-cli/src/main.rs:256-283`)
- **Cierra:** **B-286-10** (CLOSED 2026-10-02)
- **Código:** **E0332** `input is not a workspace member` (nuevo, familia manifest/package `E033x`; ver §3).
- **No reabre:** E0330 / E0331 (semántica actual intacta) · ADR-255 · ADR-286 · E0291 / E0340–E0343 · ADR-265 / 270 / 278 / 283.

## 1. Problema (hechos, no hipótesis)

`arita build <input.arita>` hoy, si existe un `arita.toml` con `[workspace]` en alguno de los (hasta 8) directorios padre, **ignora `input`**:

- `main.rs:259`: `if let Some(_manifest) = package::load_package_manifest(arita_path)?` — `find_arita_toml` (`deps.rs:15-27`) sube como máximo 8 niveles; el manifiesto se descarta (`_manifest`) y no se comprueba que `input` sea miembro.
- `main.rs:260`: `build_package_workspace(arita_path, profile)` compila **todos** los miembros (`package.rs:280-290`) tras `remove_dir_all(target/arita-pkg)` (`package.rs:234-235`).
- `main.rs:266-277`: `bin_guess` = el **primer** `bin_*` que devuelve `read_dir` (orden no determinista), sin relación con `input`.
- `main.rs:278-281`: `return Ok(BuildOutput{ bin: bin_guess, .. })`. El camino que sí emitiría `input` (`main.rs:284` en adelante) nunca se alcanza.
- `main.rs:29-31`: imprime `ok: <bin>` y devuelve `SUCCESS`.

Repro confirmado por Codegen (27-09, binario `target/release/arita` sin instrumentar, copia limpia; repo real intacto):

| input (no miembro) | resultado actual | debería |
|---|---|---|
| `ejemplos/core04/lib-api/bin/main-record.arita` (module `demo_rec`) | exit 0, `ok: …/bin_demo`; ese binario imprime `3` | no compilar; (el fichero imprimiría `30`) |
| `ejemplos/core04/ref-pkg-lib/bin/edge.arita` (module `arita_ref_pkg_lib_edge`) | exit 0, `ok: …/bin_arita_ref_pkg_lib`; imprime `42` | no compilar; (el fichero imprimiría `edge`) |

**Consecuencia esperada tras IMPL (no regresión):** `arita build` de esos dos ficheros históricos pasará a dar `E0332` (exit 1, stdout vacío), porque no son miembros de su workspace. Es el comportamiento correcto buscado por este ADR; ningún oráculo los referencia (Codegen 27-09; PREP Measure §6).

Los dos manifiestos declaran `[workspace] members = ["lib","bin"]`; `member_arita_path` (`package.rs:204-211`) solo resuelve `<m>/{<m>,mod,main,lib}.arita` y `<m>.arita`, así que `bin/` → `bin/main.arita`. Ningún oráculo de `measure` referencia esos dos ficheros (Codegen), por lo que no hay falso verde; pero es un **binario equivocado reportado como éxito** (exit 0), por eso P1.

Efecto lateral adicional: el build erróneo borra `target/arita-pkg` del workspace (`package.rs:234-235`) aunque `input` no fuera miembro.

## 2. Decisiones (pins)

**D1 — Membresía.** Tras cargar el manifiesto (`load_package_manifest` devuelve `Some`), el CLI resuelve la ruta de **cada** miembro con `member_arita_path(pkg_root, m)` (semántica actual de candidatos, sin cambios) y compara con `input` por **ruta canónica** (`std::fs::canonicalize` de ambos lados; resuelve `.`, `..`, rutas absolutas/relativas y symlinks). `input` es miembro si y solo si su ruta canónica coincide con la de algún miembro. Un miembro cuyo `member_arita_path` falla (E0331 actual) sigue propagando ese E0331 (no se oculta). El workspace es el que `arita build` ya resuelve hoy (`find_arita_toml`, sin cambios); membresía = ruta canónica contra sus `members`. Este ADR **no fija** una regla de «manifiesto más cercano».

**D2 — `input` no miembro ⇒ error, nunca éxito.** Si `input` existe y no es miembro: `Err("E0332: input <input> is not a member of workspace <ruta del arita.toml> (members: <m1>, <m2>, …>)")`; stderr con ese mensaje, **stdout vacío**, **exit 1**. Se descarta compilarlo como fichero suelto (silenciaría el error de uso: quien pone un fichero bajo un workspace espera semántica de paquete). La comprobación ocurre **antes de cualquier mutación**: no `remove_dir_all`, no creación de `target/arita-pkg`, no `cargo`.

**D3 — `input` inexistente o ilegible bajo un workspace.** `canonicalize(input)` falla ⇒ se devuelve el mismo error de E/S que hoy produce el camino de fichero suelto (`fs::read_to_string`, sin código E; exit 1). Nunca se compila el workspace por un `input` que no existe. (Hoy sí se compilaría.)

**D4 — `input` miembro ⇒ el `ok:` nombra el artefacto de SU miembro.** Se compila el workspace como hoy, pero el `BuildOutput.bin` se selecciona por el crate emitido a partir del `.arita` del `input` (no por el primer `bin_*` de `read_dir`). Si el miembro es `bin` (define `main`), `ok:` apunta a su `bin_*`. Si el miembro es `lib` (sin `main`), no existe `bin_*` de ese miembro: `ok:` apunta al directorio `target/<perfil>/` (comportamiento de fallback actual, `main.rs:277`), **nunca** al `bin_*` de otro miembro. La ruta impresa tras `ok:` **no contiene ningún componente que empiece por `bin_`**. Con varios miembros bin, gana el del `input`. Selección determinista (no depende del orden de `read_dir`). La forma de implementarlo (devolver el nombre del crate desde `build_package_workspace` o derivarlo del módulo del `input`) la elige IMPL; el contrato es el de esta sección y se verifica por oráculo (§5).

**D5 — Alcance de la comprobación.** Solo el camino `target.is_none()` de `build_with_entry_profile_target` (el que hoy entra en el workspace). Los llamadores de `build_package_workspace` en `measure.rs` (≈30 sitios) pasan miembros por construcción y **no** cambian de firma ni de comportamiento; la comprobación vive en una función nueva de `package.rs` (nombre libre, p.ej. `resolve_input_member`) invocada desde `main.rs`. No se toca `find_arita_toml` (sigue lexical, 8 niveles; ver §7 límites conocidos). Cualquier otro llamador de `build_with_entry_profile*` (contract / attest; referencia ADR-263 del comentario de main.rs, a verificar) hereda la comprobación; el PREP de Measure (§6) debe listarlos.

**D6 — Sin cambios en `arita test`, `parse`, `--target`.** `arita test` y `parse` no cargan manifiesto (siguen igual). Con `--target <triple>` el camino de workspace se omite hoy (`main.rs:258`); queda tal cual y se anota como límite conocido (§7).

## 3. Código de error

Se evaluó reusar **E0331** (`package.rs:29,39,218,227…`: manifiesto ilegible/inválido, `members` vacío, miembro sin `.arita`). No encaja: aquí el manifiesto es **válido** y el defecto está en el **input** del usuario; mezclar ambos impediría a los oráculos distinguir "manifiesto roto" de "fichero equivocado". Por eso **E0332** nuevo: `input is not a workspace member`. Comprobado antes de redactar que `E0332` no aparece en ADR 280–286 ni en `crates/arita-cli/src` (comprobación sobre todo el repo hecha al crear este fichero). E0330 y E0331 no cambian. Alta del código en catálogos/DOC: la hace IMPL/Docs al CLOSED.

## 4. Tests cargo (arita-cli; nombres orientativos, cuenta y nombres exactos los fija IMPL)

Unit en `package.rs` (sin cargo/rustc), sobre un árbol temporal con `arita.toml` `[workspace] members = ["lib","bin"]`, `lib/lib.arita`, `bin/main.arita`, `bin/other.arita` (no miembro):

1. `member_input_direct_path` — `bin/main.arita` resuelve a miembro `bin`.
2. `member_input_dotdot_and_dot_forms` — `./bin/../bin/main.arita` y la forma absoluta resuelven al mismo miembro.
3. `member_input_symlink` (`#[cfg(unix)]`) — symlink a `bin/main.arita` resuelve a `bin`.
4. `nonmember_input_is_e0332` — `bin/other.arita` ⇒ `Err` que empieza por `E0332:` y contiene la ruta del manifiesto y los miembros.
5. `nonmember_check_has_no_side_effects` — antes de la llamada se crea `target/arita-pkg/marker`; tras el `Err` el marker sigue ahí (no hubo `remove_dir_all`).
6. `missing_input_is_io_error_not_e0332` — ruta inexistente ⇒ `Err` sin `E0332` ni build del workspace.
7. `lib_member_has_no_bin_artifact` — input = `lib/lib.arita` ⇒ miembro `lib`; el selector de artefacto devuelve el directorio de perfil `target/<perfil>/`, el `file_name` de la ruta que se imprimiría tras `ok:` no empieza por `bin_` y la ruta no contiene ningún `bin_*`.
8. `two_bin_members_pick_input_crate` — workspace con dos miembros con `main`; el selector devuelve el crate del input, para ambos inputs, en ambos órdenes de `members`.

Migración: ningún test existente debería cambiar; si el PREP (§6) encuentra alguno que construye un no miembro bajo un `[workspace]`, se lista como BLOCKER y se decide antes de GO IMPL (no se migra en silencio). Criterio de cierre adicional: `cargo test -p arita-cli` verde; `cargo fmt --check` y `cargo clippy -D warnings` sin regresión; miri-workspace verde si estaba verde en el CLOSED de S1b.

## 5. Oráculos `measure` (propuestos; N exacto lo fija el Ingeniero)

Fixture nuevo (lo crea IMPL, no este ADR): `ejemplos/core10/pkg-member/` con `arita.toml`, `lib/lib.arita`, `bin/main.arita` (imprime una línea conocida), `bin/other.arita` (no miembro; imprimiría otra línea distinta) y dos no miembros más que sustituyen a los repros históricos de core04: `bin/other-record.arita` (misma forma que main-record: record + impresión, módulo propio, salida distinta de `bin/main.arita`) y `bin/other-edge.arita` (misma forma que edge, salida distinta). Un oráculo "build debe fallar con Exxxx" **no** cabe en `NegOracle` (`measure.rs:2549-2556`, solo `parse_lower_check`); se implementa como función propia en `measure.rs` que invoca el flujo de build real y comprueba el resultado, registrada en `cmd_measure`.

- **PM-1 (verde):** build de `bin/main.arita` ⇒ `Ok`; el `file_name` del `bin` empieza por `bin_`; ejecutar ese binario produce **exactamente** la salida esperada de `main.arita` (prueba que no es el de otro miembro).
- **PM-2 (neg):** build de `bin/other.arita` ⇒ **exit 1, stdout vacío** y stderr con el código exacto `E0332` (mensaje que empieza por `E0332:`); ningún `ok:`. Además el sha256 del manifiesto de ficheros del fixture **no cambia** y, si `target/arita-pkg` existía antes, sigue idéntico (marker intacto).
- **PM-3 (verde):** build de `main.arita` por forma de ruta distinta (`./bin/../bin/main.arita`) ⇒ `Ok` y misma salida que PM-1.
- **PM-4 (neg):** build de `ejemplos/core10/pkg-member/bin/other-record.arita` (fixture NUEVO) ⇒ **exit 1, stdout vacío**, stderr con `E0332`; el manifiesto sha256 del **fixture** y `target/arita-pkg` (si existía) no cambian.
- **PM-5 (neg):** build de `ejemplos/core10/pkg-member/bin/other-edge.arita` (fixture NUEVO) ⇒ **exit 1, stdout vacío**, stderr con `E0332`; mismas comprobaciones de no-mutación que PM-4 (manifiesto sha256 del **fixture** y `target/arita-pkg` si existía).
- **PM-6 (neg):** `input` inexistente bajo el workspace ⇒ **exit 1, stdout vacío**, stderr de E/S **sin** `E0332`; `target/arita-pkg` (si existía) y el manifiesto sha256 no cambian.
- **PM-7 (verde, caso lib D4):** build de `lib/lib.arita` (miembro `lib`, sin `main`) ⇒ **exit 0**, stdout `ok: <directorio de perfil target/<perfil>/>`, y la ruta impresa **no contiene ningún componente `bin_*`**; stderr vacío. Es la forma exacta del bug original (input sin `bin_*` propio que acababa en el `bin_*` de otro miembro); el test cargo 7 solo prueba el selector, PM-7 prueba la CLI de punta a punta (decisión Ingeniero 01-10).

Los oráculos neg se ejecutan como proceso (`arita build`) o con una función que devuelva exit, stdout y stderr por separado; comprobar solo `Err(String)` no basta para el criterio de stdout vacío / exit 1.

k = 7 (PM-1…PM-7; PM-7 añadido por decisión del Ingeniero 01-10). **N no se fija en este ADR:** N es siempre N_CLOSED previo + k, calculado por el Ingeniero al GO IMPL de cada slice. Con el orden vigente S1b (844) → PKG-MEMBER → S2 (k = 7), los números **provisionales** son PKG-MEMBER = 844 + 7 = **851** y S2 = 851 + 7 = **858** (ADR-286 mantiene 851 solo mientras S2 vaya inmediatamente después de S1b; se recalcula al GO IMPL).

Reglas: skip ≠ PASS; ningún oráculo cuenta si usa el binario de otro miembro; los oráculos neg no pueden modificar el árbol del fixture (manifiesto sha256 antes/después). Los ficheros históricos `ejemplos/core04/lib-api/bin/main-record.arita` y `ejemplos/core04/ref-pkg-lib/bin/edge.arita` están **congelados** y se citan solo como repro histórico (§1); no son oráculo.

## 6. PREP de Measure (bloquea GO IMPL)

Escaneo de solo lectura, informe en `DOC/reviews/PREP_ADR287_PKG_MEMBER_SCAN_<fecha>.md` con md5/sha:

1. Todo llamador de `build_with_entry_profile`, `build_with_entry_profile_target` (main.rs, contract.rs, attest.rs, measure.rs, tests) y su `input` real.
2. Para cada uno: ¿hay `arita.toml` con `[workspace]` en los ≤8 niveles superiores? ¿el `input` es miembro (D1)? Cualquier `input` **no miembro bajo un workspace que hoy "pasa"** es **BLOCKER** (hoy pasaría por el binario equivocado): se lista con ruta y oráculo/test afectado.
3. Todos los `.arita` del repo bajo un `[workspace]` que **no** son miembro (inventario; los de ejemplo ya conocidos: los dos de §1).
4. Confirmar que ningún oráculo verde depende del `ok:` apuntando al primer `bin_*` de `read_dir`.

**Resultado del PREP (Measure, md5 `03882290`, comunicado por el Orquestador 01-10): 0 BLOCKERs.** Informe: `DOC/reviews/PREP_ADR287_PACKAGE_MEMBER_SCAN_20261001.md` (md5 verificado con `md5 -q`: `03882290bd02ae8ffd2f5fd914ebd588`, coincide).

## 7. Fuera de alcance / límites conocidos

- `find_arita_toml` no canonicaliza: un `input` relativo sin directorio (`main.arita`) no descubre manifiestos superiores y se compila como fichero suelto (emite `input`, no hay binario equivocado). Una ruta con más de 8 niveles bajo el manifiesto igual. **B-287-1 (P3)**, no bloquea.
- `--target <triple>` omite el camino de workspace (D6). **B-287-2 (P3)**.
- **B-287-3 (P3):** build de un `input` bajo un manifiesto anidado autónomo (un `arita.toml` más cercano que el que declara el workspace resuelto): hoy el workspace resuelto es el que devuelve `find_arita_toml`; hay 12 manifiestos anidados de este tipo en el repo (dato del Ingeniero 01-10). No se cambia aquí; se decide en un slice posterior.
- Reglas de resolución de `member_arita_path` (candidatos) y manifiestos con `members` de forma distinta: sin cambios.
- Mensaje con spans / formato rico de diagnóstico: no aplica (error de CLI, no de compilador).

## 8. Slice, orden y GO

- **Slice único** `PKG-MEMBER` (tamaño S). Orden decidido por el Ingeniero: S1b → **este** → unused_parens/B-282 → IndexMut → Mutex.
- Secuencia: DOC (este ADR) → PREP Measure (§6) → verificación del Ingeniero → **GO IMPL** del Ingeniero → implementación (Orquestador/Codegen) → Lex → CLOSED con evidencia real. Prohibido inventar PASS / N/N / CLOSED.
- Al CLOSED: edición de cierre de ADR-286 (fila B-286-10 → CLOSED por ADR-287) y alta de E0332 en catálogos, ambas por el Arquitecto/Docs.

## 9. Changelog

- 2026-10-01 — v0 DRAFT PINS (Arquitecto). Sin verificación del Ingeniero todavía; ningún dato de ejecución propio (los hechos de §1 son los reconfirmados por Codegen 27-09).
- 2026-10-01 — v0.1 (Arquitecto): errata del Ingeniero: referencia ADR-263/ADR-255 (§Padre, D5); N como fórmula N_CLOSED previo + k con 850/857 provisionales (§5); neg PM-2/4/5/6 con exit 1 + stdout vacío + stderr exacto + no-mutación (§5); D4 caso lib sin `bin_*` (D4, test 7, nota PM). Pins D1–D6 sin cambios.
- 2026-10-01 — v0.2 (Arquitecto): decisión del Ingeniero: PM-7 (caso lib, CLI de punta a punta) ⇒ k = 7; N provisional PKG-MEMBER 851, S2 858 (se recalculan al GO IMPL); Estado → PINS APROBADOS, IMPL bloqueado hasta S1b CLOSED y PREP Measure §6 sin BLOCKER. Pins D1–D6 sin cambios.
- 2026-10-01 — v0.3 (Arquitecto): decisiones del Ingeniero: D1 sin pin de «manifiesto más cercano» (workspace = el que resuelve `find_arita_toml`); B-287-3 (P3, 12 manifiestos anidados autónomos); PM-4/PM-5 con fixture NUEVO `ejemplos/core10/pkg-member/` (`other-record`, `other-edge`), repros core04 solo históricos y E0332 esperado como consecuencia; PREP Measure §6 = 0 BLOCKERs (md5 03882290). Pins D1–D6 sin otros cambios.

## 10. Cierre (CLOSED 2026-10-02)

Datos tomados de `DOC/GATE-CORE10-PKG-MEMBER-20261002.md` (sha256 `d617170232db85c3fe0547aca2ea4ce0fc4b48f2fb3b6a5ff5772a663ab8558a`).

- **Veredicto:** GO CLOSED (Ingeniero Rust, 2026-10-02).
- **E0332** `input is not a workspace member`: `arita build <input>` en un paquete solo acepta miembros del workspace; un no-miembro da E0332 (exit 1, stdout vacío, sin mutar el árbol). Una entrada inexistente sigue siendo error de I/O (PM-6), no E0332.
- **Oráculos** (k = 7, N = 851): PM-1 `core10-pkg-member-bin-ok` · PM-2 `neg-core10-pkg-member-other` · PM-3 `core10-pkg-member-dotdot` (incluye la ruta relativa simple) · PM-4 `neg-core10-pkg-member-other-record` · PM-5 `neg-core10-pkg-member-other-edge` · PM-6 `neg-core10-pkg-member-missing-input` · PM-7 `core10-pkg-member-lib`.
- **Evidencia** (run exclusivo en Lex, 2026-10-02 19:26–20:55 CEST): `cargo fmt --all -- --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` 0; `cargo build --release -p arita-cli` 0; `cargo test` 498 passed / 0 failed; `arita measure` **851/851 accepted** sin skips; miri-workspace 0 fallos.
- **Veyra Proof** (quick): #1 `20261002T185553Z` **REJECTED** (VT006 ×3 en `tests/pkg_member.rs`, remediado solo en código de test, sin waiver); #2 `20261002T193930Z` **ACCEPTED**.
- **Artefacto de medición:** `DOC/reviews/MEASURE_ADR287_PKG_MEMBER_EXCLUSIVE_20261002.json` (sha256 `3a031a66…`, md5 `9afbc616…`). Freeze: `DOC/reviews/MEASURE-ADR287-PKG-MEMBER-FREEZE-20261002.sha`.
- **Shas al cierre:** `main.rs` `ed0820b4…`, `package.rs` `5fee2beb…`, `measure.rs` `10443913…`, `tests/pkg_member.rs` `9172a51e…`, `tests/measure_stdout.rs` `8bc7ba81…`.
- **B-measure-stdout cerrado aquí:** el stdout de `arita measure` es JSON puro (test `tests/measure_stdout.rs`).
- **Consecuencia:** los no-miembros históricos de core04 (`lib-api/bin/main-record.arita`, `ref-pkg-lib/bin/edge.arita`) y los 3 no-miembros del fixture nuevo dan E0332.
- **Limitación conocida:** la comparación `contract` del script de verificación no ejercita nada; la cobertura efectiva es PM-1..PM-7.
- **Backlog:** B-287-1/2/3 (P3) siguen abiertos (§7).
- **Registro de E0332:** no existe un catálogo dedicado de E-codes; las definiciones canónicas son este ADR, `package.rs`, `measure.rs` y `tests/pkg_member.rs` (E0332 dado de alta aquí).
- **Orden siguiente:** ADR-288 → ADR-289 → ADR-290 (IndexMut).
- Nota: sha del ADR tras este cierre: lo registra el addendum del Ingeniero.
