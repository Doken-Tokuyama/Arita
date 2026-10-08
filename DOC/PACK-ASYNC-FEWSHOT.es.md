[Original](PACK-ASYNC-FEWSHOT.md) | Español | [English](PACK-ASYNC-FEWSHOT.en.md)

# PACK Async — Few-shots para IAs (`async fn` / `await`)

- **Estado:** **final** (GO Ingeniero Rust, 2026-09-14)
- **Fecha:** 2026-09-14
- **CUT surface:** `ASYNC-V0-20260913` (`DOC/ADR/027-async-v0.md`)
- **Deps (opcional):** ADR-029 / CUT `DEPS-V0-20260914` — manifiesto `[deps] tokio` bridge only; **no** `tokio::` en surface
- **Barra:** solo programas reales E2E bajo `ejemplos/async/` (+ opcional `ejemplos/deps/`). skip ≠ PASS. Sin theater.
- **Uso:** cargar este PACK **solo** cuando la tarea pida async. Default IA = F1.1/F2; no mezclar con F3.

## Uso

Pegar el **system prompt** + 3–7 turnos few-shot cuando la tarea pida `async fn` / `await`.
Los bloques `.arita` son copias verbatim de archivos que **ya** pasan oráculo E2E (`arita measure`).

```bash
# positivos (build + run)
cargo run -p arita-cli -- build ejemplos/async/01-await-greet.arita
./target/arita-out/await_greet
# stdout: hi / done

cargo run -p arita-cli -- build ejemplos/async/02-async-print.arita
./target/arita-out/async_print
# stdout: ok

# negativo (must FAIL parse_lower_check with code)
cargo run -p arita-cli -- parse ejemplos/async/neg/e0240-await-outside.arita
cargo run -p arita-cli -- parse ejemplos/async/neg/e0241-async-illegal.arita
# → E0240: await outside async function

# opcional deps bridge (ADR-029; no requerido para emit async hoy)
cargo run -p arita-cli -- build ejemplos/deps/01-tokio-bridge.arita
# stdout: hi / deps-ok  (requiere ejemplos/deps/arita.toml)
```

## System prompt (canonical, English for the model)

```text
You write ARITA async v0 (Fase 4b / ADR-027). Load this pack only when the task asks for async.

Allowed surface:
  module <ident>

  async fn <name>(…) -> Io<()> { … }
  async fn main() -> Io<()> { … }   // canonical entrypoint v0

  await <call>                      // only inside async fn; call = async fn same module

  // Effects inside async fn: print / F2+std-min as usual

Rules:
- Ret v0 for async fn / async main = Io<()> only.
- No tokio:: / async_std / smol paths in .arita (runtime is emit bridge only).
- Safe-only (ADR-022 / E0231): no unsafe, no FFI, no raw pointers.
- OUT v0: spawn, channels, select!, JoinHandle, JoinSet, async traits, timers/net/fs async, block_on as surface, rt-multi-thread, F3 logic mixed in same module.
- E0240: await outside async function. E0241: async feature not allowed here.
- Never invent PASS. skip ≠ PASS. Evidence = real arita build+run / parse reject for neg.

Optional (ADR-029): arita.toml may declare
  [deps]
  tokio = { bridge = "async-runtime" }
This formalizes the async emit bridge (tokio-only whitelist). Not required for async emit today. Still never write tokio:: in .arita (E0261). Unknown crates → E0260.

Canonical positives:
  ejemplos/async/01-await-greet.arita  → stdout hi then done  (async-01-await-greet)
  ejemplos/async/02-async-print.arita  → stdout ok            (async-02-async-print)

Canonical negatives:
  ejemplos/async/neg/e0240-await-outside.arita → E0240
  ejemplos/async/neg/e0241-async-illegal.arita → E0241

Optional deps oracles:
  ejemplos/deps/01-tokio-bridge.arita (+ arita.toml) → hi / deps-ok
  ejemplos/deps/neg/e0260-unknown-crate/ → E0260
  ejemplos/deps/neg/e0261-crate-path.arita → E0261

When asked for async, output a complete .arita matching these patterns.
If the task is F1.1/F2/F2.2/F3 without async, use the matching PACK instead.
```

## Códigos útiles (resumen)

### Async v0 (ADR-027)

| Code | English (canonical) |
|------|---------------------|
| **E0240** | `await outside async function` |
| **E0241** | `async feature not allowed here` |

### Deps bridge (ADR-029 — optional note)

| Code | English (canonical) |
|------|---------------------|
| **E0260** | `dependency not in ARITA whitelist` |
| **E0261** | `external crate path not allowed in surface` |

(E0231 unsafe/FFI unchanged. F2 E02xx / E021x unchanged.)

## Few-shot 1 — await helper (`ejemplos/async/01-await-greet.arita`)

**User:** ARITA async v0: async greet prints "hi"; async main awaits it then prints "done".

**Assistant:**

```arita
// ADR-027 async v0 — await helper then print
module await_greet

async fn greet() -> Io<()> {
  print("hi")
}

async fn main() -> Io<()> {
  await greet()
  print("done")
}
```

Path: `ejemplos/async/01-await-greet.arita`. Stdout oracle: `hi` then `done` (`async-01-await-greet`).

## Few-shot 2 — minimal async print (`ejemplos/async/02-async-print.arita`)

**User:** Minimal async: async say prints "ok"; async main awaits it.

**Assistant:**

```arita
// ADR-027 async v0 — minimal async + await
module async_print

async fn say() -> Io<()> {
  print("ok")
}

async fn main() -> Io<()> {
  await say()
}
```

Path: `ejemplos/async/02-async-print.arita`. Stdout oracle: `ok` (`async-02-async-print`).

## Few-shot 3 — refuse await outside async (`ejemplos/async/neg/e0240-await-outside.arita`)

**User:** Can I `await` a call from a sync `fn main`?

**Assistant:** No — that must fail with **E0240**. Do not emit this as a solution:

```arita
// Expected: E0240 await outside async function
module e0240_await_outside

fn greet() -> Io<()> {
  print("hi")
}

fn main() -> Io<()> {
  await greet()
  print("x")
}
```

Path: `ejemplos/async/neg/e0240-await-outside.arita`.  
Oracle: `arita parse` → **E0240** (`await outside async function`). Use `async fn main` + `await` inside async (few-shot 1/2) instead.

## Few-shot 3b — refuse illegal `async` (`ejemplos/async/neg/e0241-async-illegal.arita`)

**User:** Can I write `async print("x")` inside a sync `fn main`?

**Assistant:** No — that must fail with **E0241**. Do not emit this as a solution:

```arita
// Expected: E0241 async feature not allowed here
module e0241_async_illegal

fn main() -> Io<()> {
  async print("x")
}
```

Path: `ejemplos/async/neg/e0241-async-illegal.arita`.  
Oracle: `arita parse` → **E0241** (`async feature not allowed here`). Only `async fn` is legal; never `async` on other stmts/items.

## Few-shot 4 — optional tokio bridge manifest (`ejemplos/deps/01-tokio-bridge.arita`)

**User:** Same async greet/print, but declare the tokio async-runtime bridge in `arita.toml` (ADR-029).

**Assistant:** Surface stays tokio-free. Pair with sibling `arita.toml`:

`arita.toml`:

```toml
# ADR-029 DEPS-V0 — whitelist tokio bridge (formalizes ADR-027)
[package]
name = "deps_tokio_bridge"

[deps]
tokio = { bridge = "async-runtime" }
```

`.arita`:

```arita
// ADR-029 DEPS-V0 — explicit [deps] tokio bridge; E2E async greet/print
module tokio_bridge

async fn greet() -> Io<()> {
  print("hi")
}

async fn main() -> Io<()> {
  await greet()
  print("deps-ok")
}
```

Path: `ejemplos/deps/01-tokio-bridge.arita` + `ejemplos/deps/arita.toml`.  
Stdout oracle: `hi` / `deps-ok` (`deps-01-tokio-bridge`).  
Note: `[deps] tokio = { bridge = "async-runtime" }` is **allowed** but **not required** for async emit today (ADR-027 still emits the runtime bridge without the manifest).

## Counter-examples (do NOT emit)

| File | Code | Why |
|------|------|-----|
| `ejemplos/async/neg/e0240-await-outside.arita` | **E0240** | `await` in sync `fn` |
| `ejemplos/async/neg/e0241-async-illegal.arita` | **E0241** | `async` on non-fn / illegal context v0 |
| `ejemplos/deps/neg/e0260-unknown-crate/` | **E0260** | crate outside whitelist |
| `ejemplos/deps/neg/e0261-crate-path.arita` | **E0261** | `tokio::…` path in `.arita` |

Also refuse: `spawn` / channels / `select!`, naming runtime in surface, `unsafe`, mixing F3 `spec`/`query` with `async` in one module, decorative PASS without oracles.

### Nota corta deps (ADR-029)

```toml
[deps]
tokio = { bridge = "async-runtime" }
```

Whitelist v0 = **tokio-only**. No new surface APIs. Async programs work without this manifest today; the entry only formalizes the emit bridge.

## Checklist GO (Ingeniero Rust) — cerrado

- [x] System prompt async OK (`async fn` + `await`; Ret `Io<()>`; no tokio paths; safe-only; no spawn/channels/select)
- [x] Few-shots = real `ejemplos/async/01`–`02` + neg E0240/E0241 (+ optional deps-01) only
- [x] E0240/E0241 + optional E0260/E0261 table OK
- [x] Counter-examples / no fake PASS OK
- [x] Separación: cargar **solo** si la tarea pide async OK
- [x] GO → estado **final** + Docs índice + ROADMAP

## Enlaces

- `DOC/ADR/027-async-v0.md` (CUT `ASYNC-V0-20260913`)
- `DOC/ADR/029-deps-v0.md` (CUT `DEPS-V0-20260914`; bridge opcional)
- `DOC/ADR/022-safe-only.md`, `DOC/ADR/024-e0231-unsafe-reject.md`
- `ejemplos/async/`, `ejemplos/async/neg/`, `ejemplos/deps/`
- `DOC/PACK-F2-FEWSHOT.md`, `DOC/PACK-F2.2-FEWSHOT.md`, `DOC/PACK-F3-FEWSHOT.md`
