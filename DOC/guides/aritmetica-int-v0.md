# Guía — Aritmética Int v0

**Audiencia:** humanos. **SoT barras:** Lex `STABLE_VERIFY` / mirrors `_mirror_096`…`_097`. No inventar PASS.

## Trilogía (1 frase cada una)

| Modo | Semántica | Retorno | Cuándo |
|------|-----------|---------|--------|
| **checked** | overflow → **Option** (`None`) | `Option<Int>` | Quieres detectar overflow y ramificar (`if let` / `match`) |
| **saturating** | overflow → **clamp** MAX/MIN | `Int` | Contadores / cotas que no deben wrapear ni fallar |
| **wrapping** | overflow → **wrap** two's complement | `Int` | Bit-pattern / hashes / semántica wrap explícita |

Mnemonic Arquitecto: **checked→Option / saturating→clamp / wrapping→wrap**.

## Ejemplos mínimos

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

## Familias verificadas (Lex)

- checked +−×/% — ADR-080…084 (cierra en **201/201**)
- saturating +−× — ADR-090…092 (**218/218** … **224/224**)
- wrapping_add — ADR-096 **238/238** verified (`_mirror_096`)
- wrapping_sub — ADR-097 **241/241** verified (`_mirror_097`)
- wrapping_mul — ADR-098 **244/244** verified (`_mirror_098`; wrapping +−× v0 cerrada)

## vs operadores lit

- `+` `-` `*` con lits que overflow → **E0217** (reject HIR)
- `/` `%` lit 0 → **E0216**
- Las APIs `checked_*` / `saturating_*` / `wrapping_*` **no** sustituyen esos rejects; son surface runtime-safe aparte.

## Receptor ilegal

`String.checked_add(...)` / similar → **E0206** (whitelist Int).

## PARK

Mutex / `[]` / `insert`. Cero crates. No bajar wrap/sat a isla lógica v0.

## Enlaces

- Pack IA: [PACK-STD-METHOD-SURFACE-F2.md](../PACK-STD-METHOD-SURFACE-F2.md)
- ADRs: 080–084, 090–092, 096–098
