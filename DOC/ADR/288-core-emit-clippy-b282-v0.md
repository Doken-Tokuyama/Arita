# ADR-288 — Core emit-clippy B-282 v0: oráculos `measure` para B-282-1 (`unnecessary_to_owned`) y B-282-2 (`unused_parens` en `let n: Int = m.len()`)

- **Estado:** **CLOSED (Ingeniero, GO CLOSED 2026-10-03; DOC/GATE-CORE10-EMIT-CLIPPY-B282-20261003.md)** · `arita measure` 853/853 accepted (N = 853, k = 2) · historia: APROBADO Y CONGELADO (Ingeniero 2026-10-02, sha del ADR congelado `252d5dd4e6acb89c1a0cdb576e4ce3ac869f354264c9ce014067e2c04d035302` (corregido: es el sha tras el sello; el v0.1 previo al sello fue `bbf27929432ab1da3b8b5b7438404d42393d7a814c45321880fc3c8b32696e1b`)), GO IMPL tras PKG-MEMBER CLOSED; freeze `DOC/reviews/MEASURE-ADR288-EMIT-CLIPPY-FREEZE-20261003.sha` y addendum de freeze `DOC/reviews/MEASURE-ADR288-EMIT-CLIPPY-FREEZE-20261003.addendum.sha`.
- **CUT-ID:** `CORE-EMIT-CLIPPY-B282-20261001` (provisional; lo confirma el Orquestador)
- **Fecha:** 2026-10-01
- **Autores:** ARITA Arquitecto (pins) · orden y alcance decididos por el Ingeniero 01-10 (S1b → PKG-MEMBER → unused_parens/B-282 → IndexMut → Mutex)
- **Padre / contexto:** [ADR-282](282-core-map-assign-v0.md) (R3, L76/L132) · [ADR-283](283-core-vec-assign-v0.md) (FASE 2, fix de codegen) · `DOC/GATE-CORE09-MAP-ASSIGN-20260926.md` L40–42 · `DOC/reviews/NOTE_B282_STATUS_20260927.md` · `DOC/reviews/PREP_B282_ORACLES_PROPOSAL_20261001.md` (Measure, md5 `e70d197a635299bc3582568d307ec43f`) · T09-22 (`TRAPS_CORE09_SURFACE`)
- **Cierra:** **B-282-1** y **B-282-2** (CLOSED 2026-10-03; el fix de codegen ya existía y ahora está medido por los oráculos `measure` de este slice).
- **Códigos:** **ninguno** (son warnings de clippy/rustc sobre el Rust emitido, no diagnósticos E-code).
- **No reabre:** ADR-282 / 283 / 284 / 285 · E0291 / E0340–E0343 · ADR-265 / 270 / 278 · ADR-286 / 287.
- **Fuera de alcance:** **B-283-1** (`unused_parens` general en `if`/`while`/asignaciones/`let` con binarios, incluido el test `lib.rs:4121` `contains("(a + b)")`), el caso `let c: i64 = (v.capacity() as i64);` de `ejemplos/f2/184-shrink-to-fit-vec.arita` y el criterio «cero warnings de rustc en el Rust emitido de los positivos» → **ADR-289** (cambio de codegen real, test 4121 migrado en su land, medición rustc/clippy sobre los positivos, PREP propio de Codegen/Measure). Orden vigente: S1b → PKG-MEMBER (287) → EMIT-CLIPPY-B282 (288) → unused_parens (289) → IndexMut → Mutex.

## 1. Hechos (no hipótesis)

Datos tomados de `PREP_B282_ORACLES_PROPOSAL_20261001.md` y del código; ninguna ejecución propia del Arquitecto.

| Id | Síntoma original | Fix ya en codegen |
|---|---|---|
| **B-282-1** | `m.get("lit")` emitía `.get(&"lit".to_string())` ⇒ clippy `unnecessary_to_owned` | emite `m.get("lit").cloned()` solo si el argumento es `LitStr` y el receptor es un binding `Map` local (`crates/arita-codegen/src/lib.rs` ~L2671–2689) |
| **B-282-2** | `let n: Int = m.len()` emitía `let n: i64 = (m.len() as i64);` ⇒ rustc `unused_parens` | emite `let n: i64 = m.len() as i64;` solo si el init es `MethodCall len` (~L1817–1822, `strip_one_outer_parens`) |

Estado de la cobertura hoy:
- Tests cargo (no cuentan en N): `b282_emit_clippy::b282_1_map_get_string_literal_key_is_clippy_clean` y `…b282_2_let_int_len_has_no_outer_parens_and_is_clippy_clean` (`lib.rs` L5669–5735), con `clippy-driver` en ediciones 2015 y 2021, `-D warnings -D clippy::all`.
- En `measure.rs` **no existe ningún oráculo B-282** (`rg b282` vacío). Hoy hay un único oráculo emit-clippy: `core09-map-assign-emit-clippy` (`run_core09_map_assign_emit_clippy_oracle`, `measure.rs` ~L9705–9813), que evitó a propósito los dos casos B-282.
- Inconsistencia documental: `GATE-CORE09-REF-MUT-20260926.md` L33 ya marca B-282-1/2 CERRADOS; `GATE-CORE09-VEC-ASSIGN` L42 y `GATE-CORE09-SCENARIO-MUT` L28 los siguen listando como backlog.

Conclusión: **este slice no cambia codegen**. Añade los oráculos `measure` y sus fixtures, y deja los dos B-282 medidos por la misma vía que el resto del gate.

## 2. Decisiones (pins)

**D1 — Dos oráculos, uno por fix (k = 2).** Ids: `core09-b282-1-map-get-lit-emit-clippy` y `core09-b282-2-let-len-emit-clippy`. Sin neg (no hay diagnóstico que pinear). Se descarta un id combinado: un fallo debe señalar qué fix regresó.

**D2 — Fixtures nuevos y mínimos** (los crea IMPL, no este ADR; no se reutilizan fixtures de core06 porque no ejercitan el literal en `get`): `ejemplos/core10/emit-clippy/01-map-get-lit.arita` y `ejemplos/core10/emit-clippy/02-let-int-len.arita`, con los programas de §4. Entran en el freeze del slice cuando se creen.

**D3 — Qué mide cada oráculo (las tres comprobaciones deben cumplirse):**
1. **Emit:** el Rust emitido contiene la forma buena y no la mala (B-282-1: contiene `m.get("a").cloned()` y no contiene `.get(&"a".to_string())`; B-282-2: contiene `let n: i64 = m.len() as i64;` y `let k: i64 = v.len() as i64;`, y no contiene `(m.len() as i64)` ni `(v.len() as i64)`).
2. **Clippy en dos configuraciones (D4):** edición 2015 con `clippy-driver` y edición 2021 con `cargo clippy -- -D warnings` en crate scratch; ambas limpias.
3. **Ejecución:** el binario emitido imprime exactamente el stdout esperado: `7` (B-282-1) y `1\n1` (B-282-2).

**D4 — Ediciones y herramienta (ambas obligatorias).** Cada oráculo mide **dos** configuraciones y las dos deben quedar limpias para `accepted`: (a) **edición 2015** (la real del camino por defecto de `arita build`: en debug síncrono el CLI invoca `rustc <fichero>.rs -o <bin>` sin `--edition`, `crates/arita-cli/src/main.rs` L450–455 / `crates/arita-codegen/src/lib.rs` L154) medida con **`clippy-driver`** (`--edition 2015 -D warnings -D clippy::all`), como los tests `b282_emit_clippy`; (b) **edición 2021** (camino Cargo: release, async, `[deps]`, host-bridges o `--target`, `main.rs` L422; el `Cargo.toml` emitido lleva `edition = "2021"`, `lib.rs` L1266) medida con **`cargo clippy -- -D warnings`** en crate scratch, como `core09-map-assign-emit-clippy` (`measure.rs` ~L9705–9813). Si falta `clippy-driver` **o** `cargo clippy` ⇒ **inconclusive**, nunca accepted (D5). **IMPL no puede reducir a una sola edición/herramienta sin volver al Arquitecto.**

**D5 — Sin clippy ⇒ inconclusive**, nunca accepted ni skip silencioso (igual que el oráculo existente: `measure.rs` ~L9772–9782); «sin clippy» incluye que falte `clippy-driver` **o** `cargo clippy` (D4). Clippy presente y con avisos en cualquiera de las dos configuraciones ⇒ rejected. skip ≠ PASS.

**D6 — Test cargo existentes.** Los dos tests `b282_emit_clippy` de `lib.rs` se mantienen tal cual (no se renombran ni se relajan). El test `lib.rs:4121` es B-283-1 (confirmado por el Ingeniero 01-10; la migración se movió a ADR-289); este slice no toca 4121 ni los dos `b282_emit_clippy`.

**D7 — GATE.** No se reescriben GATE-283/284/285 ni sus textos históricos. Al CLOSED se crea un **addendum** propio de este slice (`DOC/GATE-CORE10-EMIT-CLIPPY-B282-<fecha>.md`) que marca B-282-1/2 arreglados y medidos, y enumera las tres inconsistencias de §1 (REF-MUT ya CERRADOS; VEC-ASSIGN y SCENARIO-MUT con texto histórico de backlog).

## 3. Oráculos y N

| Id | Fixture | Expect | stdout |
|---|---|---|---|
| `core09-b282-1-map-get-lit-emit-clippy` | `ejemplos/core10/emit-clippy/01-map-get-lit.arita` | accepted | `7` |
| `core09-b282-2-let-len-emit-clippy` | `ejemplos/core10/emit-clippy/02-let-int-len.arita` | accepted | `1\n1` |

Se registran en `run_measure_with`, junto al registro Core09 (`measure.rs` ~L16192).

k = 2. **N no se fija en este ADR:** N es siempre N_CLOSED previo + k, calculado por el Ingeniero al GO IMPL. Provisional con el orden vigente: tras PKG-MEMBER (851) ⇒ **853**; si se cerrara directamente tras S1b (844) ⇒ 846. Reglas: skip ≠ PASS; ningún oráculo cuenta si alguna de las dos configuraciones clippy de D4 (2015 con `clippy-driver`, 2021 con `cargo clippy`) no se ejecutó.

## 4. Programas de los fixtures

Fixture 01 (`01-map-get-lit.arita`), programa del test `b282_1` de `lib.rs`:

```arita
module t
fn main() -> Io<()> {
  let mut m: Map<Text, Int> = Map::new()
  m["a"] = 7
  let o: Option<Int> = m.get("a")
  match o {
    Some(x) => { print(x) }
    None => { print(0) }
  }
}
```

Fixture 02 (`02-let-int-len.arita`), programa del test `b282_2` de `lib.rs`:

```arita
module t
fn main() -> Io<()> {
  let mut m: Map<Text, Int> = Map::new()
  m["a"] = 7
  let mut v: Vec<Int> = Vec::new()
  let n: Int = m.len()
  v.push(n)
  let k: Int = v.len()
  print(n)
  print(k)
}
```

(El módulo y los nombres definitivos los fija IMPL; deben conservar las construcciones `m.get("a")` con literal sobre un `Map` local y `let n: Int = m.len()` / `let k: Int = v.len()`, y el stdout de §3.)

## 5. Tests cargo y criterios de cierre

- `cargo test -p arita-codegen` verde (incluye los dos `b282_emit_clippy` sin cambios).
- `cargo test -p arita-cli` verde (smoke de `measure.rs` actualizado si cuenta ids).
- `cargo fmt --check` y `cargo clippy --workspace --all-targets -- -D warnings` sin regresión; miri-workspace verde si estaba verde en el CLOSED previo.
- `arita measure` N/N con el N de §3, sin skip ni inconclusive; **se exigen `clippy-driver` y `cargo clippy` disponibles en Lex** (las dos configuraciones de D4) para contar.

## 6. PREP de Measure (bloquea GO IMPL)

Escaneo de solo lectura: (1) confirmar que ningún oráculo verde existente depende de la forma vieja (`.get(&"…".to_string())` o `(x.len() as i64)` en el emit); (2) inventario de los `.arita` positivos con `let n: Int = <x>.len()` y con `get("lit")` sobre Map local (23 ejemplos con `let n: Int = v.len()` según el PREP de Measure) y confirmar que todos siguen aceptados con el fix ya presente (es la situación actual: no hay cambio de codegen); (3) confirmar que el nombre de los ids nuevos no colisiona con ids existentes.

## 7. Preguntas abiertas (Ingeniero)

**Resueltas por el Ingeniero 01-10:** Q1 (4121 = B-283-1, pasa a ADR-289; D6 intacto); Q2 (ambas ediciones, 2015 con `clippy-driver` + 2021 con `cargo clippy`; ver D4); Q3 (el addendum de D7 basta; GATE-283/284/285 y REF-MUT/VEC-ASSIGN/SCENARIO-MUT no se tocan). Sin preguntas abiertas.

## 8. Slice y GO

Slice único `EMIT-CLIPPY-B282` (tamaño S, solo `measure.rs` + fixtures + evidence). Secuencia: DOC (este ADR) → verificación del Ingeniero → PREP Measure (§6) → **GO IMPL** del Ingeniero con PKG-MEMBER CLOSED → implementación → Lex → CLOSED con evidencia real. Prohibido inventar PASS / N/N / CLOSED.

## 9. Changelog

- 2026-10-01 — v0 DRAFT PINS (Arquitecto). Basado en el PREP de Measure; hallazgo clave: el fix de codegen ya existe y solo falta el oráculo `measure`. Sin ejecución propia.
- 2026-10-01 — v0.1 (Arquitecto): decisiones del Ingeniero: B-283-1 / test 4121 / capacity() → ADR-289 (orden tras 288); D4 con ambas ediciones y herramientas (2015 `clippy-driver` + 2021 `cargo clippy`; IMPL no reduce); D6 intacto; Q1–Q3 resueltas. Pins D1–D3, D5, D7 sin otros cambios.

## 10. Cierre (CLOSED 2026-10-03)

Datos tomados de `DOC/GATE-CORE10-EMIT-CLIPPY-B282-20261003.md` (sha256 `f04dc2409e30aa7af8e5b07022727b3975ab9d688dfe299dc0e532fc1e71b720`).

- **Veredicto:** GO CLOSED ADR-288 (Ingeniero Rust, 2026-10-03). Medida final **853/853 accepted**, 0 skip.
- **Oráculos** (k = 2, N previo 851 → N = 853): `core09-b282-1-map-get-lit-emit-clippy` (fixture 01) y `core09-b282-2-let-len-emit-clippy` (fixture 02), ambos accepted; los 851 previos sin cambios frente a `MEASURE_ADR287_PKG_MEMBER_EXCLUSIVE_20261002.json`.
- **Evidencia** (run exclusivo en Lex, 2026-10-02 23:54 → 2026-10-03 03:40, `/tmp/ing-adr288-final`): `cargo fmt --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` 0; `cargo build --release` 0; `cargo test --workspace -- --test-threads=1` 502 passed / 0 failed; `arita measure` exit 0, stdout JSON puro, 853 ids únicos, 853 accepted, 0 skip; Miri dentro de measure y de Veyra (verde).
- **Veyra Proof** (`--profile quick`): **ACCEPTED** `20261003T003703Z` (rustfmt, cargo check, clippy, cargo test, cargo-audit, veyra-anti-theater; 0 problemas, 0 skipped). El GATE no cita ningún run REJECTED previo.
- **Herramientas:** rustc/cargo 1.97.1, clippy 0.1.97. **Nota de PATH:** los oráculos B-282 invocan `clippy-driver` (en `~/.cargo/bin`); ese directorio debe estar en el `PATH` del run de measure; sin él los 2 oráculos no pueden pasar (no hay skip silencioso).
- **Artefacto de medición:** `DOC/reviews/MEASURE_ADR288_EMIT_CLIPPY_EXCLUSIVE_20261003.json` (sha256 `5e042970257339a0…`). **Freeze:** `DOC/reviews/MEASURE-ADR288-EMIT-CLIPPY-FREEZE-20261003.sha` (997 líneas: 6 de cabecera + 991 verificables con `shasum -a 256 -c`; sha256 del fichero `f0855defdd2b…`) y su addendum `DOC/reviews/MEASURE-ADR288-EMIT-CLIPPY-FREEZE-20261003.addendum.sha` (el GATE lo cita sin ruta; la ruta es la del fichero existente).
- **Shas de código al cierre:** `crates/arita-cli/src/main.rs` `ed0820b4…` y `package.rs` `5fee2beb…` sin cambios; `crates/arita-cli/src/measure.rs` `10443913…` → `159a07ad66fe…` (B282_ORACLES + smoke, solo aditivo); fixture 01 `703aa559ea1e…`, fixture 02 `ac633cb86b90…`.
- **Addendum (Ingeniero, 2026-10-03): fixture 01:** `print(0)` → `print("none")` por E0274 / ADR-051 §2; el id del oráculo, el stdout (7) y el mecanismo clippy no cambian. (Según el GATE: `None => { print(0) }` → `None => { print("none") }`; aprobado por el Ingeniero y recogido en el addendum del freeze; los programas de §4 no se reescriben.)
- **Limitación conocida / backlog:** el GATE no declara ninguna limitación ni backlog nuevo; solo la nota de PATH de arriba.
- **Orden siguiente:** ADR-289 → ADR-290 (IndexMut) → Mutex. El GATE indica como siguiente el edit de cierre del Arquitecto (ADR-288 + addendum de ADR-289) y el arranque de IMPL de ADR-289 (GO IMPL = ADR-288 CLOSED).
- Nota: sha del ADR tras este cierre: lo registra el addendum del Ingeniero.
