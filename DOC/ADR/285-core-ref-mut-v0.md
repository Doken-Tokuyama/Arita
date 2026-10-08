# ADR-285 — Core 0.9 slice 4: REF-MUT

- **Estado:** **CLOSED** Lex **834/834** (Ingeniero 2026-09-27 · [`GATE-CORE09-REF-MUT-20260926.md`](../GATE-CORE09-REF-MUT-20260926.md)) · **cierra Core 0.9** · GO IMPL Ingeniero 2026-09-27 · pins OK Ingeniero 2026-09-26
- **CUT-ID:** `CORE-0.9-REF-MUT-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero (pins OK 2026-09-26 19:54)
- **Padre:** [ADR-281](281-core-0.9-pins.md) §0.1 slice 4 · §2 programa ref
- **Prev:** slice 3 SCENARIO-MUT (ADR-284) — prereq CLOSED al GO IMPL; este CUT **no** reabre 282–284
- **Prereq surface (al GO IMPL):** MAP-ASSIGN + VEC-ASSIGN + SCENARIO-MUT en tree + packages 0.4 layout (255/258) + `fn → Result` / `?` (277/278) + CLI/JSON 0.1 (238/240)
- **Cierra:** vertical Core **0.9** (Mutación de colecciones) — al CLOSED de este slice
- **HOLD:** Mutex · threads · idle · TLS/WS · crates.io · repair · I/O-new · String.set · IndexMut residual (E0314) · `?` in Io main · Option? · surface unwrap/expect (241) · HTTP daemon · MCP/ACP · reopen E0272/E0291/E0340/E0341/E0342/E0343

## Objetivo

Ref **no trivial** `arita-ref-mut` + evidence + Lex barra que **cierra Core 0.9**: workspace lib+bin; lib `pub fn → Result<_, Int>` que muta `Map` vía `m[k] = v` y `Vec` vía `v[i] = x` (OOB → `Err(0)` propagado) y compone con `?`; CLI main `Io<()>` **match-convierte**; happy + Err paths; scenarios verdes; evidence hash. Sin API nueva. skip ≠ PASS. **No** inventar PASS/N/N en este DOC.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Nombre** | `arita-ref-mut` (raíz `ejemplos/core09/ref-mut/`) |
| **Layout** | `arita.toml` → emit Cargo workspace `{lib,bin}` (255/258/268/280 style). Raíz `ejemplos/core09/ref-mut/` (workspace lib+bin, `fixture.json`, happy); segundo paquete `ejemplos/core09/ref-mut/edge/` (lib+bin, `fixture.json` `{"i": 7, "x": 9}`, OOB), porque `arita contract` ejecuta sin argumentos (precedente ADR-280); negs en `ejemplos/core09/ref-mut-neg/`, fuera del workspace (pin Ingeniero 2026-09-27 07:42) |
| **Lib** | ≥2 `pub` ítems `-> Result<…, Int>`: construyen `Vec`/`Map` (params en tipos `fn_param` v0 ya IN; **sin** widen a Vec/Map ni `&mut Map/Vec` — HOLD ADR-281 D1), mutan con `m[k] = v` + `v[i] = x`, componen con `?`; dominio tipado; **sin** HTTP/Mutex; **sin** unwrap |
| **CLI** | `fn main() -> Io<()>`: args y/o JSON (238/240) → lib → match Ok/Err → stdout **determinista** (happy + fail OOB); **cero** `?` y **cero** `v[i] =` en main Io (`m[k] = v` permitido) |
| **Scenarios** | exactamente dos: `ref_mut_happy` (raíz) y `ref_mut_err` (edge), con los stdout exactos de §2 (pin Ingeniero 2026-09-27 07:42) |
| **Evidence** | JSON hash source↔emit↔scenario (`tested`); sin dual-oracle (G4); gates §2.1 |
| **Emit ban** | cero IndexMut / `x[..] =` Rust · cero unwrap/expect/panic (241) · cero `?` en main Io · cero discard del set + `as usize` en las llamadas `__arita_vec_set` y sus helpers (ADR-283 R1; el `as usize` guardado de `v.get` queda fuera); recuento exacto en §2 |
| **OUT** | new host APIs · Mutex · String.set · net · crates.io · Option? · async Result · params colección |

## 1. Surface

Solo APIs ya IN (281–284 + 276–280 + 264–270 + 237/238/240 + packages 0.4). Sin std/keyword nueva más allá del sugar de escritura (282/283).

```text
// lib: pub fn apply(i: Int, x: Int) -> Result<Int, Int>   // m[k] = v ; v[i] = x (OOB → Err(0))
//      pub fn run(i: Int, x: Int) -> Result<Int, Int>     // apply(i, x)? ; …
// bin: args → run → match Ok/Err → print   // no ? / no v[i] = in main Io
scenario ref_mut_happy { acceptance { /* stdout vía lib */ } }
scenario ref_mut_err   { acceptance { /* OOB Err(0) propagado, sin panic */ } }
```

## 2. Oracles / evidence — gate CLOSED Core 0.9

Requeridos (a medir al GO IMPL; **ninguno** medido en este DOC):

| Id | Expect |
|----|--------|
| `core09-ref-mut-build` | raíz `ejemplos/core09/ref-mut/` y edge `ejemplos/core09/ref-mut/edge/`: build workspace lib+bin verde **en ambos** (existen `crates/lib_*` y `crates/bin_*`). Reglas de fuente (anti-theater, §0 L23–24): (1) lib con ≥2 `pub fn … -> Result<…, Int>`; (2) lib con ≥1 `m[…] =` y ≥1 `v[…] =`; (3) lib con ≥1 `?` de composición; (4) cero `unwrap`/`expect` en lib+bin; (5) `bin/main.arita` y `edge/bin/main.arita`: cero `?` y cero `v[…] =` (`m[…] =` permitido); (6) si `edge/lib/lib.arita` ≠ `lib/lib.arita`, Codegen justifica la diferencia y Measure la registra en el evidence/review (no es rechazo automático). Cualquier fallo de (1)–(5) ⇒ rejected; fichero ausente ⇒ inconclusive (pin Ingeniero 2026-09-27 07:42) |
| `core09-ref-mut-cli-happy` | CLI vía lib (Map insert/overwrite + Vec in-bounds) con `ejemplos/core09/ref-mut/fixture.json` **y** sin argumento (fixture por defecto): **exit 0**, stderr sin «panicked» (Codegen READY: stderr vacío), stdout exacto `["2","9","3","ref-mut-happy"]` (= `2\|9\|3\|ref-mut-happy`, una línea por token) en ambos runs (pin Ingeniero 2026-09-27 07:42) |
| `core09-ref-mut-cli-oob` | workspace edge (`edge/fixture.json` `{"i": 7, "x": 9}`) con y sin argumento: Vec OOB → `Err(0)` propagado → rama `Err` de main; **exit 0** (G2; precedente ADR-280 `cli-fail` y ADR-284 L24 «Err en main»), stderr sin «panicked», stdout exacto `["ref-mut-err","0"]` (= `ref-mut-err\|0`). Payload ≠ `0` → **REJECTED**; también rejected si aparece `ref-mut-happy` o hay líneas de más. **Anti-theater (mismo id):** el bin **raíz** con `-- ejemplos/core09/ref-mut/edge/fixture.json` también debe dar exactamente `["ref-mut-err","0"]` (pin Ingeniero 2026-09-27 07:42) |
| `core09-ref-mut-scenario` | exactamente dos bloques `scenario`: `ref_mut_happy` (`bin/main.arita`) y `ref_mut_err` (`edge/bin/main.arita`); sus `acceptance` = los dos stdout exactos de arriba (happy `["2","9","3","ref-mut-happy"]` · err `["ref-mut-err","0"]`); `arita contract` **Accepted** en los dos bins; los dos oracles cli-* Accepted. un número distinto de dos o nombres distintos ⇒ rejected (skip ≠ PASS) (pin Ingeniero 2026-09-27 07:42) |
| `core09-ref-mut-evidence` | `ejemplos/core09/ref-mut/evidence.json`: `schema_version` = `arita.evidence.v1`, `cut_id` = `CORE-0.9-REF-MUT-20260926`, `adr` = `"285"`; sha256 declarados == reales de `lib/lib.arita`, `bin/main.arita`, `edge/bin/main.arita`, `edge/lib/lib.arita`, `arita.toml`, `edge/arita.toml`, `fixture.json` y `edge/fixture.json`; hash del emit (`lib.rs`/`main.rs`) estable entre dos builds consecutivos (si el evidence declara `emit_*_sha256`, deben coincidir); `scenarios` = exactamente los 9 ids de §2; `measure_pass` informativo: el oracle exige solo que sea bool (no `true`); pasa a `true` al CLOSED sin tocar el oracle (G3; gates §2.1) (pin Ingeniero 2026-09-27 07:42) |
| `core09-ref-mut-emit-ban` | sobre todo `lib.rs`/`main.rs` emitido (raíz + edge): cero `.unwrap()`, `.expect(`, `panic!(` (241), cero `IndexMut`/`std::ops::Index` y asignación Rust indexada `ident[…] =`; cero `as usize` en los sitios de llamada `__arita_vec_set` y en los cuerpos de los helpers `__arita_vec_set`/`__arita_vec_insert` (mismo alcance que el oracle de ADR-283 R1; **no** es un ban de fichero entero): el `as usize` de la lectura `v.get(sel)`, guardado por `__i < 0`, es legítimo y queda fuera (ADR-283 §1.2). Cada `main.rs`: exactamente 0 `?`, 0 `__arita_vec_set` y 3 `.insert(`. `lib.rs`: exactamente **4** `__arita_vec_set(&mut …)?` (uno por sitio fuente `v[…] =`), todos con `?`, sin `let _ =`/descarte del set; exactamente **6** `.insert(` (conteos fijados por Measure/Codegen READY) (pin Ingeniero 2026-09-27 07:42) |
| `neg-core09-ref-vec-assign-outside` | `v[i] = x` en main Io → exactamente `E0344: index assign outside result fn` (código único; fixture `ejemplos/core09/ref-mut-neg/01-vec-assign-outside.arita`, span `@812..820` = `w[0] = 3`). Otro código, más de uno o build verde ⇒ rejected (pin Ingeniero 2026-09-27 07:42) |
| `neg-core09-ref-map-assign-non-mut` | `m[k] = v` sobre binding no-mut → exactamente `E0202: borrow conflict` (= `put`; código único; fixture `ejemplos/core09/ref-mut-neg/02-map-assign-non-mut.arita`). E0314/E0205/E0001 u otro ⇒ rejected (pin Ingeniero 2026-09-27 07:42) |
| `neg-core09-ref-vec-assign-neg-lit` | `v[-1] = x` en fn→Result → exactamente `E0319: negative set index` (código único; fixture `ejemplos/core09/ref-mut-neg/03-vec-assign-neg-lit.arita`) (pin Ingeniero 2026-09-27 07:42) |

**Negs (G5):** mensajes de ADR-281 L124/L125/L128. Fixtures nuevos en la carpeta hermana `ejemplos/core09/ref-mut-neg/`, fuera del workspace (**no** dentro de `ref-mut/`, cuyo `arita.toml` tiene `[workspace]` y secuestraría el build); ubicación **aprobada** por el Orquestador (precedente `ref-*-neg`). Cada neg: un solo código, exacto y único — E0344 · E0202 · E0319. Sin dual-oracle separado (G4).

**Estado de los pins §2:** en firme (Ingeniero 2026-09-27 07:42), sobre la propuesta Measure (`MEASURE_PREP_ADR285_REF_MUT_20260927.md`, md5 `05c5eab0f4d5b1f6e88353f70cdf7e1d` al pinear; versión vigente `5a88446ffe6d61e1b0e32bd4ed7654f2` con §11 añadido, sin cambio de pins) y Codegen READY (`ADR-285-CODEGEN-REF-MUT-20260927.md`, md5 `3d8add5d89c75fce1ee49e7d817823e5`). Los stdout happy y OOB son re-verificables en el run final; si difieren, se corrige con visto Ingeniero, nunca el oracle a mano. Condición (a) del GO IMPL cumplida en DOC.

N = 834 fijado por el Ingeniero (2026-09-27 07:42). skip ≠ PASS. **No** inventar PASS en draft.

### 2.1 Gates de evidence (pin Ingeniero 2026-09-27 07:42)

- **source↔evidence:** sha256 declarados en `evidence.json` == sha256 reales de las fuentes listadas en `core09-ref-mut-evidence`.
- **emit determinista:** sha256 de `lib.rs`/`main.rs` emitidos idéntico entre dos builds consecutivos.
- **scenarios:** `scenarios` == los ids medidos (`tested`), exactamente los 9 de §2.
- **`measure_pass`:** informativo; el oracle exige solo que sea bool (no `true`). Pasa a `true` al CLOSED y el oracle **no** se toca después (G3).

## 3. Qué NO tocar (HOLDs + post-0.9)

| Prohibido | Motivo |
|-----------|--------|
| Mutex / threads / idle / TLS/WS / crates.io / repair / I/O-new / String.set | HOLD global |
| IndexMut residual | HOLD (E0314) |
| `?` in Io main · Option? · unwrap surface | OUT / HOLD |
| HTTP daemon / compose · MCP/ACP | fuera de este vertical |
| GO IMPL antes CLOSED 282–284 + OK Ingeniero | gate Orquestador |
| CLOSED Core 0.9 sin Lex §2 verde | solo este slice cierra 0.9 cuando barra pase |
| Claim Core 0.9 CLOSED en draft | **NO** |
| Editar ROADMAP en este CUT DOC | lo hace Ingeniero/Orquestador al CLOSED |

## 4. Criterio CLOSED (slice 4 = Core 0.9 CLOSED)

Lex §2 verde con N exacto = **834** (825 + 9 ids del §2; Ingeniero 07:42); ROADMAP/ADR-281 Core 0.9 **CLOSED**; HOLDs §3 **no** unpark (salvo sugar 282/283 ya land); migración oráculos ADR-281 §0.3 aplicada.

**Cierre (cumplido 2026-09-27):** run final exclusivo 834/834 + freeze 946/946 · `cargo test -p arita-cli` exclusivo 154/0 · Veyra `--timeout 2400` `20260927T063410Z` **ACCEPTED** exit 0 · OK Ingeniero.

## 5. Post-0.9 (no este CUT)

Siguiente vertical: solo con GO <person> + ADR con oráculos. Candidatos HOLD: String.set (UTF-8 byte/char, ADR propio) · Option `?` · `?` en main Io · must-use / discard-Result (backlog D2; sin E0345) · relajar args de `put` · split E0314 · receptor loan / params `&mut Map/Vec` (HOLD D1) · Mutex (229) · idle-kill · TLS/WS · crates.io · repair. **No** reescribir 0.9 mid-flight. Vertical ≠ techo GP.

## Checklist

- [x] Pins ref `arita-ref-mut` + oracles/evidence + qué NO tocar (GO-listo DOC)
- [x] Revisión Ingeniero 2026-09-26 19:54 — pins OK · HOLD
- [x] GO IMPL (post-CLOSED 282–284) — Ingeniero 2026-09-27
- [x] 2026-09-27 07:41 · pins §2 propuesta Measure escritos (9 ids, stdout/códigos exactos) · N propuesto 834 · pendiente visto Ingeniero + Codegen READY
- [x] 2026-09-27 07:42 · pins §2 en firme (Ingeniero): N=834 sin dual-oracle · G2 cli-oob exit 0 + anti-theater bin raíz con edge/fixture.json · G3 measure_pass informativo (bool), gates §2.1 · negs en ref-mut-neg/ aprobados (E0344/E0202/E0319) · layout edge/ en §0 · Codegen READY
- [x] 2026-09-27 07:44 · Measure re-verifica con Codegen READY, sin cambios de pins · alcance `as usize` = llamadas/helpers `__arita_vec_set` (como 283) · conteos emit: 4 `__arita_vec_set(&mut …)?`, `.insert(` 6 en lib.rs y 3 en cada main.rs · scenarios exactamente dos
- [x] 2026-09-27 08:45 · IMPL + Lex 834/834 (measure 08:05–08:20 + cargo test 154/0 exclusivos, freeze 946/946) + Veyra `20260927T063410Z` ACCEPTED → **CLOSED** · **Core 0.9 CLOSED** (Ingeniero)

## Cierre

- **CLOSED** (Ingeniero 2026-09-27) · Lex **834/834** · [`MEASURE_ADR285_REF_MUT_20260927.json`](../reviews/MEASURE_ADR285_REF_MUT_20260927.json) md5 `a502f785…` · cargo test 154/0 · freeze 946/946 · Veyra `20260927T063410Z` ACCEPTED exit 0 · gate [`GATE-CORE09-REF-MUT-20260926.md`](../GATE-CORE09-REF-MUT-20260926.md) · **Core 0.9 CLOSED** · post-0.9 HOLD hasta GO <person> (§5).
