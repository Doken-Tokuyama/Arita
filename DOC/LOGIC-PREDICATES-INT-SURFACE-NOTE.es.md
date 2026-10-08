Español | [English](LOGIC-PREDICATES-INT-SURFACE-NOTE.md)

# Lógica vs predicados de surface — is_multiple_of / is_positive|negative / ilog (Option)

**CUT:** analysis-only (<person> / Ingeniero, 2026-09-18)  
**Bar context:** Lex **314/314** (surface). **No landings** this note.  
**Pins:** ADR-012 (isla F3 Datalog +); `DOC/LOGIC-INT-ARITH-OUT.md` (Int arith OUT of Logic v0).

## Veredicto (una línea)

**Solo surface (Imperativo / std Int).** No entran en la isla Tau/TML-shaped v0 como builtins del motor; un bridge futuro sería ADR nuevo, no bajar métodos `Int` a `arita-logic`.

## IN / OUT vs Logic

| Predicado (surface) | Tipo | Logic v0 | Por qué |
|---------------------|------|----------|--------|
| `is_positive` / `is_negative` | `Int → Bool` (ADR-106) | **OUT** | Predicado aritmético sobre magnitud; Logic v0 no tiene sort `Int` ni evaluación numérica |
| `is_multiple_of` | `Int×Int → Bool` (ADR-117; `d==0 → false`) | **OUT** | Divisibilidad / rem; motor F3 es Herbrand positivo, no aritmética |
| `ilog2` / `ilog10` | `Int → Option<Int>` (ADR-121/122) | **OUT** | Log entero + **Option** (failure channel surface); Logic no hostea Option/Result ni partial numeric functions |

**También OUT (relacionado):** familia wrapping/saturating/checked_* — ya PARK/OUT en `LOGIC-INT-ARITH-OUT.md`. El control de flujo Option/Result se queda en F1/F2.

**Lo que Logic *sí* posee:** `fact`/`rule`/`query` relacionales (path, join, multi-query fail E0301). Los símbolos son átomos opacos, no `i64`.

## Posibles facts/queries futuros (si algún día se puentea — no v0)

Solo tras un ADR dedicado (“numeric atoms in Logic” o “host builtins”):

1. **Tablas ground, no métodos:** p. ej. precálculo `fact multiple(6, 3)` / `fact positive(5)` como *data*, luego `query multiple(6, 3)` — sigue sin `n % d` dentro del motor.
2. **Reglas de clasificación sobre etiquetas simbólicas:** `fact sign(n, pos)` emitido por Imperativo → Logic consume etiquetas (sin recomputar).
3. **Nunca:** bajar call sites de `ilog`/`is_multiple_of` a builtins TML/Tau o `arita-logic` en v0 (acoplaría la isla a la semántica Int + Option).

## Recomendación

Mantener ADR-106 / 117 / 121 / 122 como oráculos **solo-surface** en measure Imperativo. Logic idle en esta oleada; los siguientes CUT de Logic siguen siendo relacionales (corpus/E0301), no predicados numéricos.
