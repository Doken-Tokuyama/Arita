# Guía — Option/Result helpers v0

**Audiencia:** humanos. Barras desde ADRs / `STABLE_VERIFY` — no inventar PASS.

## Helpers v0

| API | Tipos | Arity | Ret | Semántica |
|-----|-------|------:|-----|-----------|
| `unwrap_or(d)` | Option\<T\>, Result\<T,E\> | 1 | `T` | Some/Ok → valor; None/Err → `d` |
| `is_some()` | Option | 0 | Bool | |
| `is_none()` | Option | 0 | Bool | |
| `is_ok()` | Result | 0 | Bool | |
| `is_err()` | Result | 0 | Bool | |

## Ejemplos

```arita
let x: Int = opt.unwrap_or(0)
let y: Int = res.unwrap_or(-1)
let b1: Bool = opt.is_some()
let b2: Bool = res.is_err()
```

## Verificado Lex

- ADR-093 `unwrap_or` → **229/229**
- ADR-094 `is_some`/`is_none`/`is_ok`/`is_err` → **234/234**

## OUT v0

`unwrap` / `expect` / `?` / `unwrap_or_else` / `is_some_and` — PARK.

## Neg

Receptor Int (u otro no-Option/Result) → **E0206**.

## Relación con swallow traps

`unwrap_or` es default **explícito** — no colisiona con E0272/E0274 (match-arm swallow theater).

## Enlaces

- Pack IA: [PACK-STD-METHOD-SURFACE-F2.md](../PACK-STD-METHOD-SURFACE-F2.md)
- ADRs: [093](../ADR/093-std-unwrap-or-v0.md), [094](../ADR/094-std-is-some-ok-v0.md)
- Guía arith: [aritmetica-int-v0.md](aritmetica-int-v0.md)
