# PACK — Std method surface F2 (emitters / few-shot)

- **Estado:** **DOC v0** (Inventario IA; 2026-09-18)
- **Audiencia:** IAs / emitters / few-shot — denso, sin fluff
- **Barra SoT:** Lex `STABLE_VERIFY` + mirrors `/workspace/arita-mirrors/_mirror_096`…`_097` (no `_098` aún)
- **Última barra firmada:** ADR-098 → **244/244** (`_mirror_098`); ADR-097 → **241/241** (`_mirror_097`); ADR-096 → **238/238** (`_mirror_096`)
- **ADR-098** `wrapping_mul`: **verified** Lex **244/244** (wrapping +−× v0 cerrada)
- **PARK:** Mutex / `[]` / `insert` — cero crates nuevos
- **Trilogía:** checked → `Option` · saturating → clamp · wrapping → wrap (ver guías humanas)

## Few-shot 1 línea / modo

```text
checked:    let o: Option<Int> = a.checked_add(b)          // overflow → None
saturating: let x: Int = a.saturating_add(b)               // overflow → MAX/MIN
wrapping:   let x: Int = a.wrapping_add(b)                 // overflow → wrap i64
unwrap_or:  let x: Int = opt.unwrap_or(0)                  // Option|Result → T
pred:       let b: Bool = opt.is_some()                    // is_none/is_ok/is_err
```

**E0206** en receptor no-Int (arith) o no-Option/Result (helpers): `neg-e0206-*` → rejected.

## Inventario — Int checked (ADR-080…084)

| Surface | Arity | Ret | Emit Rust | Neg típico | Oráculos (ids) | ADR | Barra Lex |
|---------|------:|-----|-----------|------------|----------------|-----|-----------|
| `a.checked_add(b)` | 1 | `Option<Int>` | `.checked_add` | E0206 String | `std-checked-add-some/none`, `neg-e0206-checked-add-string` | [080](ADR/080-std-checked-add-v0.md) | **187/187** |
| `a.checked_sub(b)` | 1 | `Option<Int>` | `.checked_sub` | E0206 | `std-checked-sub-some/none`, `neg-e0206-checked-sub-string` | [081](ADR/081-std-checked-sub-v0.md) | **190/190** |
| `a.checked_mul(b)` | 1 | `Option<Int>` | `.checked_mul` | E0206 | `std-checked-mul-some/none`, `neg-e0206-checked-mul-string` | [082](ADR/082-std-checked-mul-v0.md) | **193/193** |
| `a.checked_div(b)` | 1 | `Option<Int>` | `.checked_div` | E0206 | `std-checked-div-some/none-zero/none-overflow`, `neg-e0206-checked-div-string` | [083](ADR/083-std-checked-div-v0.md) | **197/197** |
| `a.checked_rem(b)` | 1 | `Option<Int>` | `.checked_rem` | E0206 | `std-checked-rem-some/none-zero/none-overflow`, `neg-e0206-checked-rem-string` | [084](ADR/084-std-checked-rem-v0.md) | **201/201** |

Pins: Int×Int only; None en overflow / div0 / MIN%-1; complementa E0216/E0217 (no sustituye lit reject).

## Inventario — Int saturating (ADR-090…092)

| Surface | Arity | Ret | Emit Rust | Neg típico | Oráculos | ADR | Barra Lex |
|---------|------:|-----|-----------|------------|----------|-----|-----------|
| `a.saturating_add(b)` | 1 | `Int` | `.saturating_add` | E0206 | `std-saturating-add-normal/max`, `neg-e0206-saturating-add-string` | [090](ADR/090-std-saturating-add-v0.md) | **218/218** |
| `a.saturating_sub(b)` | 1 | `Int` | `.saturating_sub` | E0206 | `std-saturating-sub-normal/min`, `neg-e0206-saturating-sub-string` | [091](ADR/091-std-saturating-sub-v0.md) | **221/221** |
| `a.saturating_mul(b)` | 1 | `Int` | `.saturating_mul` | E0206 | `std-saturating-mul-normal/max`, `neg-e0206-saturating-mul-string` | [092](ADR/092-std-saturating-mul-v0.md) | **224/224** |

Pins: ret Int (no Option); clamp a MAX/MIN; saturating_div OUT (no std i64).

## Inventario — Int wrapping (ADR-096…098)

| Surface | Arity | Ret | Emit Rust | Neg típico | Oráculos | ADR | Barra Lex |
|---------|------:|-----|-----------|------------|----------|-----|-----------|
| `a.wrapping_add(b)` | 1 | `Int` | `.wrapping_add` | E0206 | `std-wrapping-add-normal/overflow`, `neg-e0206-wrapping-add-string` | [096](ADR/096-std-wrapping-add-v0.md) | **238/238** verified |
| `a.wrapping_sub(b)` | 1 | `Int` | `.wrapping_sub` | E0206 | `std-wrapping-sub-normal/underflow`, `neg-e0206-wrapping-sub-string` | [097](ADR/097-std-wrapping-sub-v0.md) | **241/241** verified |
| `a.wrapping_mul(b)` | 1 | `Int` | `.wrapping_mul` | E0206 | `std-wrapping-mul-normal/overflow`, `neg-e0206-wrapping-mul-string` | [098](ADR/098-std-wrapping-mul-v0.md) | **244/244** verified |

## Inventario — Option/Result helpers (ADR-093…094)

| Surface | Arity | Ret | Emit Rust | Neg típico | Oráculos | ADR | Barra Lex |
|---------|------:|-----|-----------|------------|----------|-----|-----------|
| `o.unwrap_or(d)` | 1 | `T` | `.unwrap_or` | E0206 Int | `std-unwrap-or-option-some/none`, `std-unwrap-or-result-ok/err`, `neg-e0206-unwrap-or-int` | [093](ADR/093-std-unwrap-or-v0.md) | **229/229** |
| `o.is_some()` | 0 | `Bool` | `.is_some` | E0206 Int | `std-is-some-true`, `neg-e0206-is-some-int` | [094](ADR/094-std-is-some-ok-v0.md) | **234/234** |
| `o.is_none()` | 0 | `Bool` | `.is_none` | E0206 | `std-is-none-true` | [094](ADR/094-std-is-some-ok-v0.md) | **234/234** |
| `r.is_ok()` | 0 | `Bool` | `.is_ok` | E0206 | `std-is-ok-true` | [094](ADR/094-std-is-some-ok-v0.md) | **234/234** |
| `r.is_err()` | 0 | `Bool` | `.is_err` | E0206 | `std-is-err-true` | [094](ADR/094-std-is-some-ok-v0.md) | **234/234** |

Pins: whitelist Option|Result; `d:T` mismo tipo; no unwrap/expect/? v0.

## Emit sketch (canonical)

```text
a.checked_add(b)       → a.checked_add(b)                 // Option<i64>
a.saturating_add(b)    → a.saturating_add(b)              // i64
a.wrapping_add(b)      → a.wrapping_add(b)                // i64
opt.unwrap_or(0)       → opt.unwrap_or(0)
opt.is_some()          → opt.is_some()
```

## Anti-reglas IA

1. No inventar barra / PASS — citar solo `STABLE_VERIFY` o mirrors.
2. Receptor wrong type → **E0206**, no rustc panic theater.
3. `+`/`-`/`*` lit overflow sigue **E0217**; `/` `%` lit0 → **E0216** — checked/saturating/wrapping son APIs aparte.
4. Mutex / `[]` / insert → **PARK**.
5. Logic island **OUT** de wrap/sat/checked i64 (ver `DOC/LOGIC-INT-ARITH-OUT.md` si existe).

## Guías humanas

- [guides/aritmetica-int-v0.md](guides/aritmetica-int-v0.md) — cuándo checked / saturating / wrapping
- [guides/option-result-helpers-v0.md](guides/option-result-helpers-v0.md) — unwrap_or + predicados
