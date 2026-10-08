# ADR-027 — Async v0 (Fase 4b)

- **Estado:** **aceptada** + **IMPL verified** (Lex measure **73/73** accepted; async + `neg-e0240`/`neg-e0241`)
- **CUT-ID:** `ASYNC-V0-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft/align) + Ingeniero Rust (**GO DOC+IMPL**)
- **Relacionados:** ADR-022 / E0231 (safe-only), ADR-001 (emit-Rust), ADR-006/026 (std), ROADMAP Fase 4b / 4c
- **Gobernanza:** **aceptada**. CUT IMPL autorizado (`ASYNC-V0-20260913`). Sin ejemplos decorativos; oráculos E2E reales. Nombre = **ARITA**. Safe-only (ADR-022 / E0231).
- **Barra:** parse → HIR → emit → rustc → execute + `clippy -D warnings`; skip ≠ PASS; inconclusive ≠ accepted.

## Contexto

Fase 4b pide async **safe-only**: surface ARITA acotado, runtime Rust solo en el **crate generado** (puente), alineado con Fase 4c (deps whitelist / bridges). **Prohibido** “pegar tokio” (u otro runtime) como API cruda en `.arita`.

ADR-006 listaba `async` como OUT de F2 — este ADR lo abre en **Fase 4b**, no en F2.

Std mínima (ADR-026) se asume **aceptada** (`len` / `is_empty`).

## Decisión (MVP)

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
| `async fn name(…) -> Ret` | Ret v0: **`Io<()>`** únicamente |
| `async fn main() -> Io<()>` | **Entrypoint canónico** v0 (pin cerrado) |
| `await <expr>` | Solo **dentro** de `async fn`; `<expr>` = call a `async fn` del mismo módulo (v0) |
| Efectos | `print` / std F2+026 siguen válidos dentro de `async fn` |
| Scrutinee / ownership | Mismas reglas F2 (move/borrow); `await` no introduce `unsafe` |

### 2. Emit / runtime (bridge — no surface) — pins cerrados

1. El backend genera un crate Cargo con dependencia runtime **solo en el manifiesto generado**.
2. **Ningún** identificador `tokio` / `async_std` / `smol` aparece en `.arita`.
3. **Entrypoint:** surface `async fn main() -> Io<()>` → emit **`#[tokio::main(flavor = "current_thread")]`** + `async fn main` (forma natural para IAs). Glue **`block_on` = OUT de v0** salvo que measure lo fuerce (entonces documentar en CUT IMPL).
4. **Runtime pin (Cargo emitido):**
   ```toml
   tokio = { version = "=1.53.1", default-features = false, features = ["rt", "macros"] }
   ```
   Flavor runtime: **current_thread** (determinismo / Miri-friendly). **`rt-multi-thread` = OUT** de v0. (Versión: última estable 1.x en crates.io al pin 2026-09-13; bump solo con CUT.)
5. Programa usuario: `#![forbid(unsafe_code)]` (ADR-022). Clippy `-D warnings` en measure.
6. Fase 4c: este runtime cuenta como **dep whitelist del bridge**, no como `use` libre.

### 3. Diagnósticos draft

| Código | Mensaje EN canónico (draft) | Cuándo |
|--------|----------------------------|--------|
| **E0240** | `await outside async function` | `await` en `fn` sync / top-level |
| **E0241** | `async feature not allowed here` | `async` en item no fn / contexto ilegal v0 |

Reusar E0203 para type/arity; E0231 si aparece unsafe/FFI. **E0240/E0241** = rango async v0 (pin cerrado; no roza E023x).

### 4. OUT (v0)

- Runtime multi-thread / `rt-multi-thread`
- `block_on` como surface o glue v0 (OUT salvo force measure)
- `spawn`, tasks libres, `JoinHandle`
- channels / `select!` / `JoinSet`
- `async` traits, generics async, lifetimes en futures
- Timers / IO de red / filesystem async (salvo CUT std async posterior)
- Nombrar runtime en surface (`tokio::…`)
- `unsafe`, FFI, raw pointers
- Mezclar isla F3 (`spec`/`query`) con `async` en el mismo módulo (módulos separados; pin)

### 5. Oráculos (tras IMPL GO)

| Id | Path | Expect |
|----|------|--------|
| `async-01-await-greet` | `ejemplos/async/01-await-greet.arita` | stdout contiene `hi` y `done` (orden fijado en GO) |
| `async-02-async-print` | `ejemplos/async/02-async-print.arita` | un `async fn` + `await` mínimo |
| neg | `ejemplos/async/neg/e0240-await-outside.arita` | **E0240** |
| neg | `ejemplos/async/neg/e0241-async-illegal.arita` | **E0241** |

Wire a `arita measure` solo con E2E real + clippy.

### 6. PACK

`DOC/PACK-ASYNC-FEWSHOT.md` solo tras oráculos verdes; cargar **solo** si la tarea pide async.

## Decisiones cerradas (2026-09-13 — Ingeniero / <person> decide-y-sigue)

1. **Entrypoint:** `async fn main() -> Io<()>` → emit `#[tokio::main(flavor = "current_thread")]`. **`block_on` OUT de v0** (salvo force measure).
2. **Runtime:** `tokio = "=1.53.1"` features `rt` + `macros`; **current_thread**. Multi-thread OUT.
3. **Códigos:** **E0240 / E0241** OK (E024x async; no choca E0231).

## Consecuencias

- Parser/HIR/Codegen: **IMPL GO** activo. Docs índice → aceptada. Measure async oracles cuando existan E2E reales.
- ADR-006: `async` deja de ser OUT global → “OUT de F2; IN Fase 4b vía ADR-027”.
- Perf (ADR futuro) y Deps 4c no bloquean este MVP surface.

## Checklist GO

- [x] Surface `async fn` + `await` OK
- [x] Ret v0 = `Io<()>` OK
- [x] Bridge runtime / no tokio en `.arita` OK
- [x] E0240/E0241 pin OK
- [x] Decisiones 1–3 **cerradas**
- [x] GO DOC → **aceptada** (std-min measure `accepted: 60`)
- [x] CUT IMPL autorizado (Ingeniero)
- [x] Oráculos `ejemplos/async/` E2E + measure Lex **63/63** accepted

## Addendum (ADR-039)

**E0242** `borrow held across await` — CUT `TRAPS-BORROW-AWAIT-20260915`. No sobrecarga E0202. Ver `DOC/ADR/039-e0242-borrow-across-await.md`.
