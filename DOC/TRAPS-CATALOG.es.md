Español | [English](TRAPS-CATALOG.md)

# ARITA — Catálogo de trampas (oráculos measure)

- **Estado:** **draft / v0** (+ E0214 vacuous len — CUT `TRAPS-LEN-THEATER-20260914`; + E0241 neg wire; + E0225 vacuous match — CUT `TRAPS-MATCH-VACUOUS-20260914`; + E0215 vacuous cmp — CUT `TRAPS-CMP-THEATER-20260915`; + E0242 borrow×await — CUT `TRAPS-BORROW-AWAIT-20260915`; + E0226 while false — CUT `TRAPS-WHILE-FALSE-20260915`; + E0227 if false — CUT `TRAPS-IF-FALSE-20260915`; + E0216 int div0 — CUT `TRAPS-INT-DIV0-20260915`; + E0217 int overflow — CUT `TRAPS-INT-OVERFLOW-20260915`; status DOC ADR-043 `CORPUS-STATUS-20260915` + ADR-046 `CORPUS-ARITH-20260915`)
- **Fecha:** 2026-09-14
- **SoT:** `<repo>` (Lex); companion to `arita measure` + `DOC/THREAT_MODEL.md`
- **Framing:** truth-gate / measure companion. Maps AI→Rust failure modes that existing **measure oracles already block**. Not an antivirus product; project name = **ARITA**.
- **Rule:** every **Covered now** row cites a wired oracle id + path under `ejemplos/` (or a required workspace oracle in `crates/arita-cli/src/measure.rs`). No unverified claims.
- **Roadmap:** starts Fase 4 item “Corpus trampas ARITA” — catalog v0 landed; ports from Investigación Rust-IA still pending.

## Cómo leer

| Columna | Significado |
|--------|---------|
| Trap | Nombre corto del modo de fallo de la IA |
| Why AIs do it | Atajo generativo típico al apuntar a código con forma Rust |
| ARITA defense | Código estable `E0xxx` / motor + id de oráculo measure |
| Path | Fixture `.arita` / contract / CLI usado por el oráculo |

Regla measure (anti-theater): **accepted** para un oráculo negativo iff el pipeline falla con el código **esperado**; éxito accidental o código incorrecto → **rejected**; fichero/toolchain ausente → **inconclusive**. `skip ≠ PASS`. Nunca `inconclusive` → `accepted`.

---

## Cubierto ahora

Fuente del inventario: `NEG_ORACLES` + runners neg CLI/deps/perf en `crates/arita-cli/src/measure.rs`, más companions notables **expect-fail / reject** (F3 `expect_sat=false`, contract `expect_reject`, attest bad-hash, clippy/miri requeridos). Los oráculos E2E positivos de stdout (hello, F2 `01`–`09`, etc.) quedan fuera de alcance salvo que sean gates anti-theater explícitos.

### A. Superficie anti-theater (E021x) — ADR-010

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 1 | `todo!` / `unimplemented!` stub | Dejar cuerpos a medias que “compilarán luego” | **E0210** · `neg-e0210-todo` | `ejemplos/f2/neg/e0210-todo.arita` |
| 2 | Tautological `assert true` | Tests verdes falsos sin evidencia | **E0211** · `neg-e0211-assert-true` | `ejemplos/f2/neg/e0211-assert-true.arita` |
| 3 | Empty `test` block | Reclamar cobertura con un test vacío | **E0212** · `neg-e0212-empty-test` | `ejemplos/f2/neg/e0212-empty-test.arita` |
| 4 | Stub valued `fn` (empty body) | Bocetar la forma de la API sin implementación | **E0213** · `neg-e0213-empty-fn` | `ejemplos/f2/neg/e0213-empty-fn.arita` |
| 4b | Vacuous `len` / `is_empty` theater | `assert v.len() >= 0`, `len()==len()`, `is_empty()||!is_empty()` | **E0214** · `neg-e0214-*` | `ejemplos/f2/neg/e0214-len-ge-zero.arita`, `e0214-len-eq-len.arita`, `e0214-is-empty-taut.arita` |
| 4c | Vacuous comparison assert | `assert n == n` / `n <= n` / lit==lit | **E0215** · `neg-e0215-*` | `ejemplos/f2/neg/e0215-eq-self.arita`, `e0215-le-self.arita` |
| 4d | Unchecked Int `/` `%` by zero | Lit `/0` o `div(x,0)` → teatro rustc/panic | **E0216** · `neg-e0216-div0` | `ejemplos/f2/neg/e0216-div0.arita` (+ lit companions; ADR-044 **verified** Lex **84**+2) |
| 4e | Unchecked Int `+/-/*` overflow | Lit MAX+1; wrap silencioso en release | **E0217** · `neg-e0217-add/sub/mul-overflow` | `ejemplos/f2/neg/e0217-*.arita`; CUT `E0217-ORACLES-SUBMUL-20260915` **DONE** |

### B. Ownership / borrow (E020x) — ADR-009

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 5 | Use after move | Copiar patrones Rust sin rastrear moves (`String`/`Vec`) | **E0201** · `neg-e0201-use-after-move` | `ejemplos/f2/neg/e0201-use-after-move.arita` |
| 6 | Double `&mut` / borrow conflict | Borrows mut en paralelo desde modelos mentales Python/JS | **E0202** · `neg-e0202-double-mut` | `ejemplos/f2/neg/e0202-double-mut.arita` |

### C. Whitelist std (E0206) — ADR-026

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 7 | Invented / out-of-whitelist method | Alucinar `Int.len()` o métodos Rust libres | **E0206** · `neg-e0206-bad-method` | `ejemplos/f2/neg/e0206-bad-method.arita` |

### D. Control flow / match (E022x) — ADR-014 / 015 / 016

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 8 | Non-Bool `if`/`while` condition | Usar Int/truthiness como C/Python | **E0220** · `neg-e0220-bad-cond` | `ejemplos/f2.1/neg/e0220-bad-cond.arita` |
| 9 | Non-exhaustive Bool `match` | Ramas parciales; olvidar `false` / `_` | **E0221** · `neg-e0221-bool-nonex` | `ejemplos/f2.2/neg/e0221-bool-nonex.arita` |
| 10 | Int `match` without `_` | Solo brazos lit finitos; olvidar el dominio abierto de Int | **E0222** · `neg-e0222-int-no-wild` | `ejemplos/f2.2/neg/e0222-int-no-wild.arita` |
| 11 | Pattern / scrutinee type mismatch | Mezclar patrones Bool sobre Int (y similares) | **E0223** · `neg-e0223-pat-mismatch` | `ejemplos/f2.2/neg/e0223-pat-mismatch.arita` |
| 12 | `break`/`continue` outside `while` | Meter keywords de bucle en `if` / bloques sueltos | **E0224** · `neg-e0224-break-outside` | `ejemplos/f2.3/neg/e0224-break-outside.arita` |
| 12b | Vacuous `match` arms (same constant) | Morph exhaustivo: todos los brazos `{ lit }` idénticos | **E0225** · `neg-e0225-match-*-same` | `ejemplos/f2.2/neg/e0225-match-bool-same.arita`, `e0225-match-int-same.arita` |
| 12c | Vacuous `while false` | Teatro de cuerpo de bucle inalcanzable; el print posterior aún “pasa” | **E0226** · `neg-e0226-while-false` | `ejemplos/f2.1/neg/e0226-while-false.arita` (ADR-040; **verified**) |
| 12d | Vacuous `if false` | Then inalcanzable; teatro de print posterior | **E0227** · `neg-e0227-if-false` | `ejemplos/f2.1/neg/e0227-if-false.arita` (ADR-041; **verified**) |

### E. Safe-only / async / deps / perf CLI

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 13 | `unsafe` / FFI in surface | Escapar a power tools Rust por “rendimiento” | **E0231** · `neg-e0231-unsafe` | `ejemplos/f2/neg/e0231-unsafe.arita` |
| 14 | `await` outside `async fn` | Llamar helpers async desde `main`/`fn` sync | **E0240** · `neg-e0240-await-outside` | `ejemplos/async/neg/e0240-await-outside.arita` |
| 14b | Illegal `async` outside `async fn` | Poner `async` en stmts/items que no son `async fn` | **E0241** · `neg-e0241-async-illegal` | `ejemplos/async/neg/e0241-async-illegal.arita` |
| 14c | Borrow held across `await` | `&`/`&mut` vivos atraviesan `await` (era agujero HIR → rustc E0100) | **E0242** · `neg-e0242-borrow-across-await` | `ejemplos/async/neg/e0242-borrow-across-await.arita` (ADR-039; **verified**) |
| 15 | Unknown CLI `--profile` | Inventar nombres de perfil; saltarse evidencia real de release | **E0250** · `perf-neg-e0250` | `ejemplos/perf/neg/e0250-bad-profile.arita` (CLI `--profile fantasma`) |
| 16 | Unknown `[deps]` crate | Tirar de nombres arbitrarios de crates.io | **E0260** · `deps-neg-e0260` | `ejemplos/deps/neg/e0260-unknown-crate/main.arita` (+ sibling `arita.toml`) |
| 17 | External crate path in surface | Escribir `tokio::…` como Rust libre | **E0261** · `deps-neg-e0261` | `ejemplos/deps/neg/e0261-crate-path.arita` |

### F. Isla lógica expect-fail (E03xx) — ADR-012

Measure **accepted** iff el motor lógico rechaza con el código listado (`expect_sat = false`). SAT accidental → **rejected**.

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 18 | Claim unsat path as proven | Afirmar alcanzabilidad que los hechos no sostienen | **E0301** · `f3-02-path-fail` | `ejemplos/f3/02-path-fail.arita` |
| 19 | Invented sibling / wrong join | Alucinar cierre relacional | **E0301** · `f3-04-sibling-fail` | `ejemplos/f3/04-sibling-fail.arita` |
| 20 | Unknown predicate | Nombres de predicado a medida | **E0303** · `f3-06-unknown-pred` | `ejemplos/f3/06-unknown-pred.arita` |
| 21 | Forbidden fn in logic island | Colar llamadas estilo host en la superficie Datalog | **E0304** · `f3-07-forbidden-fn` | `ejemplos/f3/07-forbidden-fn.arita` |
| 22 | Predicate arity mismatch | Aridad incorrecta por few-shot incompleto | **E0303** · `f3-08-arity-mismatch` | `ejemplos/f3/08-arity-mismatch.arita` |
| 23 | Partial-PASS multi-query | Declarar el módulo OK cuando una query sat y otra falló | **E0301** · `f3-11-multi-query-fail` | `ejemplos/f3/11-multi-query-fail.arita`; ADR-099 **verified** Logic **12/12** |
| 24 | Join-miss as proven | Alucinar cierre de join sobre dept/relación incorrectos | **E0301** · `f3-12-join-miss-fail` | `ejemplos/f3/12-join-miss-fail.arita`; ADR-099 **verified** Logic **12/12** |

### G. Companions contract / attest reject — ADR-017 / 018 / 019

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 23 | Contract ignores reject obligation | Embarcar un “contract” que solo espera stdout feliz | `expect_reject` **E0224** · `contract-neg-e0224` | `ejemplos/contracts/contract-neg-e0224.json` |
| 24 | Language `contract` without reject check | El mismo patrón in-source | `expect_reject` **E0224** · `lang-contract-neg-e0224` | `ejemplos/contracts/lang-neg-e0224.arita` |
| 25 | Pinned hash theater / wrong attestation | Afirmar source attested mientras los bytes difieren | attest mismatch · `contract-hello-bad-hash` (measure accepted **iff** reject) | `ejemplos/contracts/contract-hello-bad-hash.json` |

### H. Gates de workspace requeridos (skip ≠ PASS)

No son trampas `.arita`; siguen siendo oráculos measure anti-theater — un toolchain ausente nunca debe volverse `accepted` global.

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 26 | Skip Clippy / treat missing as PASS | “Verde” sin `-D warnings` en los crates del workspace | `clippy-workspace` (missing → **inconclusive**) | `crates/arita-cli/src/measure.rs` → `run_clippy_oracle` |
| 27 | Skip Miri / treat missing as PASS | Afirmar libre de UB sin ejecutar Miri | `miri-workspace` (missing → **inconclusive**) | `crates/arita-cli/src/measure.rs` → `run_miri_oracle` |

### I. Companion de evidencia de perfil release — ADR-028

| # | Trap | Why AIs do it | ARITA defense | Path |
|---|------|---------------|---------------|------|
| 28 | Fake “release” without opt-level evidence | Imprimir stdout con pinta de release sin emit real de perfil | `perf-02-release-optlevel` (requires `[profile.release] opt-level = 3` in generated `Cargo.toml`) | `ejemplos/perf/01-release-run.arita` (paired with release build) |

**Conteo cubierto (este catálogo v0): 38** trampas con defensas measure cableadas (+ E0217; + ADR-099 f3-11/12).

---

## Candidatos siguientes

Solo stubs. **PENDING** — no tratar como cobertura measure. Aquí no se inventa ningún oráculo.

| Stub | Notas | Status |
|------|-------|--------|
| E0217 sub/mul measure oracles | Addendum ADR-045 `E0217-ORACLES-SUBMUL-20260915` (sin ADR-047); Lex **87**+2 | **DONE** (pasado a Cubierto) |
| E0217 integer overflow | ADR-045 **verified** Lex **87**+2 (`TRAPS-INT-OVERFLOW` + `E0217-ORACLES-SUBMUL`); add/sub/mul | **DONE** (pasado a Cubierto) |
| Mutex×await theater | Necesita `Mutex` (+ interacción async) en surface — **PARK** hasta CUT de surface | **PARK** (ADR-038) |
| Negative string repeat | ADR-075 / **E0280** `negative repeat count`; CUT `STD-REPEAT-20260918` **verified** Lex **172/172** (+ Vec ADR-076) | **DONE**/GO |
| Invalid clamp range | ADR-072 / **E0279** `invalid clamp range`; CUT `STD-CLAMP-20260918` **verified** Lex **162/162** | **DONE** |
| Int abs overflow MIN | ADR-069 / **E0278** `integer abs overflow`; CUT `STD-ABS-20260918` **verified** Lex **153/153** | **DONE** |
| Negative pow exponent | ADR-079 / **E0281**/E0283**; CUT `STD-POW-20260918` **verified** Lex **184/184** | **DONE** |
| Vacuous while-let Result | ADR-078 / **E0282** `vacuous while-let result`; CUT `TRAPS-WHILE-LET-RESULT-VACUOUS-20260918` **verified** Lex **180/180** | **DONE** |
| Vacuous while-let None | ADR-056 / **E0277** `vacuous while-let none`; CUT `TRAPS-WHILE-LET-VACUOUS-20260918` **IMPL GO** | **DONE** Lex **118/118** |
| Option swallow / ignore-None | ADR-051 / **E0274** `option none swallowed`; CUT `TRAPS-OPTION-SWALLOW-20260918` **IMPL GO** | **GO IMPL** |
| Partial-PASS multi-query / join-miss | ADR-099 / **E0301**; CUT `F3-UNSAT-FAIL-NEXT-20260918`; Logic `f3_` **12/12** | **DONE** |
| Result swallow / ignore-Err | ADR-048 / **E0272** `result error swallowed`; CUT `TRAPS-RESULT-SWALLOW-20260918` **verified** Lex **96/96** | **DONE** |
| Port corpus from Investigación Rust-IA | Puertos ROADMAP Fase 4 aún abiertos; oleada + arith **ticked** vía ADR-043/046 DOC; mapear nombres de Investigación solo tras oráculo ARITA; morphs/Mutex **PARK**; next = ADR-047 Result IMPL (#58); Mutex/swallow PARK (#59); HOLE no-control standby | **PENDING** |
| Timed perf thresholds / mutation oracles | Explícitamente OUT de PERF-V0; serían oráculos nuevos, no inventario | **PENDING** (primero decisión de producto) |

---

## Non-goals (v0)

- **No** cambiar crates ni el comportamiento de measure por este DOC.
- **No** llamar a este catálogo antivirus / detección de malware.
- **No** listar smoke tests positivos solo-stdout como “trampas” salvo que sean gates reject/expect-fail.
- **No** afirmar ningún código como cubierto sin fila de oráculo measure (E0241 ahora cableado vía `neg-e0241-async-illegal`).

## Verify

```bash
# From repo root — negatives must fail with the listed code:
cargo run -p arita-cli -- parse ejemplos/f2/neg/e0210-todo.arita
cargo run -p arita-cli -- measure   # includes NEG_ORACLES + companions; skip ≠ PASS
```

## Relacionados

- `DOC/THREAT_MODEL.md` — modelo de amenazas anti-theater
- `DOC/ADR/010-e021x-negative-oracles.md` — E021x
- `ejemplos/README.md` — tablas completas de oráculos
- `crates/arita-cli/src/measure.rs` — SoT de ids cableados
