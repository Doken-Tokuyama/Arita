Translation of `028-perf-v0.md`; the original is normative. / Traducción de `028-perf-v0.md`; el original es el normativo.

# ADR-028 — Perf v0 (perfiles emit, Phase 4b)

- **Estado:** **aceptada** + **IMPL verified** (Lex measure **66/66** accepted)
- **CUT-ID:** `PERF-V0-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft/align) + Ingeniero Rust (**GO DOC+IMPL**)
- **Relacionados:** ADR-001 (emit-Rust), ADR-022 / E0231 (safe-only), ADR-027 (async verified 63/63), ROADMAP Fase 4b Perf
- **Gobernanza:** **aceptada**. CUT IMPL autorizado (`PERF-V0-20260913`). Executor Ingeniero (stack completo); Parser/Codegen HOLD salvo review. Sin theater de tiempos. Nombre = **ARITA**.
- **Barra:** parse → emit → rustc → execute + `clippy -D warnings`; skip ≠ PASS; inconclusive ≠ accepted.

## Context

Phase 4b (remainder after Async): tuning **without** hacking bytes ni `unsafe`. Allowed path = emit profiles, real flags in the generated crate, oracles E2E that exercise the profile — not microbench theater.

Async v0 (ADR-027) ya **verified** Lex 63/63.

## Decision (MVP)

### 1. Perfiles IN

| Perfil | Default | Emit Cargo (crate generado) |
|--------|---------|------------------------------|
| `debug` | **yes** | Cargo profile `dev` / without forced `opt-level` (current baseline) |
| `release` | no | `[profile.release] opt-level = 3`; **LTO off** (no `lto = true` / fat) |

Future `perf` (LTO / codegen-units) = **OUT** of v0 — ROADMAP mention only.

### 2. Surface / CLI (pin cerrado)

- Flag CLI canonical: `arita build --profile debug|release`.
- Default without flag = **`debug`**.
- `profile` en `arita.toml` = **OUT de v0** (future).
- **No** keywords/`unsafe`/intrinsics in `.arita` to “go faster”.

### 3. Oracles measure (E2E reales)

| Id | What it tests | Expect |
|----|------------|--------|
| `perf-01-release-run` | Small program build+run with `--profile release` | stdout fijado en GO IMPL (anti-theater) |
| `perf-02-release-optlevel` | Evidence en **Cargo.toml generado**: `[profile.release] opt-level = 3` | parse/check of the manifiesto emitido (no sidecar meta; no smoke) |
| `perf-neg-e0250` (optional) | `--profile fantasma` / desconocido | **E0250** `unknown build profile` |

Corpus: **`ejemplos/perf/`** dedicated (no reuse paths `ejemplos/f2/`).

**Time / wall-clock** thresholds: **OUT of v0**. If measured in the future: host without a reliable clock → `inconclusive`, never `accepted` (ROADMAP). v0 does not invent PASS for “it was fast”.

### 4. Diagnostics

| Code | Message EN canonical (draft) | When |
|--------|----------------------------|--------|
| **E0250** | `unknown build profile` | perfil ≠ `debug`\|`release` |

No colisiona E024x (async) ni E023x (unsafe/contract).

### 5. OUT

- `unsafe` / FFI / `asm` / SIMD / byte-hacking en surface o emit de usuario
- Perfil `perf` fantasma without real flags en Cargo
- LTO fat / `codegen-units = 1` as default v0
- Mutate Clippy policy (`-D warnings`) to “make perf pass”
- Oracles de latencia with umbrales inventados

### 6. Safe-only

ADR-022 / E0231: emit usuario `#![forbid(unsafe_code)]`. Perf no es escape hatch.

### 7. Forma (CLI)

```bash
arita build ejemplos/f2/01-let-int.arita
arita build --profile release ejemplos/perf/01-release-hello.arita
```

Paths bajo `ejemplos/perf/`; crear en IMPL with stdout fijado.

## Decisiones cerradas (2026-09-13 — Ingeniero GO)

1. **CLI-only** v0: `arita build --profile debug|release` (default debug). `profile` en `arita.toml` = OUT v0 / future.
2. Evidence opt-level: **Cargo.toml generado** (`[profile.release] opt-level = 3`); **no** sidecar meta.
3. Oracles en **`ejemplos/perf/`** dedicated (no reuse paths f2).

## Consequences

- **Verify 2026-09-13:** `arita measure` Lex **accepted 66/66** (PERF-V0 landed). Parser/Codegen: HOLD → **review-only**.
- CUT IMPL autorizado: executor Ingeniero (full stack, pattern ASYNC-V0).
- Parser/Codegen: HOLD crates salvo review-only.
- ROADMAP Phase 4b Perf: this ADR **accepted**.
- PACK: no few-shot perf until oracles E2E verdes.

## Checklist GO

- [x] Perfiles `debug` / `release` + `opt-level = 3` OK
- [x] LTO off v0 OK
- [x] Umbrales timed OUT v0 OK
- [x] E0250 OK
- [x] Abiertas 1–3 **cerradas**
- [x] GO DOC → **accepted** + IMPL autorizado
- [x] Oracles `ejemplos/perf/` E2E + measure wire
