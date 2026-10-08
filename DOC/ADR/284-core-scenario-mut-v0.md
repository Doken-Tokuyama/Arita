# ADR-284 — Core 0.9 slice 3: SCENARIO-MUT

- **Estado:** **CLOSED** Lex **825/825** (Ingeniero 2026-09-27 · measure + `cargo test -p arita-cli` exclusivos · Veyra ACCEPTED `20260927T051022Z`) → [`GATE-CORE09-SCENARIO-MUT-20260926.md`](../GATE-CORE09-SCENARIO-MUT-20260926.md) · antes: GO IMPL 2026-09-27 tras CLOSED ADR-283
- **CUT-ID:** `CORE-0.9-SCENARIO-MUT-20260926`
- **Fecha:** 2026-09-26
- **Autores:** Arquitecto (pins) · Codegen · Measure · Ingeniero (pins OK 2026-09-26 19:54)
- **Padre:** [ADR-281](281-core-0.9-pins.md) §0.1 slice 3
- **Prev:** slice 2 VEC-ASSIGN (ADR-283) — prereq CLOSED al GO IMPL; este CUT **no** reabre 282/283
- **Prereq surface (al GO IMPL):** MAP-ASSIGN (282) + VEC-ASSIGN (283) en tree + `fn → Result` / `?` (277/278) + `set`/`put`/`get`/`[]` (265/266/237) + CLI/JSON 0.1 (238/240) si se usa entrada
- **HOLD:** Mutex · threads · idle · TLS/WS · crates.io · repair · I/O-new · String.set · IndexMut residual (E0314) · `?` in Io main · Option? · surface unwrap/expect (241) · HTTP scenarios · reopen E0272/E0291/E0340/E0341/E0342/E0343

## Objetivo

Scenarios/acceptance **end-to-end** que mutan **Map + Vec** juntos por index-assign: happy (Map insert/overwrite + Vec in-bounds dentro de helper `fn → Result<_, Int>`) · err (Vec OOB propagado como `Err(0)` → main match fail determinista) · neg (E0344 / E0202 / E0314). Measure firma sin theater. **Sin API nueva.** Puente hacia slice 4 REF-MUT (`arita-ref-mut`).

## 0. Corte decisivo

| Pin | Decisión |
|-----|----------|
| **Harness** | `scenario`/`acceptance` language-level (reusar 0.1 / 239 / 262 / 267 / 279 style) |
| **Entrada** | ints/strings literales **y/o** args CLI / JSON mínimo ya curado (238/240) — **no** bindings I/O nuevos |
| **Mutación** | `m[k] = v` (282) en cualquier contexto · `v[i] = x` (283) **solo** en helper `fn → Result<_, Int>` · composición con `?` (278) |
| **Salida** | stdout determinista (print/assert); main `Io<()>` **match-convierte** (sin `?` ni `v[i] =` en main Io) |
| **Err en main** | «main match fail determinista» = rama `Err` de `main`: exit 0, stdout fijo, sin panic (precedente ADR-279 `02-err`). El oráculo también rechaza si aparece `panicked` en stderr (pin Ingeniero 05:27) |
| **Mínimo** | los 9 ids exactos del §2 (4 positivos con stdout pineado + negs con código exacto); ver §2 (pin Ingeniero 05:27) |
| **Anti-theater** | caller **no** traga `Err` a lit éxito (H4: **E0272** `result error swallowed` exacto, como ADR-283; E0223 solo si el patrón no casa con el tipo; sin reopen; sin oracle en este slice) · sin unwrap/expect (241) · sin discard del `Result` (backlog must-use, ADR-281 D2; sin oracle) |
| **OUT** | Mutex · net/HTTP · String.set · IndexMut residual · `?` in main Io · crates.io · reopen 282/283 |

## 1. Surface

Sin keyword nueva. Reusa `scenario` + `m[k] = v` + `v[i] = x` + `fn → Result` + `?` + `[]`/`get` + print/assert.

Ejemplo ilustrativo: no fija outputs. Los stdout exactos y los ids están en §2 (pin Ingeniero 05:27).

```text
fn tally(i: Int, x: Int) -> Result<Int, Int> {
  let mut m: Map<Text, Int> = Map::new()
  m["hits"] = 1                    // 282 insert
  m["hits"] = 2                    // 282 overwrite
  let mut v: Vec<Int> = Vec::new()
  v.push(0)
  v.push(0)
  v[i] = x                         // 283: OOB → return Err(0)
  let n: Int = v.len()
  Ok(n)
}

scenario mut_happy {
  // tally(1, 9) → Ok → main match → stdout fijo
  acceptance { /* measure */ }
}
scenario mut_err {
  // tally(99, 9) → Err(0) propagado → main match fail determinista; sin panic
  acceptance { /* measure */ }
}
scenario mut_chain {
  // helper A (Map) ? → helper B (Vec OOB) ? → primer Err corta cadena
  acceptance { /* measure */ }
}
// neg: v[i] = x en main Io → E0344
// neg: m[k] = v sobre `let m` no-mut → E0202 (= put)
```

## 2. Oracles (Measure) — éxito medible

Requeridos (a medir al GO IMPL; **ninguno** medido en este DOC):

| Id | Expect |
|----|--------|
| `core09-scen-mut-happy` | Map insert/overwrite + Vec in-bounds en fn→Result → stdout exacto `["2","9","2","mut-happy"]` + exit 0 + `arita contract` **Accepted** (pin Ingeniero 05:27) |
| `core09-scen-mut-err` | Vec OOB → `Err(0)` propagado → main match fail determinista (rama `Err` de `main`, §0 **Err en main**; no panic) → stdout exacto `["0","mut-err"]` + exit 0 + `arita contract` **Accepted**; rejected si aparece `panicked` en stderr (pin Ingeniero 05:27) |
| `core09-scen-mut-chain` | `?` chain: Map ok + Vec OOB → early-return Err; efectos posteriores **no** ocurren → stdout exacto `["40","err","77","err","0"]` + exit 0 + `arita contract` **Accepted**. Orden: el `77` de `chain(1)` es control de alcanzabilidad; **ningún** `77` después del tercer elemento; final `err`,`0` (`Err(0)` por early-return, **no** `Err(1)`). El único efecto observable de `stage_post` es su valor de retorno (pin Ingeniero 05:27) |
| `core09-scen-mut-map-main` | `m[k] = v` directamente en main `Io<()>` → OK (total) → stdout exacto `["3","1","7","3"]` + exit 0 + `arita contract` **Accepted** (pin Ingeniero 05:27) |
| `neg-core09-scen-vec-assign-outside` | `v[i] = x` en main Io → **E0344** |
| `neg-core09-scen-map-assign-non-mut` | `m[k] = v` sobre binding no-mut → **E0202** (= `put`) |
| `neg-core09-scen-unwrap-theater` | `unwrap`/`expect` sobre el `Result` de un `set`/assign dentro de un helper `fn → Result`, ligado antes en un bind (`let r = …; r.unwrap()`), sin encadenar (mismo patrón que 282/283) → **E0206** `method not in F2 std whitelist` exacto; **Rejected** si compila, si da E0291 o cualquier otro código (emit-ban 241; no E0291 reopen; pin Ingeniero 05:27) |
| `core09-scen-mut-emit-ban` | emit grep: cero IndexMut-style `x[..] =` Rust, cero unwrap/expect/panic |
| `core09-scen-mut-build` | measure build verde |

**Positivos (happy / err / chain / map-main; pin Ingeniero 05:27):** cada positivo exige `arita contract` **Accepted** además del stdout pineado y exit 0. El bloque `acceptance` declara el stdout real completo, sin quitar ni debilitar el bloque `scenario`.

**Anti-theater:** skip ≠ PASS. **No** inventar PASS. **No** usar discard `Result` (E0340 es host IO; must-use → backlog; sin E0345) como oracle de este slice.

## 3. Qué NO tocar

| Prohibido | Motivo |
|-----------|--------|
| Mutex · threads · idle · TLS/WS · crates.io · repair · I/O-new · String.set | HOLD |
| IndexMut residual | HOLD (E0314) |
| `?` / `v[i] =` en main Io | OUT (278 / 283 E0344) |
| GO IMPL antes CLOSED 282+283 + OK Ingeniero | gate Orquestador |
| Reabrir E0272/E0291/E0340/E0341/E0342/E0343 | anti-colisión |
| Declarar CLOSED Core 0.9 | solo slice 4 REF-MUT |
| Inventar PASS / Lex N/N | DOC only |

## 4. Criterio CLOSED

Lex §2 verde con **N exacto = 825** (816 previos + los 9 ids del §2; pin Ingeniero 05:27); tick ADR-281 slice 3; HOLDs §3 intactos. **Cierre requiere:** run final en exclusiva + freeze shasum · `cargo test -p arita-cli` en exclusiva · Veyra con `--timeout 2400` **ACCEPTED** exit 0 (pendiente; **no** CLOSED en este DOC). Siguiente: slice 4 REF-MUT ([ADR-285](285-core-ref-mut-v0.md)).

## Checklist

- [x] Pins scenarios Map+Vec happy/err/chain/neg + oracles (GO-listo DOC)
- [x] Revisión Ingeniero 2026-09-26 19:54 — pins OK · HOLD
- [x] 2026-09-27 05:27 · pins vistos por el Ingeniero (05:27): unwrap-theater E0206 exacto · stdout exacto 4 positivos + orden chain · main Err determinista sin panic · contract Accepted · N=825 · cierre exclusivo + Veyra 2400
- [x] GO IMPL (Ingeniero 2026-09-27, tras CLOSED ADR-283 Lex 816/816)
- [x] IMPL + Lex CLOSED — Lex 825/825 (Ingeniero 2026-09-27)

## Cierre

- **CLOSED** Lex 825/825 (Ingeniero 2026-09-27) · gate [`GATE-CORE09-SCENARIO-MUT-20260926.md`](../GATE-CORE09-SCENARIO-MUT-20260926.md) · siguiente: slice 4 REF-MUT GO IMPL · **sin** Core 0.9 CLOSED.
