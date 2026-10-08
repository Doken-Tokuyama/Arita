# ADR-282 — Core 0.9 slice 1: MAP-ASSIGN

- **Estado:** **CLOSED** Lex **792/792** (Ingeniero 2026-09-26 · [`GATE-CORE09-MAP-ASSIGN-20260926.md`](../GATE-CORE09-MAP-ASSIGN-20260926.md) · Veyra Proof **ACCEPTED** `.veyra/evidence/20260926T193713Z/`) · CUT `CORE-0.9-MAP-ASSIGN-20260926` · pins OK Ingeniero (19:54 · 19:58 · A6) + Orquestador 19:59
- **CUT-ID:** `CORE-0.9-MAP-ASSIGN-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero (pins OK + GO IMPL 2026-09-26; decisiones 19:58 + corrección A6) · Orquestador (nodo `IndexAssign` / emit 19:59)
- **Padre:** [ADR-281](281-core-0.9-pins.md) §0.1 slice 1
- **Prev:** Core **0.8 CLOSED** (ADR-276/280) — este CUT **no** reabre 276–280 ni 264–268
- **Prereq surface:** `Map<Text|String, Int>` + `m.put(k, v)` (ADR-237; su emit **no** se toca) **ya IN** · `m[k]` → `Option` (ADR-266) **ya IN** · receptor exclusivo no-mut / loan → **E0202** (ADR-049 §1b) **ya IN**
- **HOLD:** Mutex · threads · idle · TLS/WS · crates.io · repair · I/O-new · String.set · IndexMut residual — receptor no-Map (Vec hasta 283, String, otros tipos) o sin binding → E0314 (ADR-281 §0.2) · `v[i] = x` (slice 2 ADR-283) · receptor loan `&mut` / params `&mut Map` (ADR-281 D1) · asignación compuesta / anidada / campo sobre índice (sin gramática; E0006) · relajar args de `put` (backlog) · surface unwrap/expect (241) · Option? · `?` in Io main · reopen E0272/E0291/E0340/E0341/E0342/E0343

## Objetivo

Unpark **`m[k] = v`** sobre `Map` vía **nodo propio `IndexAssign`** cuya semántica y checks son los de `m.put(k, v)` (ADR-237): escritura **total** (insert/overwrite), sin panic posible. **Regla base (Ingeniero 19:58):** en **todo** caso de error el azúcar da el **mismo diag exacto** que el `put` equivalente. **Única excepción** (corrección A6): la restricción literal/ident de los args de `put` es **gramatical** (E0006) y el azúcar **no** la hereda → RHS = cualquier expresión v0 y el self-ref es **positivo**. Receptor: **solo** binding `let mut` propio (loans / params `&mut Map` **HOLD**, D1). Sin API nueva; sin código nuevo; **sin gramática nueva** (compuesto / anidado / campo sobre índice siguen E0006).

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Nodo** | `IndexAssign` propio (Orquestador 19:59) para la forma ya parseada `ident[key] = rhs` cuando el receptor es `Map`; invoca los checks de `put` (mutabilidad, loans, tipos). Receptor no-Map (Vec hasta 283, String, otros tipos) o sin binding → **E0314** desde HIR con span `@a..b` (§0.2); los 9 ids E0314 vigentes siguen E0314 en este slice |
| **Equivalencia con `put`** (Ingeniero 19:58/19:59) | (a) misma semántica y mismo stdout en los programas que **ambas** formas aceptan · (b) mismo diag **exacto** (y mismo texto de mensaje) para mutabilidad (**E0202**), loans (**E0202**) y tipos (**E0203**) · (c) oracle eq-put exige `insert(` en el emit — **no** emit exacto |
| **RHS** | **cualquier** expresión v0 de `assign_rhs` (ident, literal, `m.len()`, aritmética, `f(x)`, `f(x)?` en fn→Result…). **No** hereda el límite de args de `put` (`method_push_arg` literal/ident; fuera → E0006, límite de grammar). Relajar `put` → **backlog** |
| **Semántica** | clave ausente → inserta; presente → sobrescribe; valor viejo **descartado** (statement-level; no es `Result`, no hay discard theater) |
| **Total** | sin `Result`, sin OOB, sin panic; legal en **cualquier** contexto (main `Io<()>`, fn→Result, fn no-Result, `test`) |
| **Tipos v0** | `K` ∈ {`Text`, `String`}; `V` = `Int` (ADR-237). Clave/valor de tipo incorrecto → **E0203** `type mismatch` (= `put`; **no** mint) · Map clave numérica sigue HOLD |
| **Receptor** | binding `let mut m` propio **IN** · loan `borrow mut m` / `&mut m` y params `&mut Map` → **HOLD** (ADR-281 D1) · `fn_param` **no** se amplía |
| **Borrow negs** (= `put`) | binding no-`mut` → **E0202** `borrow conflict` (ADR-049 §1b) · escribir vía loan compartida (`borrow m` / `&m`) → **E0202** · loan viva sobre `m` → **E0202** · `m` movido → **E0201** `use of moved value` |
| **R2 — temporales antes del borrow** | key y value (y cualquier temporal del RHS) se **evalúan antes** de tomar el borrow `&mut` del map, en el checker ARITA y en el emit Rust: formas auto-referenciales (`m["b"] = m.len()`, key/value que leen `m`) compilan **sin** E0202 ni E0502 rustc |
| **R3 — emit statement clippy-clean** | statement desnudo `m.insert(k, v);` precedido de temporales R2 (Orquestador 19:59); forma actual Codegen: `{ let __arita_mk = <k>; let __arita_mv = <v>; m.insert(__arita_mk, __arita_mv); }`; `cargo clippy -- -D warnings` verde sobre el emit |
| **Emit ban** | **cero** `m[k] = …` Rust, **cero** `Index`/`IndexMut`, **cero** unwrap/expect/panic (241) |
| **Evaluación** | orden: `k`, luego `v` (R2: ambos a temporales), luego insert |
| **OUT** | compuesto `m[k] += v` (`-=`, `*=`…) · anidado `m[k][j] = v` / `v[i][j] = x` · campo sobre índice `m[k].f = v` / `v[i].f = x` → **E0006** `construct outside F1.1 (parse failure)` (decidido; sin gramática nueva en 0.9; Ingeniero A3) · index-assign como expresión · `Map.set` numérico · cambiar `put` |
| **No reabre** | ADR-266 (read sugar) · ADR-237 (`put` y su emit) · ADR-049 · E0272/E0340/E0341/E0342/E0343 |

### 0.1 Nota — emit del azúcar ≠ emit de `put` (esperado; `put` no se toca)

El azúcar emite el statement `m.insert(k, v);` (con temporales R2), **no** la forma ADR-237 de `put` (`{ let _ = m.insert(k, v); }`). Es **esperado** y no rompe la equivalencia: `core09-map-assign-eq-put` compara **stdout + diag + presencia de `insert(`**, nunca emit exacto. La ruta `put` **no** cambia (regresión `core09-map-assign-put-emit-unchanged`).

### 0.2 E0314 desde HIR — span `@a..b` + tests

Con el nodo `IndexAssign`, la decisión Map vs no-Map se toma en HIR tras tipar (`HirStmt::IndexAssign`, `crates/arita-hir/src/lib.rs`): receptor no-Map (Vec hasta 283, String, otros tipos) o sin binding → **E0314** con el texto pre-282 y el **span del statement** en el formato del parser: `E0314: IndexMut assign not allowed (v[i]= / m[k]= HOLD) @a..b`.

- Tests existentes en `crates/arita-hir/src/lib.rs`: `adr282_e0314_carries_index_assign_stmt_span` (mensaje exacto con `@start..end` y `&src[start..end] == "v[0] = 1"`) · `adr282_vec_string_unbound_stay_e0314` (Vec, String y target sin binding → E0314; «otros tipos» siguen la misma rama HIR — sin test dedicado visto en `crates/`, test requerido en IMPL si Measure lo pide).
- Puente temporal en parser: `has_non_map_index_assign` (`crates/arita-syntax/src/lib.rs`) solo conserva la precedencia pre-282 frente a los gates de forma (E0001 / E0006 non-main) y devuelve el span para `@a..b`; HIR sigue siendo la autoridad. Se **elimina** en slice 2 (ADR-283, Ingeniero 20:23).

## 1. Surface ejemplo

```text
// POS — insert / overwrite (cualquier contexto); RHS expr v0
fn main() -> Io<()> {
  let mut m: Map<Text, Int> = Map::new()
  m["a"] = 7                 // insert
  m["a"] = 9                 // overwrite (7 descartado)
  m["b"] = m.len()           // R2: auto-referencial — len evaluado antes del insert (positivo)
  let c: Int = 2
  m["c"] = c + 1             // aritmética en RHS (no hereda límite de put)
  match m["a"] { Some(x) => print(x), None => print(0) }   // 9
}

// NEG → E0202 (binding no-mut; mismo diag y texto que m.put)
fn main() -> Io<()> {
  let m: Map<Text, Int> = Map::new()
  m["a"] = 1
  print(m.len())             // print obligatorio en fixtures neg (sin print → E0001 antes)
}

// NEG → E0202 (loan compartida / loan viva)   let r = borrow m ; r["a"] = 1
// NEG → E0203 (tipo)                          m[1] = 2   ·   m["a"] = "x"
// NEG → E0006 (sin gramática)                 m["a"] += 1   ·   m["a"][0] = 1   ·   m["a"].f = 1
// NEG → E0206 (invent unwrap)                 let o: Option<Int> = m["a"] ; o.unwrap()
```

## 2. Oracles

Requeridos (a medir en IMPL; **ninguno** medido en este DOC). Todo neg exige **código exacto** (nunca «cualquier error»); fixtures neg llevan `print`.

| Id | Expect |
|----|--------|
| `core09-map-assign-insert` | `m[k] = v` clave nueva → `m[k]`/`get` = `Some(v)` → stdout esperado |
| `core09-map-assign-overwrite` | segundo `m[k] = w` → `Some(w)`; `len` sin cambio |
| `core09-map-assign-eq-put` | programa aceptado por ambas formas: `m[k] = v` ≡ `m.put(k, v)` → mismo stdout + mismo diag + `insert(` presente en ambos emits (**no** emit exacto, §0.1) |
| `core09-map-assign-fn-result` | `m[k] = v` dentro de `fn → Result` + `?` (278) → OK sin cambio de semántica |
| `core09-map-assign-self-ref` | **POSITIVO** (compila y ejecuta): `m["b"] = m.len()` (y key/value que leen `m`) → build OK, sin E0202 ni E0502 rustc; stdout lo fija Codegen/Measure (R2; sin gemelo `put`: `put` da E0006 gramatical) |
| `neg-core09-map-assign-non-mut` | binding `let m` (no-mut) → **E0202** exacto, mismo texto que gemelo `m.put` |
| `neg-core09-map-assign-shared-loan` | escribir vía `borrow m` / `&m` → **E0202** exacto (= gemelo `put`; loans HOLD D1; medido, nunca Inconclusive) |
| `neg-core09-map-assign-live-loan` | `m[k] = v` con loan viva de `m` usada después → **E0202** exacto (= gemelo `put`) |
| `neg-core09-map-assign-type` | clave `Int` / valor `Text` → **E0203** exacto (= gemelo `put`) |
| `neg-core09-map-assign-compound` | `m["a"] += 1` → **E0006** exacto (`construct outside F1.1 (parse failure)`); build OK = Rejected (anidado / campo sobre índice = misma regla E0006) |
| `neg-core09-map-assign-unwrap` | ligar a `Option` y `o.unwrap()` / `o.expect(…)` → **E0206** exacto (`method not in F2 std whitelist`; = negs invent-unwrap existentes); ≠ E0291 reopen. Forma encadenada `m["a"].unwrap()` da E0006 → el fixture **no** encadena |
| `core09-map-assign-emit-ban` | emit grep: `insert(` presente; cero `\w+\[[^\]]+\]\s*=[^=]` Rust, cero `IndexMut`/`std::ops::Index`, cero `.unwrap()`/`.expect(`/`panic!` |
| `core09-map-assign-emit-clippy` | `cargo clippy -- -D warnings` verde sobre el emit (R3; Ingeniero A1); clippy ausente → Inconclusive (nunca PASS) |
| `core09-map-assign-put-emit-unchanged` | regresión ADR-237: emit de `put` (`ejemplos/core01/07-map-put-get.arita`) idéntico a baseline — el nodo `IndexAssign` no altera la ruta `put` |

`E0201` (movido) sin oracle propio en este slice.

**Migración (opción A, ADR-281 §0.3) — cambio atómico (fixture + runner + smoke + IMPL):** `neg-core06-map-index-mut` — **mismo id**, sigue neg; el archivo `ejemplos/core06/map-index/neg/01-index-mut.arita` pasa a binding **no-`mut`** (`let m`), sin línea `m.put`, con `print` → expect **E0202** exacto. **No** se convierte en positivo. Los 9 ids E0314 (Vec/String/unknown-field) **siguen E0314** en este slice.

## 3. Diags

Sin código nuevo. Reusa con texto canónico vigente: **E0202** `borrow conflict` (no-mut · loan compartida · loan viva · vía `&`) · **E0201** `use of moved value` · **E0203** `type mismatch` · **E0006** `construct outside F1.1 (parse failure)` (compuesto · anidado · campo sobre índice) · **E0206** `method not in F2 std whitelist` · **E0314** (receptor no-Map (Vec hasta 283, String, otros tipos) o sin binding → E0314; §0.2). Versiones previas con E0205 para no-mut/loans: «superado (→ E0202, rev. 2026-09-26)». **Sin E0345.** **No** reasignar E0272 / E0340 / E0341 / E0342 / E0343.

## 4. Qué NO tocar

| Prohibido | Motivo |
|-----------|--------|
| `v[i] = x` Vec | slice 2 ADR-283 (HOLD hasta CLOSED 282) |
| IndexMut residual (receptor no-Map (Vec hasta 283, String, otros tipos) o sin binding → E0314) · String.set | HOLD (E0314) |
| Gramática de asignación compuesta / anidada / campo sobre índice | sin gramática nueva en 0.9 (E0006) |
| Receptor loan `&mut` · params `&mut Map` · widen `fn_param` | HOLD (ADR-281 D1) |
| Relajar args de `put` · cambiar emit de `put` | backlog / regresión ADR-237 |
| Mutex / threads / idle / TLS/WS / crates.io / repair / I/O-new | HOLD global |
| Surface unwrap/expect | emit-ban 241 HOLD |
| Inventar PASS / Lex N/N | DOC only |

## 5. Criterio CLOSED

Lex §2 verde (N/N lo fija Ingeniero en el gate con el conteo real); clippy emit verde (R3); migración `neg-core06-map-index-mut` aplicada (E0202, mismo id); 9 ids E0314 intactos; tick ADR-281 slice 1; HOLDs §4 intactos. Siguiente: slice 2 VEC-ASSIGN ([ADR-283](283-core-vec-assign-v0.md)) — GO IMPL tras CLOSED 282.

## Checklist

- [x] Pins `m[k] = v` ≡ `put` + borrow negs + oracles (GO-listo DOC)
- [x] Revisión Ingeniero 2026-09-26 19:54 — pins OK · opción A · D1 HOLD · R2/R3
- [x] Decisiones Ingeniero 19:58 + corrección A6 · Orquestador 19:59 — nodo `IndexAssign` · RHS expr v0 · self-ref positivo · diag exacto = `put` (E0202/E0203) · compuesto E0006 · unwrap E0206 · emit `m.insert(k, v);` · relajar `put` → backlog
- [x] Rev. Measure prep (E0205 «superado (→ E0202, rev. 2026-09-26)» · compuesto E0006 · nota emit ≠ `put`)
- [x] Limpieza pre-gate 2026-09-26 (Measure 21:29 + Codegen): cuerpo sin E0205 / sin requisito de descarte explícito / E0314 solo Vec·String·sin binding «superado (→ receptor no-Map (Vec hasta 283, String, otros tipos) o sin binding → E0314, Arquitecto rev. 2026-09-26)» · anidado/campo = E0006 decidido · §0.2 E0314 HIR span + tests
- [x] Decisión **Arquitecto 2026-09-26** (alineada con HIR `HirStmt::IndexAssign`, sin cambio IMPL): E0314 residual = receptor no-Map (Vec hasta 283, String, otros tipos) o sin binding → E0314
- [x] GO IMPL (CUT `CORE-0.9-MAP-ASSIGN-20260926`)
- [ ] IMPL + Lex CLOSED

## Cierre

- **GO IMPL** activo · IMPL/Lex pendientes · nada medido en este DOC.
