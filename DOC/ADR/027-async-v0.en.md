Translation of `027-async-v0.md`; the original is normative. / Traducción de `027-async-v0.md`; el original es el normativo.

# ADR-027 — Async v0 (Phase 4b)

- **Estado:** **aceptada** + **IMPL verified** (Lex measure **73/73** accepted; async + `neg-e0240`/`neg-e0241`)
- **CUT-ID:** `ASYNC-V0-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft/align) + Ingeniero Rust (**GO DOC+IMPL**)
- **Relacionados:** ADR-022 / E0231 (safe-only), ADR-001 (emit-Rust), ADR-006/026 (std), ROADMAP Fase 4b / 4c
- **Gobernanza:** **aceptada**. CUT IMPL autorizado (`ASYNC-V0-20260913`). Sin ejemplos decorativos; oráculos E2E reales. Nombre = **ARITA**. Safe-only (ADR-022 / E0231).
- **Barra:** parse → HIR → emit → rustc → execute + `clippy -D warnings`; skip ≠ PASS; inconclusive ≠ accepted.

## Context

Phase 4b asks for async **safe-only**: bounded ARITA surface, runtime Rust only in the **crate generado** (puente), aligned with Phase 4c (deps whitelist / bridges). **Forbidden** “pegar tokio” (u otro runtime) as API cruda en `.arita`.

ADR-006 listed `async` as OUT of F2 — this ADR opens it in **Phase 4b**, not in F2.

Minimal std (ADR-026) is assumed **accepted** (`len` / `is_empty`).

## Decision (MVP)

### 1. Surface IN

```arita
module demo

async fn greet() -> Io<()> {
  print("hi")
}

async fn main() -> Io<()> {
  await greet()
  print("done")
}
```

| Constructo | Regla v0 |
|------------|----------|
| `async fn name(…) -> Ret` | Ret v0: **`Io<()>`** only |
| `async fn main() -> Io<()>` | **Entrypoint canonical** v0 (pin cerrado) |
| `await <expr>` | Solo **inside** de `async fn`; `<expr>` = call a `async fn` of the same module (v0) |
| Efectos | `print` / std F2+026 remain valid inside `async fn` |
| Scrutinee / ownership | Mismas reglas F2 (move/borrow); `await` no introduce `unsafe` |

### 2. Emit / runtime (bridge — no surface) — pins cerrados

1. El backend genera un crate Cargo with dependencia runtime **only in the manifiesto generado**.
2. **No** `tokio` / `async_std` / `smol` identifier appears in `.arita`.
3. **Entrypoint:** surface `async fn main() -> Io<()>` → emit **`#[tokio::main(flavor = "current_thread")]`** + `async fn main` (forma natural for IAs). Glue **`block_on` = OUT de v0** salvo que measure lo fuerce (entonces documentar en CUT IMPL).
4. **Runtime pin (Cargo emitido):**
   ```toml
   tokio = { version = "=1.53.1", default-features = false, features = ["rt", "macros"] }
   ```
   Runtime flavor: **current_thread** (determinism / Miri-friendly). **`rt-multi-thread` = OUT** of v0. (Version: latest stable 1.x on crates.io at pin 2026-09-13; bump only with CUT.)
5. Programa usuario: `#![forbid(unsafe_code)]` (ADR-022). Clippy `-D warnings` en measure.
6. Phase 4c: this runtime counts as **bridge dep whitelist**, not as free `use`.

### 3. Diagnostics draft

| Code | Message EN canonical (draft) | When |
|--------|----------------------------|--------|
| **E0240** | `await outside async function` | `await` en `fn` sync / top-level |
| **E0241** | `async feature not allowed here` | `async` on non-fn item / illegal v0 context |

Reusar E0203 for type/arity; E0231 si aparece unsafe/FFI. **E0240/E0241** = rango async v0 (pin cerrado; no roza E023x).

### 4. OUT (v0)

- Runtime multi-thread / `rt-multi-thread`
- `block_on` as surface o glue v0 (OUT salvo force measure)
- `spawn`, tasks libres, `JoinHandle`
- channels / `select!` / `JoinSet`
- `async` traits, generics async, lifetimes en futures
- Timers / IO de red / filesystem async (salvo CUT std async later)
- Nombrar runtime en surface (`tokio::…`)
- `unsafe`, FFI, raw pointers
- Mix F3 island (`spec`/`query`) with `async` in the same module (separate modules; pin)

### 5. Oracles (after IMPL GO)

| Id | Path | Expect |
|----|------|--------|
| `async-01-await-greet` | `ejemplos/async/01-await-greet.arita` | stdout contiene `hi` y `done` (orden fijado en GO) |
| `async-02-async-print` | `ejemplos/async/02-async-print.arita` | un `async fn` + `await` minimum |
| neg | `ejemplos/async/neg/e0240-await-outside.arita` | **E0240** |
| neg | `ejemplos/async/neg/e0241-async-illegal.arita` | **E0241** |

Wire a `arita measure` only with E2E real + clippy.

### 6. PACK

`DOC/PACK-ASYNC-FEWSHOT.md` only after oracles verdes; cargar **only** si la tarea asks for async.

## Decisiones cerradas (2026-09-13 — Ingeniero / <person> decide-y-remains)

1. **Entrypoint:** `async fn main() -> Io<()>` → emit `#[tokio::main(flavor = "current_thread")]`. **`block_on` OUT de v0** (salvo force measure).
2. **Runtime:** `tokio = "=1.53.1"` features `rt` + `macros`; **current_thread**. Multi-thread OUT.
3. **Codes:** **E0240 / E0241** OK (E024x async; no choca E0231).

## Consequences

- Parser/HIR/Codegen: **IMPL GO** active. Docs index → accepted. Measure async oracles when real E2E exist.
- ADR-006: `async` deja de ser OUT global → “OUT de F2; IN Phase 4b via ADR-027”.
- Perf (future ADR) and Deps 4c do not block this MVP surface.

## Checklist GO

- [x] Surface `async fn` + `await` OK
- [x] Ret v0 = `Io<()>` OK
- [x] Bridge runtime / no tokio en `.arita` OK
- [x] E0240/E0241 pin OK
- [x] Decisiones 1–3 **cerradas**
- [x] GO DOC → **accepted** (std-min measure `accepted: 60`)
- [x] CUT IMPL autorizado (Ingeniero)
- [x] Oracles `ejemplos/async/` E2E + measure Lex **63/63** accepted

## Addendum (ADR-039)

**E0242** `borrow held across await` — CUT `TRAPS-BORROW-AWAIT-20260915`. No sobrecarga E0202. Ver `DOC/ADR/039-e0242-borrow-across-await.md`.
