# ADR-283 — Core 0.9 slice 2: VEC-ASSIGN

- **Estado:** **CLOSED** Lex **816/816** (Ingeniero 2026-09-27 · REMEASURE3 exclusivo · Veyra ACCEPTED `20260927T013200Z`) → [`GATE-CORE09-VEC-ASSIGN-20260926.md`](../GATE-CORE09-VEC-ASSIGN-20260926.md) · antes: GO IMPL 2026-09-26 tras CLOSED ADR-282
- **CUT-ID:** `CORE-0.9-VEC-ASSIGN-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero (pins OK 2026-09-26 19:54 · R1/R2)
- **Padre:** [ADR-281](281-core-0.9-pins.md) §0.1 slice 2 · contrato [ADR-270](270-lang-contract-set.md)
- **Prev:** slice 1 MAP-ASSIGN (ADR-282) — prereq CLOSED al GO IMPL; este CUT **no** reabre 265/270 (set) ni 277/278 (fn→Result / `?`)
- **Prereq surface:** `v.set(i, x) -> Result<(), Int>` + `Err(0)` OOB + **E0319** + helper `__arita_vec_set` (ADR-265/270) **ya IN** · `fn … -> Result<T,E>` (ADR-277) + `?` desugar (ADR-278) **ya IN**
- **HOLD:** receptor loan `&mut` / params `&mut Vec` (ADR-281 D1) · Mutex · threads · idle · TLS/WS · crates.io · repair · I/O-new · String.set · IndexMut residual — receptor no-Map (String, otros tipos) o sin binding → E0314 (Vec sale a E0344 en este slice) · `?` on Option · `?` in Io main · async Result · surface unwrap/expect (241) · reopen E0272/E0291/E0340/E0341/E0342/E0343

## Objetivo

Unpark **`v[i] = x`** sobre `Vec`/`List` con **cero** path de panic en emit. El sugar es un `?` implícito sobre el set fallible de ADR-265: `v[i] = x` ≡ `v.set(i, x)?`. Por tanto es legal **solo** como statement dentro del body de `fn … -> Result<_, Int>` (ADR-277), donde OOB se propaga como `Err(0)` visible en la firma. Fuera de ese contexto → **E0344** (nuevo) con mensaje canónico `index assign outside result fn` + span del statement `@a..b`, **sin hint** (fix-hint → Backlog; no bloquea el CLOSED). Lit negativo sigue **E0319**. Sin API nueva.

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Desugar** | stmt `v[i] = x` donde `v: Vec<T>`/`List<T>` ⇒ `v.set(i, x)?` (265 + 278) — mismo nodo `IndexAssign` de ADR-282 extendido a receptor Vec/List; diag exacto = `v.set` equivalente (salvo E0344 de contexto) |
| **Ámbito** | **solo** body de `fn … -> Result<U, E>` síncrona (incluye `while`/`if`/`match` anidados dentro de ese body) |
| **OK** | `0 ≤ i < len` → reemplaza en `i`; `len` sin cambio; sigue la ejecución |
| **Err runtime** | `i ≥ len` o `i < 0` no-lit → early-return `Err(0)` (código OOB de ADR-270); `v` **intacto** |
| **Tipo de error** | el de set: **`Int`**, valor `0` (ADR-265/270) — **no** se inventa tipo nuevo |
| **Compat E** | regla ADR-278 (`?`): `E` de la fn debe **unificar** con `Int` (mismo E; **sin** From / coerce nueva). `E ≠ Int` → **E0203** `type mismatch` (lo emite HIR para el azúcar; ADR-281 D4 · revisión Ingeniero 2026-09-26 §2). No se toca el `?` general (ADR-278 cerrado; hueco `g()?` → backlog) |
| **Fuera de ámbito** | `fn main() -> Io<()>` · cualquier fn→Io · fn no-Result · `async fn` (async Result OUT) · bloques `test` · `scenario`/`acceptance` fuera de helper fn→Result → **E0344** `index assign outside result fn` |
| **Lit negativo** | `v[-1] = x` (y cualquier `int_lit < 0`) → **E0319** `negative set index` (aplicación al sugar de set; ADR-281 D3) |
| **Precedencia** | **E0344 > E0319 > E0203** para el azúcar: contexto (**E0344**) > lit negativo (**E0319**) > tipo (**E0203**) — mismo criterio que E0343 «gana sobre ruido de tipo» (ADR-278). `v[-1] = "x"` → **E0319**; `v.set(-1, "x")` sigue dando **E0203** (ADR-265/270, **no** se reabren). Divergencia aceptada solo en programas multi-error y documentada; Measure fija ambos códigos exactos con el par (`v[-1] = "x"` E0319 · `v.set(-1, "x")` E0203). **Entre sentencias** (un solo diag por fn; multi-error → backlog): **gana el primer ofensor en orden fuente**. En fn no-Result, `v[i] = x` antes de `g()?` → **E0344**; `g()?` antes de `v[i] = x` → **E0343** (ADR-278, sin reetiquetar). La precedencia E0344 > E0319 > E0203 rige solo dentro de la misma sentencia `v[i] = x` (visto Ingeniero 23:58). |
| **Receptor / borrow** | binding `let mut` propio **IN** · loan `&mut` y params `&mut Vec` → **HOLD** (ADR-281 D1) · no-mut / loan compartida / loan viva → **E0202** (mismo diag que `v.set` no-mut, ADR-049 §1b) · movido → **E0201** |
| **Evaluación** | orden ≡ `v.set(i, x)`: `i`, luego `x`, luego bounds-check; RHS con `?` (278) que falla retorna **antes** de tocar `v` |
| **R1 — sin `as usize`** (Ingeniero 19:58) | (a) el lowering de `v[i] = x` **no** emite casts `as usize`; conversión vía `usize::try_from(i)` **o** delegando en `__arita_vec_set` · (b) **helpers**: `__arita_vec_set` (ADR-265) **y** `__arita_vec_insert` (ADR-260) en `crates/arita-codegen/src/lib.rs` (`ARITA_VEC_INSERT_HELPER` / `ARITA_VEC_SET_HELPER`, ~L152–175; hoy `i as usize` tras guard `i < 0`) se reescriben con `usize::try_from(i)` → fallo = `Err(0)`. Semántica **idéntica** (OK/`Err(0)`; `set` no crece len; `insert` desplaza) — **no** reabre 260/265 |
| **Parser peek → HIR** (Ingeniero 20:23) | el peek de parser `has_non_map_index_assign` (`crates/arita-syntax/src/lib.rs`) usado por ADR-282 es un **puente temporal**; en este slice (E0344) **toda** la decisión Map/Vec (y contexto fn→Result) pasa a HIR (`HirStmt::IndexAssign`) y el peek se **elimina** |
| **R2 — temporales antes del borrow** | índice y valor (y temporales del RHS) se evalúan **antes** de tomar el `&mut` del Vec (checker ARITA + emit Rust): `v[0] = v.len()` y formas auto-referenciales compilan **sin** E0202 ni error de borrow rustc |
| **Emit** | **nunca** IndexMut ni `v[i] = x` Rust · **nunca** unwrap/expect/panic (241) · **nunca** descartar el resultado del set (`let _ =`, `.ok()`, `unwrap_or`) |
| **OUT** | compuesto `v[i] += x` (`-=`, `*=`…) · anidado `v[i][j] = x` · campo sobre índice `v[i].f = x` → **E0006** `construct outside F1.1 (parse failure)` (decidido; sin gramática nueva en 0.9) · `s[i] = x` String · otros tipos → **E0314** (ADR-281 §0.2) · index-assign como expresión |
| **No reabre** | E0319 texto/código (265/270) · E0342 (277) · E0343 (278) · E0272 · E0340 · E0341 · E0291 |

## 1. Surface ejemplo

```text
// POS — v[i] = x solo en fn → Result<_, Int>; OOB → Err(0) propagado
fn bump(i: Int, x: Int) -> Result<Int, Int> {
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  v.push(2)
  v[i] = x          // ≡ v.set(i, x)?   (i ≥ len → return Err(0))
  let n: Int = v.len()
  Ok(n)
}

fn main() -> Io<()> {
  // PIN: sin v[i] = ni ? aquí — match-convert
  match bump(1, 9) {
    Ok(n) => print(n),
    Err(_) => print("oob")
  }
  match bump(99, 0) {
    Ok(n) => print(n),
    Err(_) => print("oob")     // Err(0) propagado, sin panic
  }
}

// NEG → E0344 (fuera de fn→Result)
fn main() -> Io<()> {
  let mut v: Vec<Int> = Vec::new()
  v.push(1)
  v[0] = 2          // E0344 index assign outside result fn @a..b (sin hint)
}

// NEG → E0319 (lit negativo)          fn f() -> Result<Int, Int> { …; v[-1] = 0; Ok(0) }
// NEG → E0203 (E incompatible, D4)    fn f() -> Result<Int, Text> { …; v[0] = 1; Ok(0) }
// NEG → E0202 (binding no-mut, = set)  fn f() -> Result<Int, Int> { let v: Vec<Int> = Vec::new(); v[0] = 1; Ok(0) }
// NEG → E0006 (sin gramática)           v[0] += 1  ·  v[0][1] = 2  ·  v[0].f = 1
// NEG → E0314 (residual)               s[0] = "x"
```

### 1.1 Emit de referencia (Codegen elige A o B; ambos sin panic)

```text
// A — get_mut + usize::try_from (R1: sin `as usize`; R2: temporales antes del &mut)
{ let __i: i64 = <i>; let __x = <x>;
  let __slot = match usize::try_from(__i) { Ok(__u) => <v>.get_mut(__u), Err(_) => None };
  match __slot {
    Some(__s) => *__s = __x,
    None => return Err(0),
  } }

// B — helper ADR-265 + early-return (≡ desugar `?`); R2: temporales antes del &mut
{ let __i: i64 = <i>; let __x = <x>; __arita_vec_set(&mut <v>, __i, __x)?; }
```

Ambos: cero `IndexMut`, cero `<v>[..] =`, cero unwrap/expect/panic, cero `as usize` (R1) en el lowering **y** en los helpers; `Err(0)` idéntico al de `set`.

### 1.2 Helpers sin `as usize` (R1b — forma de referencia; desviaciones Codegen FASE 1 aceptadas)

```text
fn __arita_vec_set<T>(v: &mut [T], i: i64, x: T) -> Result<(), i64> {
    use std::convert::TryFrom;
    let Ok(u) = usize::try_from(i) else { return Err(0) };
    match v.get_mut(u) { Some(slot) => { *slot = x; Ok(()) } None => Err(0) }
}

fn __arita_vec_insert<T>(v: &mut Vec<T>, i: i64, x: T) -> Result<(), i64> {
    use std::convert::TryFrom;
    let Ok(u) = usize::try_from(i) else { return Err(0) };
    if u > v.len() { return Err(0); }
    v.insert(u, x);
    Ok(())
}
```

Equivalente aceptable si: cero `as usize`, cero unwrap/expect/panic, `i < 0` o fuera de rango → `Err(0)`. Los demás `as usize` del emit (swap/remove/resize/reserve/…) quedan **fuera** de este slice (no se tocan ni se escanean aquí).

**Desviaciones Codegen FASE 1 (revisión Ingeniero 2026-09-26, aceptadas):**

- `__arita_vec_set(&mut [T])` en vez de `&mut Vec<T>` (clippy `ptr_arg`). Correcto: set no cambia la longitud. Llamadas emitidas y comportamiento iguales; lo cubre helpers-semantics-unchanged. `__arita_vec_insert` debe seguir con `&mut Vec<T>` porque sí la cambia.
- `use std::convert::TryFrom;` dentro de cada helper por la edición 2015 de `arita build`. OK. Emit-clippy debe seguir limpio en 2015 y 2021 (sin `unused_imports` en 2021).

## 2. Oracles

Requeridos (a medir al GO IMPL; **ninguno** medido en este DOC):

| Id | Expect |
|----|--------|
| `core09-vec-assign-ok` | in-bounds `v[i] = x` en fn→Result → Ok + valor leído vía `get`/`[]` → stdout esperado |
| `core09-vec-assign-oob-err` | OOB (`i ≥ len`) → `Err(0)` propagado → caller/main match fail determinista (no panic); `v` intacto |
| `core09-vec-assign-neg-nonlit` | `i < 0` no-lit → `Err(0)` propagado (no panic) |
| `core09-vec-assign-eq-set` | golden: `v[i] = x` ≡ `v.set(i, x)?` (mismo stdout Ok/Err) |
| `core09-vec-assign-self-ref` | **POSITIVO**: `v[0] = v.len()` (y temporales que leen `v`) en fn→Result → compila y ejecuta sin E0202 ni error rustc (R2) |
| `core09-vec-assign-qmark-chain` | `v[i] = x` + `?` (278) en la misma fn → primer Err corta cadena |
| `neg-core09-vec-assign-outside-main` | `v[i] = x` en `fn main() -> Io<()>` → **E0344** |
| `neg-core09-vec-assign-outside-fn` | `v[i] = x` en fn no-Result → **E0344** (fixture de superficie); `test` → **E0344** solo vía test HIR directo (`test {}` rechaza `let`; ver Backlog) |
| `neg-core09-vec-assign-qmark-chain` | `v[i] = x` seguido de `let n = g()?` (g: fn→`Result<_, Int>`) en la misma fn **no-Result**, con `v[i] = x` primero en orden fuente → **E0344** (pin 23:50, visto Ingeniero 23:58: gana el primer ofensor en orden fuente; par inverso en `neg-core09-vec-assign-qmark-chain-rev`). `g()?` con `E ≠ Int` sigue en Backlog (rustc E0277) y **no** lleva neg |
| `neg-core09-vec-assign-qmark-chain-rev` | `let n = g()?` (g: fn→`Result<_, Int>`) **antes** de `v[i] = x` en la misma fn **no-Result** → **E0343** exacto (código existente ADR-278, sin reetiquetar; par de `-qmark-chain`, regla orden fuente; visto Ingeniero 23:58) |
| `neg-core09-vec-assign-neg-lit` | `v[-1] = x` en fn→Result → **E0319** |
| `neg-core09-vec-assign-neg-lit-vs-set` | par Measure (revisión Ingeniero 2026-09-26 ítem 1): `v[-1] = "x"` → **E0319** · `v.set(-1, "x")` → **E0203** (ADR-265/270, **no** se reabre) — ambos códigos exactos fijados para que ninguno cambie sin que nos enteremos; divergencia aceptada solo en programas multi-error (§0 Precedencia) |
| `neg-core09-vec-assign-err-type` | fn→Result con `E ≠ Int` → **E0203** (HIR; D4 · revisión Ingeniero 2026-09-26 §2) |
| `neg-core09-vec-assign-non-mut` | binding no-mut / loan compartida → **E0202** (= `v.set`) |
| `neg-core09-vec-assign-compound` | `v[i] += x` → **E0006** exacto (build OK = Rejected); anidado `v[i][j] = x` / campo `v[i].f = x` = misma regla **E0006** |
| `neg-core09-vec-assign-unwrap` | ligar `let r: Result<(), Int> = v.set(i, x)` y `r.unwrap()` / `r.expect(…)` (mismo código que los negs invent-unwrap existentes) → **E0206** `method not in F2 std whitelist` exacto (pin nuevo 23:50; el oracle exige solo E0206 y **no** acepta E0342, igual que `neg-core09-map-assign-unwrap`; la forma encadenada `v.set(i, x).unwrap()` no se usa en el fixture); ≠ E0291 reopen |
| `neg-core09-vec-assign-bad-return` | fn→Result que muta y retorna payload no-Result (`v[0] = 1; 0`) → **E0342** (sin reetiquetar) |
| `neg-core09-vec-assign-err-swallow` | caller `match f() { …, Err(_) => <lit éxito> }` → **E0272** `result error swallowed` exacto (pin nuevo 23:50; ADR-048 · mismo código que `neg-e0272-err-default-lit`). E0223 solo aplica si el patrón no casa con el tipo del scrutinee, y el fixture no lo hace — **no** reopen H4 |
| `core09-vec-assign-emit-ban` | emit grep: cero `\w+\[[^\]]+\]\s*=[^=]` Rust (IndexMut-style), cero `IndexMut`/`std::ops::Index`, cero `.unwrap()`/`.expect(`/`panic!`, cero `let _ = __arita_vec_set` / `.ok()` sobre set; presencia de `get_mut(`+`usize::try_from` o `__arita_vec_set(…)?`; build sin warnings |
| `core09-vec-assign-emit-no-as-usize` | emit grep (R1): cero `as usize` en el lowering de `v[i] = x` **y** en los cuerpos de `__arita_vec_set` y `__arita_vec_insert`; presencia de `usize::try_from` en ambos helpers (cubre programas que usan `set`, `insert` y `v[i] = x`) |
| `core09-vec-helpers-semantics-unchanged` | regresión: oráculos `set` (265) e `insert` (260) existentes siguen con la misma expectativa (OK / `Err(0)` / E0319 / E0311) tras reescribir helpers |

**Discard (ADR-281 D2 → backlog must-use):** descartar el `Result` de una fn→Result que usa `v[i] = x` (`let _ = f()`) o de `v.set` bare **no** tiene diag en 0.9. **Sin E0345**; E0340 (solo host IO) **no** se reetiqueta; sin oracle en este slice.

**Migración (opción A, ADR-281 §0.3):** mismos ids y archivos; expect E0314 → **E0344**: `neg-core05-index-mut` · `neg-core05-scen-coll-index-mut` · `neg-core05-ref-coll-index-mut` · `neg-core06-index-mut-assign` · `neg-core06-scen-gp-index-mut` · `neg-core06-ref-gp-index-mut` · `neg-e0310-index-vec`. `neg-e0310-index-string` sin cambio (E0314). Edición en IMPL de este slice; **no** reabre cierre de 261/262/263/265/267/268.

### 2.1 Marcador `<arita:deferred-shape>` (revisión Ingeniero 2026-09-26 ítem 3)

Se acepta el diseño (HIR decide E0344/E0314 o devuelve el E0001/E0006 diferido), con tres condiciones para el CLOSED:

- `arita parse` **debe fallar por sí solo** si el AST resultante contiene un marcador: reporta el código diferido del parser (E0001/E0006), que era su comportamiento previo. Ningún programa inválido puede salir de `parse` con exit 0.
- El marcador no puede llegar nunca al Rust emitido: oráculo emit-ban de `arita:deferred-shape` sobre los positivos, y codegen rechaza con error (no panic) si lo ve.
- Test que demuestre que no se puede escribir desde la superficie (no es un identificador válido y da error de parse).

## 3. Pin E0344

**E0344** `index assign outside result fn` — statement `v[i] = x` sobre `Vec`/`List` fuera del body de una función síncrona cuyo tipo de retorno es `Result<_,_>` (p.ej. `fn main() -> Io<()>`, fn no-Result, `async fn`, `test`).

- **Emisión:** mensaje canónico `index assign outside result fn` + span del statement `@a..b`, **sin hint**. La ausencia de hint **no bloquea el CLOSED**; el fix-hint pasa a Backlog (Ingeniero 2026-09-26 22:20, backlog aceptado).
- **Dueño:** este ADR (283).
- **No** reasignar E0343 / E0342 / E0319 / E0314 / E0272 / E0340 / E0341 / E0291.
- Nota: E0344 aparece hoy solo en una aserción negativa de test HIR (Option? → E0203, `!starts_with("E0344")`); sin significado previo.

## 4. Qué NO tocar

| Prohibido | Motivo |
|-----------|--------|
| `?` in Io main · Option? · async Result | OUT (278) |
| IndexMut residual (receptor no-Map (String, otros tipos) o sin binding → E0314; Vec → E0344 aquí) · String.set | HOLD (E0314) |
| Raw `IndexMut` / `v[i] = x` Rust en emit | panic path — prohibido |
| Nuevo tipo de error OOB / From magic | `Err(0): Int` es SoT (270) |
| Mutex / threads / idle / TLS/WS / crates.io / repair / I/O-new | HOLD global |
| Surface unwrap/expect | emit-ban 241 HOLD |
| Receptor loan `&mut` · params `&mut Vec` · widen `fn_param` | HOLD (ADR-281 D1) |
| Casts `as usize` en el lowering y en `__arita_vec_set` / `__arita_vec_insert` | R1 |
| GO IMPL antes CLOSED 282 + OK Ingeniero | gate Orquestador |
| Inventar PASS / Lex N/N | DOC only |

## 5. Criterio CLOSED

Lex §2 verde (N/N lo fija Ingeniero al GO IMPL); tick ADR-281 slice 2; migración §0.3 aplicada; HOLDs §4 intactos; condiciones §2.1 del marcador (ítem 3); ningún fixture existente cambia de código fuera de la migración declarada (ítem 4, lo verifica Measure en el full run). Siguiente: slice 3 SCENARIO-MUT ([ADR-284](284-core-scenario-mut-v0.md)).

## Backlog (fuera de Core 0.9)

- Hueco preexistente de ADR-278: `g()?` con `E` incompatible pasa el frontend y acaba en rustc **E0277**. Fuera de Core 0.9; **no** reabre ADR-278 (cerrado). El azúcar `v[i] = x` con `E ≠ Int` no depende de esto: HIR emite **E0203** (§0 Compat E). (Revisión Ingeniero 2026-09-26 §2)
- Caso borde multi-error (main sin print + otro error HIR antes de `v[i]=x` da E0001): se acepta y va a **backlog** (orden de diagnósticos multi-error), con la condición de que **ningún** fixture existente cambie de código fuera de la migración declarada (7 negs Vec a E0344). Measure lo verifica en el full run. (revisión Ingeniero 2026-09-26 ítem 4)
- Fix-hint de **E0344** (antes pineado en §3; texto EN de referencia: `` `v[i] = x` can fail (index out of bounds → Err(0)); here use `match v.set(i, x) { Ok(()) => …, Err(_) => … }`, or move the assignment into a `fn … -> Result<_, Int>` ``). Encaja con el pin aparcado del scout de diag JSON `rationale`/`fix`/`spec_ref`. (Ingeniero 2026-09-26 22:20, backlog aceptado)
- `test {}` rechaza `let`: E0344 dentro de `test` queda cubierto solo por tests HIR directos, no por fixtures de superficie. (Ingeniero 2026-09-26 22:20, backlog aceptado)
- `fn f(v: Vec<Int>)` no parsea: params colección siguen en HOLD (ADR-281 D1). (Ingeniero 2026-09-26 22:20, backlog aceptado)

## Checklist

- [x] Pins `v[i] = x` ≡ `v.set(i, x)?` + E0344 + E0319 + emit A/B + oracles (GO-listo DOC)
- [x] Revisión Ingeniero 2026-09-26 19:54 — pins OK · opción A (7 negs → E0344) · D1 HOLD · R1/R2 · sin E0345
- [x] Ingeniero 20:23 — peek `has_non_map_index_assign` = puente temporal de 282; en slice 2 decisión Map/Vec → HIR y el peek se elimina
- [x] Limpieza pre-gate 2026-09-26 — compuesto/anidado/campo = E0006 decidido; E0314 solo String · sin binding (Vec → E0344 aquí) «superado (→ receptor no-Map (Vec hasta 283, String, otros tipos) o sin binding → E0314, Arquitecto rev. 2026-09-26)»
- [x] Decisión **Arquitecto 2026-09-26** (alineada con HIR `HirStmt::IndexAssign`, sin cambio IMPL): E0314 residual = receptor no-Map (Vec hasta 283, String, otros tipos) o sin binding → E0314; en este slice Vec sale a E0344
- [x] Ingeniero 19:58 — R1 cubre helpers `__arita_vec_set` + `__arita_vec_insert` (`usize::try_from` → `Err(0)`) + oracle `core09-vec-assign-emit-no-as-usize` · compuesto E0006
- [x] Revisión Ingeniero 2026-09-26 §2 (`DOC/reviews/INGENIERO-ADR283-DECISIONS-20260926.md`) — `E ≠ Int` en el azúcar = **E0203** (lo emite HIR; sustituye «diag vigente de `?` con E incompatible»); backlog: `g()?` con E incompatible → rustc E0277 (hueco preexistente de 278, fuera de 0.9, 278 no se reabre); precedencia E0344 > E0319 > E0203 explícita; `v.set(-1, "x")` sigue E0203 (265/270 no se reabren)
- [x] Revisión Ingeniero 2026-09-26 ítems 1/3/4 + desviaciones Codegen FASE 1 (`DOC/reviews/INGENIERO-ADR283-DECISIONS-20260926.md`) — oracle par E0319/E0203 (§2) · condiciones marcador `<arita:deferred-shape>` para el CLOSED (§2.1) · backlog multi-error sin cambio de fixtures fuera de los 7 negs (Backlog) · §1.2: `__arita_vec_set(&mut [T])`, `__arita_vec_insert(&mut Vec<T>)`, `use std::convert::TryFrom;` por helper, emit-clippy limpio en 2015 y 2021 · alineación con cabecera GO IMPL (ítem 5): HOLD de L10, checklist y Cierre
- [x] Ingeniero 2026-09-26 22:20 (aprobación escrita, backlog aceptado) — E0344 se emite con mensaje canónico `index assign outside result fn` + span `@a..b`, **sin hint** (§3, L14, ejemplo §1); la falta de hint no bloquea el CLOSED · Backlog: fix-hint E0344 (con pin aparcado diag JSON rationale/fix/spec_ref) · `test {}` rechaza `let` (E0344 en tests solo vía tests HIR) · `fn f(v: Vec<Int>)` no parsea (HOLD ADR-281 D1) · oráculo outside-fn alineado (Arquitecto)
- [x] 2026-09-26 23:50 · **PIN NUEVO (Arquitecto), visto Ingeniero 23:58:** unwrap → E0206 exacto (sin E0342) · err-swallow → E0272 exacto (E0223 no aplica) · neg qmark-chain → E0344 por orden fuente · `g()?` E≠Int sigue backlog · E0344/E0203/E0202/E0342 sin cambio
- [x] 2026-09-26 23:58 · regla «primer ofensor en orden fuente» (E0344/E0343) escrita en §0 · oracle inverso `neg-core09-vec-assign-qmark-chain-rev` → E0343 (código existente) · condición Ingeniero cumplida
- [x] GO IMPL (Ingeniero 2026-09-26, tras CLOSED ADR-282)
- [ ] IMPL + Lex CLOSED

## Cierre

- pendiente: measure N/N + Veyra + OK Ingeniero
