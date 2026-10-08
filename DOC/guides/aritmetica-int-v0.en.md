[Español](aritmetica-int-v0.md) | English

# Guide — Int arithmetic v0

**Audience:** humans. **SoT bars:** Lex `STABLE_VERIFY` / mirrors `_mirror_096`…`_097`. Do not invent PASS.

## Trilogy (1 sentence each)

| Mode | Semantics | Return | When |
|------|-----------|---------|--------|
| **checked** | overflow → **Option** (`None`) | `Option<Int>` | You want to detect overflow and branch (`if let` / `match`) |
| **saturating** | overflow → **clamp** MAX/MIN | `Int` | Counters / bounds that must not wrap or fail |
| **wrapping** | overflow → **wrap** two's complement | `Int` | Bit-pattern / hashes / explicit wrap semantics |

Architect mnemonic: **checked→Option / saturating→clamp / wrapping→wrap**.

## Minimal examples

```arita
// checked — Option
let o: Option<Int> = a.checked_add(b)
if let Some(x) = o {
  print(x)
}

// saturating — clamp
let x: Int = a.saturating_add(b)

// wrapping — wrap
let y: Int = a.wrapping_add(b)
```

## Verified families (Lex)

- checked +−×/% — ADR-080…084 (closes at **201/201**)
- saturating +−× — ADR-090…092 (**218/218** … **224/224**)
- wrapping_add — ADR-096 **238/238** verified (`_mirror_096`)
- wrapping_sub — ADR-097 **241/241** verified (`_mirror_097`)
- wrapping_mul — ADR-098 **244/244** verified (`_mirror_098`; wrapping +−× v0 closed)

## vs lit operators

- `+` `-` `*` with overflowing lits → **E0217** (HIR reject)
- `/` `%` lit 0 → **E0216**
- The `checked_*` / `saturating_*` / `wrapping_*` APIs do **not** replace those rejects; they are a separate runtime-safe surface.

## Illegal receiver

`String.checked_add(...)` / similar → **E0206** (Int whitelist).

## PARK

Mutex / `[]` / `insert`. Zero crates. Do not lower wrap/sat to logic island v0.

## Links

- AI pack: [PACK-STD-METHOD-SURFACE-F2.md](../PACK-STD-METHOD-SURFACE-F2.md)
- ADRs: 080–084, 090–092, 096–098
