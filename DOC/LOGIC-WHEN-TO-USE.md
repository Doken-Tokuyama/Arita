# Cuándo usar la isla lógica (F3)

**Estado:** draft GO ADR-099 — Logic completa con oráculos f3-11/12.

## Humano

- Usa **logic** para hechos/reglas/queries (cierres relacionales, unsat real).
- Usa **Imperativo** para ownership, Vec/String, aritmética checked/saturating/wrapping.
- **Mutex / `[]` / insert:** PARK — no pedirlos aún.

## IA

- Query fail → **E0301 rejected**, nunca PASS.
- Multi-query: un fail tumba el run (anti partial-PASS) — ver `f3-11`.
- Join miss → E0301 — ver `f3-12`.
- `skip` / `inconclusive` ≠ accepted.
- Cero theater: no inventar facts para forzar PASS.

## Relacionados

- ADR-012, PACK-F3-FEWSHOT, ADR-099, TRAPS-CATALOG #18–19.

## Int wrap/sat → Logic

**OUT v0.** No bajar i64 wrapping/saturating/checked a queries. Ver [LOGIC-INT-ARITH-OUT.md](LOGIC-INT-ARITH-OUT.md).

## Ejemplos canónicos (ADR-099)

| Archivo | Oracle | Outcome |
|---------|--------|---------|
| `ejemplos/f3/11-multi-query-fail.arita` | `f3-11-multi-query-fail` | path(a,c) sat + path(c,a) fail → **E0301** whole module |
| `ejemplos/f3/12-join-miss-fail.arita` | `f3-12-join-miss-fail` | `team_mate(alice, bob)` join miss → **E0301** |

```bash
cargo run -p arita-cli -- logic ejemplos/f3/11-multi-query-fail.arita  # exit 1 + E0301
cargo run -p arita-cli -- logic ejemplos/f3/12-join-miss-fail.arita    # exit 1 + E0301
cargo test -p arita-cli f3_
```

**Pins:** reuse E0301 (ADR-012); OUT Mutex / Imperativo / wrapping_mul; inconclusive ≠ accepted.

