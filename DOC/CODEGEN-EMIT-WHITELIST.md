# CODEGEN — Emit whitelist de métodos (AI emitters)

- **Estado:** **REVIEW / HOLD IMPL** (2026-09-18) — docs-only; **no** land crates sin GO.
- **Autores:** ARITA Codegen (audit) · <person> + Ingeniero (request HOLD).
- **SoT código (box `/workspace/ARITA`; el gate local `<repo>`):**
  - Emit: `crates/arita-codegen/src/lib.rs` → `emit_method_call` **L634–737**
  - HIR gate: `crates/arita-hir/src/lib.rs` → `type_of_method_call` **L760–1257**; loans **L472–473**
- **ADRs:** 026 (std minima), 045 (overflow / checked OUT v0), 046 (traps arith), 049 (clear), 050 (Option), 052 (pop; `get`/`remove` OUT DOC).
- **Gobernanza:** Safe-only (ADR-022). Emit falla → `None` (stmt omitido / expr falla cadena); HIR fuera whitelist → **E0206**. No inventar PASS.

## 1. Contrato para AI emitters

Plantilla canónica (una fila = un método whitelisted):

| Columna | Significado |
|---------|-------------|
| **method** | Identificador ARITA (string match en emit/HIR) |
| **receiver** | Tipo HIR permitido |
| **arity** | Nº args; mismatch HIR → **E0203** |
| **ret** | Tipo HIR retorno |
| **loan** | `shared` / `exclusive` / (ninguno si no place) — HIR L472–473 |
| **emit Rust** | Fragmento literal que debe producir codegen (o `None` hoy) |
| **Copy/move** | Notas para el emitter (no mover `mut` place; casts `usize`/`i64`) |

Regla: **HIR acepta ≠ emit existe**. Si HIR OK y emit `None`, el programa puede typecheck y **desaparecer** la llamada en Rust (silent drop) — riesgo crítico (ver §5).

---

## 2. Tabla aritmética `checked_*` / `saturating_*` / `wrapping_*`

**Fuente HIR:** `type_of_method_call` L1055–1202. **Fuente emit:** `emit_method_call` — **ninguna** rama; caen en `_ => None` **L736**.

ADR-045 §2.3: *OUT v0: wrapping APIs intencionales; checked_add surface* — surface HIR ya existe en código; **emit aún no**; DOC ADR no tiene CUT dedicado landed para estos métodos.

| method | HIR recv | arity | HIR ret | loan (HIR) | emit Rust fragment (hoy) | Copy/move notes | Proposed emit (HOLD — no land) |
|--------|----------|-------|---------|------------|--------------------------|-----------------|--------------------------------|
| `checked_add` | `Int` | 1×`Int` | `Option<Int>` | shared | **not present in emit yet** (`None`) | `i64` is Copy; Rust `i64::checked_add` by value | `{recv}.checked_add({arg})` |
| `checked_sub` | `Int` | 1×`Int` | `Option<Int>` | shared | **not present** | idem | `{recv}.checked_sub({arg})` |
| `checked_mul` | `Int` | 1×`Int` | `Option<Int>` | shared | **not present** | idem | `{recv}.checked_mul({arg})` |
| `checked_div` | `Int` | 1×`Int` | `Option<Int>` | shared | **not present** | div0 → `None` (no panic); distinto de op `/` E0216 | `{recv}.checked_div({arg})` |
| `checked_rem` | `Int` | 1×`Int` | `Option<Int>` | shared | **not present** | idem rem | `{recv}.checked_rem({arg})` |
| `saturating_add` | `Int` | 1×`Int` | `Int` | shared | **not present** | Copy; no Option | `{recv}.saturating_add({arg})` |
| `saturating_sub` | `Int` | 1×`Int` | `Int` | shared | **not present** | idem | `{recv}.saturating_sub({arg})` |
| `saturating_mul` | `Int` | 1×`Int` | `Int` | shared | **not present** | idem | `{recv}.saturating_mul({arg})` |
| `wrapping_add` | `Int` | 1×`Int` | `Int` | shared | **not present** | Copy; wrap silencioso intencional (contraste E0217 en `+`) | `{recv}.wrapping_add({arg})` |
| `wrapping_sub` | `Int` | 1×`Int` | `Int` | shared | **not present** | idem | `{recv}.wrapping_sub({arg})` |
| `wrapping_mul` | `Int` | 1×`Int` | `Int` | shared | **not present** | idem | `{recv}.wrapping_mul({arg})` |

**Ausentes en HIR y emit (no inventar):** `saturating_div`, `wrapping_div`/`rem`, `checked_neg`, `wrapping_neg`, `saturating_pow`, etc. → E0206 si se llaman.

---

## 3. Whitelist emit actual (éxito vs `None`)

### 3.1 Emit **OK** (`emit_method_call` match arms L637–735)

| method | emit Rust fragment (código real) | arity gate en emit | Copy/move / notas |
|--------|----------------------------------|--------------------|-------------------|
| `push` | `{recv}.push({args…})` L638–643 | **no** chequea arity en emit | exclusive loan HIR; requiere `mut` (E0202) |
| `push_str` | LitStr → `{recv}.push_str("…")`; else `{recv}.push_str(&{a})` L645–658 | arity ≠1 → `None` | exclusive; String mut; `&` en no-lit |
| `clear` | `{recv}.clear()` L660–664 | args no vacío → `None` | exclusive; Vec\|String mut (ADR-049) |
| `pop` | `{recv}.pop()` L666–670 | args no vacío → `None` | exclusive; ret `Option<T>` (ADR-052); solo Vec en HIR |
| `clone` | `{recv}.clone()` L672–676 | args no vacío → `None` | shared; String\|Vec (no move out del place) |
| `contains` | LitStr → `.contains("…")`; else `.contains(&{a})` L678–691 | arity ≠1 → `None` | shared; String o Vec\<T\> |
| `starts_with` | espejo contains L693–706 | arity ≠1 → `None` | shared; String |
| `ends_with` | espejo L708–721 | arity ≠1 → `None` | shared; String |
| `len` | `({recv}.len() as i64)` L723–728 | args no vacío → `None` | shared; **bytes** String; Int=`i64` (ADR-026) |
| `is_empty` | `{recv}.is_empty()` L730–734 | args no vacío → `None` | shared |

Cualquier otro `method` → **`None`** L736 (incl. familia arith, Option helpers, `get`/`remove`, String trim/replace, Int `abs`/`pow`/…).

### 3.2 HIR whitelist **sin** emit (gap — HOLD)

Métodos tipados en HIR (`type_of_method_call`) que **hoy** emit rechaza (`None`):

| method | HIR (líneas) | ret HIR | loan | Proposed emit template (HOLD) |
|--------|--------------|---------|------|-------------------------------|
| `get` | L828–839 | `Option<T>` Vec | shared | ver §5 — **no thin**; índice `as usize` + ownership |
| `remove` | L841–852 | `Option<T>` Vec | exclusive | ver §5 — **≠** Rust `Vec::remove` (panic/`T`) |
| `first` / `last` | L854–861 | `Option<T>` | shared | Rust da `Option<&T>` — necesita política clone/copy |
| `trim` / `trim_start` / `trim_end` | L919–926 | `String` | shared | `{recv}.trim().to_string()` (o `trim_start`/`trim_end`) |
| `to_uppercase` / `to_lowercase` | L928–935 | `String` | shared | `{recv}.to_uppercase()` / `to_lowercase()` |
| `parse_int` | L937–944 | `Result<Int,String>` | shared | `{recv}.parse::<i64>().map_err(|e| e.to_string())` (pin CUT) |
| `to_string` | L946–953 | `String` | shared | `{recv}.to_string()` (Int\|Bool) |
| `abs` | L955–965 | `Int` (+ E0278 MIN) | shared | `{recv}.abs()` — cuidado `i64::MIN` |
| `min` / `max` | L967–978 | `Int` | shared | `{recv}.min({arg})` / `.max({arg})` |
| `clamp` | L980–997 | `Int` (+ E0279) | shared | `{recv}.clamp({lo}, {hi})` |
| `strip_prefix` / `strip_suffix` | L999–1010 | `Option<String>` | shared | `.strip_*(…).map(\|s\| s.to_string())` |
| `repeat` | L1012–1030 | String\|Vec | shared | `{recv}.repeat({n} as usize)` (+ E0280 neg) |
| `pow` | L1032–1053 | `Int` | shared | `{recv}.pow({exp} as u32)` (+ E0281/E0283) |
| `checked_*` / `saturating_*` / `wrapping_*` | L1055–1202 | ver §2 | shared | ver §2 |
| `is_some` / `is_none` | L1203–1210 | `Bool` Option | shared | `{recv}.is_some()` / `.is_none()` |
| `is_ok` / `is_err` | L1212–1219 | `Bool` Result | shared | `{recv}.is_ok()` / `.is_err()` |
| `unwrap_or` | L1221–1240 | inner / Ok | shared | `{recv}.unwrap_or({arg})` — **by-value** Option\|Result |
| `replace` | L1242–1254 | `String` | shared | `{recv}.replace(&{from}, &{to})` (ajustes lit) |

### 3.3 Option / Result constructores (no MethodCall)

Emit expr (no whitelist de método): `Some`/`None` L603–607; `Ok`/`Err` L595–601. `None` → `None::<i64>` turbofish (ADR-055 nota en código).

---

## 4. Plantilla AI — filas prioritarias (emitidos + Option)

Copiar/pegar para emitters (solo lo que **hoy** debe emitir o el draft marcado HOLD):

```
# EMITTED (codegen L634+)
push        | Vec mut     | 1 | ()           | exclusive | {recv}.push({a})
push_str    | String mut  | 1 | ()           | exclusive | lit: {recv}.push_str("…") | else: {recv}.push_str(&{a})
clear       | Vec|String mut | 0 | ()        | exclusive | {recv}.clear()
pop         | Vec mut     | 0 | Option<T>    | exclusive | {recv}.pop()
clone       | String|Vec  | 0 | same         | shared    | {recv}.clone()
contains    | String|Vec  | 1 | Bool         | shared    | lit / &{a} (ver L678)
starts_with | String      | 1 | Bool         | shared    | lit / &{a}
ends_with   | String      | 1 | Bool         | shared    | lit / &{a}
len         | String|Vec  | 0 | Int          | shared    | ({recv}.len() as i64)
is_empty    | String|Vec  | 0 | Bool         | shared    | {recv}.is_empty()

# HOLD — HIR yes / emit None (no land sin GO)
unwrap_or   | Option|Result | 1 | inner/Ok   | shared    | {recv}.unwrap_or({arg})
is_some     | Option      | 0 | Bool         | shared    | {recv}.is_some()
is_none     | Option      | 0 | Bool         | shared    | {recv}.is_none()
checked_add | Int         | 1 | Option<Int>  | shared    | {recv}.checked_add({arg})
# … resto familia §2
```

---

## 5. Inconsistencias / riesgos (flag)

1. **HIR ≫ emit gap:** docenas de métodos pasan `type_of_method_call` pero `emit_method_call` → `None` → llamada **silenciada** en Rust si el pipeline no aborta en `?`. Oráculos E2E que solo midan HIR pueden dar falsa sensación de surface completa.
2. **`get` / `remove` vs ADR-052 OUT:** ADR-052 §2 lista `get`/`remove` como **OUT v0**; HIR **L828–852** ya los tipa (`Option<T>`). Emit los rechaza. DOC vs código desalinea — no tratar como PASS emit.
3. **`get`/`remove`/`first`/`last` no son thin:**  
   - Rust `Vec::get` → `Option<&T>` + índice `usize`; HIR promete `Option<T>` (owned) → hace falta `.copied()`/`.cloned()` o rediseño.  
   - Rust `Vec::remove` → `T` y **panic** OOB; HIR `Option<T>` → **no** es thin `.remove()`; habría que emitir patrón distinto (p.ej. get+remove condicional) o cambiar HIR.  
   → Motivo para **bloquear** thin emit frente a `pop`/`clear`/`len`.
4. **ADR-045 vs HIR arith methods:** overflow binary `+`/`-`/`*` = E0217; methods `wrapping_*`/`checked_*` en HIR sin emit ni ADR surface landed. AI puede “inventar” surface incompleta.
5. **`push` sin arity gate en emit** (L638) vs `clear`/`pop`/string methods que sí validan → inconsistencia; HIR sí exige arity 1 (L788–790).
6. **`contains` Vec vs String:** mismo arm emit; HIR distingue tipos (L873–891). LitStr path en Vec es raro (HIR E0203) pero emit no typecheckea.
7. **Loan lists desincronizadas del match HIR:** `shared` L473 incluye métodos sin arm emit; métodos nuevos en `type_of_method_call` sin añadir a `shared`/`exclusive` → borrow holes (históricamente ADR-049 §1b).
8. **`unwrap_or` by-value:** consume Option/Result; loan solo “shared” temporal — si el place se reusa tras unwrap_or, riesgo E0201 / semántica move no espeljada en emit aún.
9. **DOC ADR-026/049 stale snippets:** tablas OUT aún citan `clear`/`pop` como fuera en sitios históricos; SoT método = HIR match + este DOC emit, no solo ADR-026 § whitelist antigua.
10. **No `saturating_div` / `wrapping_div`:** asimetría vs `checked_div` — AI no debe asumir familia completa.

---

## 6. Checklist review (HOLD)

- [x] Audit `emit_method_call` L634–737 citado
- [x] Tabla arith completa (todas **not present in emit yet**)
- [x] Plantilla AI + Vec/String + Option helpers
- [x] Flags get/remove vs thin + ADR-052
- [ ] GO Ingeniero para CUT emit arith / Option methods (crates) — **no** este DOC
- [ ] Alinear ADR-052 OUT ↔ HIR `get`/`remove` o revert HIR — decisión Arquitecto

## 7. Referencias rápidas

| Qué | Dónde |
|-----|--------|
| Emit method whitelist | `crates/arita-codegen/src/lib.rs:634` |
| HIR method whitelist | `crates/arita-hir/src/lib.rs:760` |
| Exclusive/shared loans | `crates/arita-hir/src/lib.rs:472-473` |
| Std minima | `DOC/ADR/026-std-minima.md` |
| Overflow / checked OUT DOC | `DOC/ADR/045-e0217-int-overflow.md` |
| Clear / Pop / Option | ADR-049 / 052 / 050 |
