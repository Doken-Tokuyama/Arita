Translation of `244-core-timeout-cancel-v0.md`; the original is normative. / Traducción de `244-core-timeout-cancel-v0.md`; el original es el normativo.

# ADR-244 — Core 0.2 TIMEOUT-CANCEL (slice 3)

- **Estado:** **CLOSED** Lex **611/611** (CUT `CORE-0.2-TIMEOUT-CANCEL-20260920`)
- **Gate:** `../GATE-CORE02-TIMEOUT-CANCEL-20260920.md` (not in the public export / no incluido en el export público)
- **Barra:** `arita measure` → **611/611 accepted** (exit 0); oracles timeout-ok / timeout-err / cancel-cooperative / `neg-hang-theater` (**E0320**)
- **CUT-ID:** `CORE-0.2-TIMEOUT-CANCEL-20260920`
- **Fecha:** 2026-09-20 (Europe/Madrid)
- **Autores:** ARITA Codegen (IMPL) · Arquitecto (pins ADR-233) · Ingeniero (**GO IMPL** 2026-09-20, reconfirm land crates)
- **Prereq pins:** [ADR-233](233-core-0.2-pins.en.md) slice **3** · [ADR-027](027-async-v0.md) bridge tokio
- **Prereq slice 2:** [ADR-243](243-core-task-spawn-v0.en.md) TASK-SPAWN — Orchestrator/Engineer: **CLOSED** Lex **607/607** (GATE spawn). Codegen preserves spawn arms; **no** whole-`lib.rs` overwrite.
- **Gobernanza:** Ingeniero GO IMPL **supersedes** any prior ADR-244 «NO GO IMPL land» / HOLD-against-IMPL wording. skip ≠ PASS; only **CLOSED** remains gated (measure + Ingeniero). `Mutex` / `[]` / repair IMPL = **HOLD**.
- **SoT Lex:** <machine-id-label> `<machine-id>` · `<repo>`

## Objective

Typed surface **`timeout(ms, async …) -> Result`** + **cooperative cancel** → typed `Err`; emit tokio `time` bridge (+ curated cancel token); **no** `tokio::` in `.arita`.

## 1. Surface IN (v0 land)

| Construct | v0 rule |
|-----------|---------|
| `timeout(ms, <async_call\|delay\|until_cancelled>)` | `ms: Int`; return **`Result<(), String>`**; elapsed → `Err("timeout")` |
| `delay(ms)` | curated sleep (emit `tokio::time::sleep`) — **no** busy-spin |
| `cancel_token()` | curated token (`Arc<AtomicBool>`) as `Int`-typed handle in surface v0 |
| `cancel(token)` / `until_cancelled(token)` | cooperative → completes when cancelled |
| Timeout/cancel outside async | **E0313** |
| Hang theater (`busy_spin` / `hang_forever`) | **E0320** (Lex **E0314** unknown field / **E0317** unknown variant = unknown field CORE-0.1 record — do not collide) |
| Runtime | Single emitted bridge; **no** `tokio::…` on surface |

## 2. Emit

- `tokio::time::timeout(Duration::from_millis(ms), body).await` → map `Ok` / `Err(Elapsed)` → `Result`
- Tokio feature **`time`** (+ `sync` for token atomics) on top of `rt`/`macros` (pin `=1.53.1`)
- Cancel: allowlisted `Arc<AtomicBool>` — **cooperative**, no AbortForce surface
- `#![forbid(unsafe_code)]` user crates; **no** `unwrap` in emit surface
- **Preserve** existing spawn emit arms (ADR-243) — surgical merge only

## 3. Diagnostics

| Code | Meaning |
|------|---------|
| E0313 | timeout/delay/cancel misuse (arity, types, outside async) |
| E0314 | **reserved** — unknown field (CORE-0.1 record) on Lex |
| E0315 | missing field (CORE-0.1) |
| **E0320** | hang theater / busy_spin / hang_forever |

## 4. OUT / HOLD

| OUT | Reason |
|-----|--------|
| Hang theater / busy-spin | **E0320** + neg oracle |
| Busy sleep substituting real timeout | ADR-233 |
| `Mutex` / sugar `[]` / repair IMPL | HOLD |
| `tokio::` in `.arita` | ADR-027 |
| Reopen slice 3 / unpark HOLD | forbidden without new GO |

## 5. Oracles (accepted Lex 611/611)

| Class | Id | Path | Expect |
|-------|----|------|--------|
| pos | `core02-timeout-ok-fast` | `ejemplos/core02/01-timeout-ok-fast.arita` | Ok before ms |
| pos | `core02-timeout-err-elapsed` | `ejemplos/core02/02-timeout-err-elapsed.arita` | typed timeout Err |
| pos | `core02-cancel-cooperative` | `ejemplos/core02/03-cancel-cooperative.arita` | cancel path |
| neg | `neg-hang-theater` | `ejemplos/core02/neg/hang-theater.arita` | **E0320 rejected** |

## 6. Close (2026-09-20)

- Measure Lex **611/611** + Engineer GO (gate).
- Docs sign **CLOSED**. HOLDs intact.
- Next: ADR-233 **GO** slice 4 HTTP-BINDINGS.

## 7. Codegen checklist

- [x] Surface timeout/delay/cancel_* via `user_call` (spawn_call style — no raw tokio in surface)
- [x] HIR E0313 misuse + **E0320** hang theater
- [x] Emit tokio `time::timeout` + cancel token; forbid unsafe; no unwrap
- [x] Unit tests hir/codegen timeout ok / outside-async / hang theater
- [x] Examples `ejemplos/core02/`
- [x] ADR-244 → **CLOSED** Lex **611/611**
- [x] Lex SoT land + measure (gate)
- [x] Measure Lex slice 3 — **611/611**
- [x] CLOSED ADR-244 — Engineer GO + Docs sign

## Queue

Slice 3 CLOSED. Codegen/Architect: slice 4 HTTP-BINDINGS under Engineer GO. Docs: no global Core 0.2 CLOSED.
