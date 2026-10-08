# Logic vs surface predicates — is_multiple_of / is_positive|negative / ilog (Option)

**CUT:** analysis-only (<person> / Ingeniero, 2026-09-18)  
**Bar context:** Lex **314/314** (surface). **No landings** this note.  
**Pins:** ADR-012 (isla F3 Datalog +); `DOC/LOGIC-INT-ARITH-OUT.md` (Int arith OUT of Logic v0).

## Verdict (one line)

**Solo surface (Imperativo / std Int).** No entran en la isla Tau/TML-shaped v0 como builtins del motor; un bridge futuro sería ADR nuevo, no bajar métodos `Int` a `arita-logic`.

## IN / OUT vs Logic

| Predicado (surface) | Tipo | Logic v0 | Por qué |
|---------------------|------|----------|--------|
| `is_positive` / `is_negative` | `Int → Bool` (ADR-106) | **OUT** | Predicado aritmético sobre magnitud; Logic v0 no tiene sort `Int` ni evaluación numérica |
| `is_multiple_of` | `Int×Int → Bool` (ADR-117; `d==0 → false`) | **OUT** | Divisibilidad / rem; motor F3 es Herbrand positivo, no aritmética |
| `ilog2` / `ilog10` | `Int → Option<Int>` (ADR-121/122) | **OUT** | Log entero + **Option** (failure channel surface); Logic no hostea Option/Result ni partial numeric functions |

**Also OUT (related):** wrapping/saturating/checked_* family — already PARK/OUT in `LOGIC-INT-ARITH-OUT.md`. Option/Result control flow stays F1/F2.

**What Logic *does* own:** relational `fact`/`rule`/`query` (path, join, multi-query fail E0301). Symbols are opaque atoms, not `i64`.

## Possible future facts/queries (if ever bridged — not v0)

Only after a dedicated ADR (“numeric atoms in Logic” or “host builtins”):

1. **Ground tables, not methods:** e.g. precompute `fact multiple(6, 3)` / `fact positive(5)` as *data*, then `query multiple(6, 3)` — still no `n % d` inside the engine.
2. **Classification rules over symbolic tags:** `fact sign(n, pos)` emitted by Imperativo → Logic consumes tags (no recomputation).
3. **Never:** lower `ilog`/`is_multiple_of` call sites into TML/Tau or `arita-logic` builtins in v0 (would couple isla to Int semantics + Option).

## Recommendation

Keep ADR-106 / 117 / 121 / 122 as **surface-only** oracles in measure Imperativo. Logic idle on this wave; next Logic CUTs stay relational (corpus/E0301), not numeric predicates.
