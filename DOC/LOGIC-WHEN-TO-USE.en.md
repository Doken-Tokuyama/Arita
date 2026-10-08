[Español](LOGIC-WHEN-TO-USE.md) | English

# When to use the logic island (F3)

**Estado:** draft GO ADR-099 — Logic completa con oráculos f3-11/12.

## Human

- Use **logic** for facts/rules/queries (relational closures, real unsat).
- Use **Imperativo** for ownership, Vec/String, checked/saturating/wrapping arithmetic.
- **Mutex / `[]` / insert:** PARK — do not ask for them yet.

## AI

- Query fail → **E0301 rejected**, never PASS.
- Multi-query: one fail fails the whole run (anti partial-PASS) — see `f3-11`.
- Join miss → E0301 — see `f3-12`.
- `skip` / `inconclusive` ≠ accepted.
- Zero theater: do not invent facts to force PASS.

## Related

- ADR-012, PACK-F3-FEWSHOT, ADR-099, TRAPS-CATALOG #18–19.

## Int wrap/sat → Logic

**OUT v0.** Do not lower i64 wrapping/saturating/checked to queries. See [LOGIC-INT-ARITH-OUT.md](LOGIC-INT-ARITH-OUT.en.md).

## Canonical examples (ADR-099)

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
