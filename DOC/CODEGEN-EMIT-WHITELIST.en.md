[Español](CODEGEN-EMIT-WHITELIST.md) | English

# CODEGEN — Method emit whitelist (AI emitters)

- **Estado:** **REVIEW / HOLD IMPL** (2026-09-18) — docs-only; **do not** land crates without GO.
- **Autores:** ARITA Codegen (audit) · <person> + Ingeniero (request HOLD).
- **Code SoT (box `<scratch>/ARITA`; Lex `<repo>`):**
  - Emit: `crates/arita-codegen/src/lib.rs` → `emit_method_call` **L634–737**
  - HIR gate: `crates/arita-hir/src/lib.rs` → `type_of_method_call` **L760–1257**; loans **L472–473**
- **ADRs:** 026 (std minima), 045 (overflow / checked OUT v0), 046 (traps arith), 049 (clear), 050 (Option), 052 (pop; `get`/`remove` OUT DOC).
- **Gobernanza:** Safe-only (ADR-022). Emit falla → `None` (stmt omitido / expr falla cadena); HIR fuera whitelist → **E0206**. No inventar PASS.

## 1. Contract for AI emitters

Canonical template (one row = one whitelisted method):

| Column | Meaning |
|---------|-------------|
| **method** | ARITA identifier (string match in emit/HIR) |
| **receiver** | Allowed HIR type |
| **arity** | Arg count; HIR mismatch → **E0203** |
| **ret** | HIR return type |
| **loan** | `shared` / `exclusive` / (ninguno si no place) — HIR L472–473 |
| **emit Rust** | Literal fragment codegen must produce (or `None` today) |
| **Copy/move** | Notes for the emitter (do not move `mut` place; `usize`/`i64` casts) |

Rule: **HIR accepts ≠ emit exists**. If HIR OK and emit `None`, the program may typecheck and the call may **disappear** in Rust (silent drop) — critical risk (see §5).

---

## 2. Arithmetic table `checked_*` / `saturating_*` / `wrapping_*`

**HIR source:** `type_of_method_call` L1055–1202. **Emit source:** `emit_method_call` — **no** branch; they fall through `_ => None` **L736**.

ADR-045 §2.3: *OUT v0: wrapping APIs intencionales; checked_add surface* — HIR surface already exists in code; **emit still missing**; ADR DOC has no dedicated landed CUT for these methods.

| method | HIR recv | arity | HIR ret | loan (HIR) | emit Rust fragment (hoy) | Copy/move notes | Proposed emit (HOLD — no land) |
|--------|----------|-------|---------|------------|--------------------------|-----------------|--------------------------------|
| `checked_add` | `Int` | 1×`Int` | `Option<Int>` | shared | **not present in emit yet** (`None`) | `i64` is Copy; Rust `i64::checked_add` by value | `{recv}.checked_add({arg})` |
| `checked_sub` | `Int` | 1×`Int` | `Option<Int>` | shared | **not present** | idem | `{recv}.checked_sub({arg})` |
| `checked_mul` | `Int` | 1×`Int` | `Option<Int>` | shared | **not present** | idem | `{recv}.checked_mul({arg})` |
| `checked_div` | `Int` | 1×`Int` | `Option<Int>` | shared | **not present** | div0 → `None` (no panic); different from `/` op E0216 | `{recv}.checked_div({arg})` |
| `checked_rem` | `Int` | 1×`Int` | `Option<Int>` | shared | **not present** | idem rem | `{recv}.checked_rem({arg})` |
| `saturating_add` | `Int` | 1×`Int` | `Int` | shared | **not present** | Copy; no Option | `{recv}.saturating_add({arg})` |
| `saturating_sub` | `Int` | 1×`Int` | `Int` | shared | **not present** | idem | `{recv}.saturating_sub({arg})` |
| `saturating_mul` | `Int` | 1×`Int` | `Int` | shared | **not present** | idem | `{recv}.saturating_mul({arg})` |
| `wrapping_add` | `Int` | 1×`Int` | `Int` | shared | **not present** | Copy; intentional silent wrap (contrast E0217 on `+`) | `{recv}.wrapping_add({arg})` |
| `wrapping_sub` | `Int` | 1×`Int` | `Int` | shared | **not present** | idem | `{recv}.wrapping_sub({arg})` |
| `wrapping_mul` | `Int` | 1×`Int` | `Int` | shared | **not present** | idem | `{recv}.wrapping_mul({arg})` |

**Absent from HIR and emit (do not invent):** `saturating_div`, `wrapping_div`/`rem`, `checked_neg`, `wrapping_neg`, `saturating_pow`, etc. → E0206 if called.

---

## 3. Current emit whitelist (success vs `None`)

### 3.1 Emit **OK** (`emit_method_call` match arms L637–735)

| method | emit Rust fragment (real code) | arity gate in emit | Copy/move / notes |
|--------|----------------------------------|--------------------|-------------------|
| `push` | `{recv}.push({args…})` L638–643 | **does not** check arity in emit | exclusive loan HIR; requires `mut` (E0202) |
| `push_str` | LitStr → `{recv}.push_str("…")`; else `{recv}.push_str(&{a})` L645–658 | arity ≠1 → `None` | exclusive; String mut; `&` en no-lit |
| `clear` | `{recv}.clear()` L660–664 | non-empty args → `None` | exclusive; Vec\|String mut (ADR-049) |
| `pop` | `{recv}.pop()` L666–670 | non-empty args → `None` | exclusive; ret `Option<T>` (ADR-052); Vec only in HIR |
| `clone` | `{recv}.clone()` L672–676 | non-empty args → `None` | shared; String\|Vec (do not move out of the place) |
| `contains` | LitStr → `.contains("…")`; else `.contains(&{a})` L678–691 | arity ≠1 → `None` | shared; String or Vec\<T\> |
| `starts_with` | mirrors contains L693–706 | arity ≠1 → `None` | shared; String |
| `ends_with` | mirrors L708–721 | arity ≠1 → `None` | shared; String |
| `len` | `({recv}.len() as i64)` L723–728 | non-empty args → `None` | shared; **bytes** String; Int=`i64` (ADR-026) |
| `is_empty` | `{recv}.is_empty()` L730–734 | non-empty args → `None` | shared |

Any other `method` → **`None`** L736 (incl. arith family, Option helpers, `get`/`remove`, String trim/replace, Int `abs`/`pow`/…).

### 3.2 HIR whitelist **without** emit (gap — HOLD)

Methods typed in HIR (`type_of_method_call`) that emit **today** rejects (`None`):

| method | HIR (lines) | HIR ret | loan | Proposed emit template (HOLD) |
|--------|--------------|---------|------|-------------------------------|
| `get` | L828–839 | `Option<T>` Vec | shared | see §5 — **not thin**; index `as usize` + ownership |
| `remove` | L841–852 | `Option<T>` Vec | exclusive | ver §5 — **≠** Rust `Vec::remove` (panic/`T`) |
| `first` / `last` | L854–861 | `Option<T>` | shared | Rust gives `Option<&T>` — needs clone/copy policy |
| `trim` / `trim_start` / `trim_end` | L919–926 | `String` | shared | `{recv}.trim().to_string()` (or `trim_start`/`trim_end`) |
| `to_uppercase` / `to_lowercase` | L928–935 | `String` | shared | `{recv}.to_uppercase()` / `to_lowercase()` |
| `parse_int` | L937–944 | `Result<Int,String>` | shared | `{recv}.parse::<i64>().map_err(|e| e.to_string())` (pin CUT) |
| `to_string` | L946–953 | `String` | shared | `{recv}.to_string()` (Int\|Bool) |
| `abs` | L955–965 | `Int` (+ E0278 MIN) | shared | `{recv}.abs()` — careful with `i64::MIN` |
| `min` / `max` | L967–978 | `Int` | shared | `{recv}.min({arg})` / `.max({arg})` |
| `clamp` | L980–997 | `Int` (+ E0279) | shared | `{recv}.clamp({lo}, {hi})` |
| `strip_prefix` / `strip_suffix` | L999–1010 | `Option<String>` | shared | `.strip_*(…).map(\|s\| s.to_string())` |
| `repeat` | L1012–1030 | String\|Vec | shared | `{recv}.repeat({n} as usize)` (+ E0280 neg) |
| `pow` | L1032–1053 | `Int` | shared | `{recv}.pow({exp} as u32)` (+ E0281/E0283) |
| `checked_*` / `saturating_*` / `wrapping_*` | L1055–1202 | see §2 | shared | see §2 |
| `is_some` / `is_none` | L1203–1210 | `Bool` Option | shared | `{recv}.is_some()` / `.is_none()` |
| `is_ok` / `is_err` | L1212–1219 | `Bool` Result | shared | `{recv}.is_ok()` / `.is_err()` |
| `unwrap_or` | L1221–1240 | inner / Ok | shared | `{recv}.unwrap_or({arg})` — **by-value** Option\|Result |
| `replace` | L1242–1254 | `String` | shared | `{recv}.replace(&{from}, &{to})` (lit adjustments) |

### 3.3 Option / Result constructors (not MethodCall)

Emit expr (not a method whitelist): `Some`/`None` L603–607; `Ok`/`Err` L595–601. `None` → `None::<i64>` turbofish (ADR-055 note in code).

---

## 4. AI template — priority rows (emitted + Option)

Copy/paste for emitters (only what **today** must emit or the HOLD-marked draft):

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

# HOLD — HIR yes / emit None (do not land without GO)
unwrap_or   | Option|Result | 1 | inner/Ok   | shared    | {recv}.unwrap_or({arg})
is_some     | Option      | 0 | Bool         | shared    | {recv}.is_some()
is_none     | Option      | 0 | Bool         | shared    | {recv}.is_none()
checked_add | Int         | 1 | Option<Int>  | shared    | {recv}.checked_add({arg})
# … rest of family §2
```

---

## 5. Inconsistencies / risks (flag)

1. **HIR ≫ emit gap:** dozens of methods pass `type_of_method_call` pero `emit_method_call` → `None` → call **silenced** in Rust if the pipeline does not abort on `?`. E2E oracles that only measure HIR can give a false sense of a complete surface.
2. **`get` / `remove` vs ADR-052 OUT:** ADR-052 §2 lists `get`/`remove` as **OUT v0**; HIR **L828–852** already types them (`Option<T>`). Emit rejects them. DOC vs code misaligned — do not treat as emit PASS.
3. **`get`/`remove`/`first`/`last` no son thin:**  
   - Rust `Vec::get` → `Option<&T>` + `usize` index; HIR promises `Option<T>` (owned) → needs `.copied()`/`.cloned()` or redesign.  
   - Rust `Vec::remove` → `T` y **panic** OOB; HIR `Option<T>` → **not** thin `.remove()`; would need a different emit pattern (e.g. conditional get+remove) or HIR change.  
   → Reason to **block** thin emit versus `pop`/`clear`/`len`.
4. **ADR-045 vs HIR arith methods:** overflow binary `+`/`-`/`*` = E0217; `wrapping_*`/`checked_*` methods in HIR without emit or landed surface ADR. AI may “invent” an incomplete surface.
5. **`push` without arity gate in emit** (L638) vs `clear`/`pop`/string methods that do validate → inconsistency; HIR does require arity 1 (L788–790).
6. **`contains` Vec vs String:** same emit arm; HIR distinguishes types (L873–891). LitStr path on Vec is rare (HIR E0203) but emit does not typecheck.
7. **Loan lists desynced from HIR match:** `shared` L473 includes methods with no emit arm; new methods in `type_of_method_call` without adding to `shared`/`exclusive` → borrow holes (historically ADR-049 §1b).
8. **`unwrap_or` by-value:** consumes Option/Result; loan only temporary “shared” — if the place is reused after unwrap_or, E0201 risk / move semantics not yet mirrored in emit.
9. **DOC ADR-026/049 stale snippets:** OUT tables still cite `clear`/`pop` as out in historical places; method SoT = HIR match + this emit DOC, not only the old ADR-026 § whitelist.
10. **No `saturating_div` / `wrapping_div`:** asymmetry vs `checked_div` — AI must not assume a complete family.

---

## 6. Checklist review (HOLD)

- [x] Audit `emit_method_call` L634–737 citado
- [x] Full arith table (all **not present in emit yet**)
- [x] Plantilla AI + Vec/String + Option helpers
- [x] Flags get/remove vs thin + ADR-052
- [ ] Engineer GO for emit arith / Option methods CUT (crates) — **not** this DOC
- [ ] Align ADR-052 OUT ↔ HIR `get`/`remove` or revert HIR — Architect decision

## 7. Quick references

| What | Where |
|-----|--------|
| Emit method whitelist | `crates/arita-codegen/src/lib.rs:634` |
| HIR method whitelist | `crates/arita-hir/src/lib.rs:760` |
| Exclusive/shared loans | `crates/arita-hir/src/lib.rs:472-473` |
| Std minima | `DOC/ADR/026-std-minima.md` |
| Overflow / checked OUT DOC | `DOC/ADR/045-e0217-int-overflow.md` |
| Clear / Pop / Option | ADR-049 / 052 / 050 |
