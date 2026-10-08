# ADR-028 — Perf v0 (perfiles emit, Fase 4b)

- **Estado:** **aceptada** + **IMPL verified** (Lex measure **66/66** accepted)
- **CUT-ID:** `PERF-V0-20260913`
- **Fecha:** 2026-09-13
- **Autores:** ARITA Arquitecto (draft/align) + Ingeniero Rust (**GO DOC+IMPL**)
- **Relacionados:** ADR-001 (emit-Rust), ADR-022 / E0231 (safe-only), ADR-027 (async verified 63/63), ROADMAP Fase 4b Perf
- **Gobernanza:** **aceptada**. CUT IMPL autorizado (`PERF-V0-20260913`). Executor Ingeniero (stack completo); Parser/Codegen HOLD salvo review. Sin theater de tiempos. Nombre = **ARITA**.
- **Barra:** parse → emit → rustc → execute + `clippy -D warnings`; skip ≠ PASS; inconclusive ≠ accepted.

## Contexto

Fase 4b (resto tras Async): tuning **sin** hackear bytes ni `unsafe`. Camino permitido = perfiles de emit, flags reales en el crate generado, oráculos E2E que prueben el perfil — no microbench theater.

Async v0 (ADR-027) ya **verified** Lex 63/63.

## Decisión (MVP)

### 1. Perfiles IN

| Perfil | Default | Emit Cargo (crate generado) |
|--------|---------|------------------------------|
| `debug` | **sí** | perfil Cargo `dev` / sin `opt-level` forzado (baseline actual) |
| `release` | no | `[profile.release] opt-level = 3`; **LTO off** (no `lto = true` / fat) |

Futuro `perf` (LTO / codegen-units) = **OUT** de v0 — solo mención ROADMAP.

### 2. Surface / CLI (pin cerrado)

- Flag CLI canónica: `arita build --profile debug|release`.
- Default sin flag = **`debug`**.
- `profile` en `arita.toml` = **OUT de v0** (futuro).
- **Sin** keywords/`unsafe`/intrinsics en `.arita` para “ir más rápido”.

### 3. Oráculos measure (E2E reales)

| Id | Qué prueba | Expect |
|----|------------|--------|
| `perf-01-release-run` | Programa pequeño build+run con `--profile release` | stdout fijado en GO IMPL (anti-theater) |
| `perf-02-release-optlevel` | Evidencia en **Cargo.toml generado**: `[profile.release] opt-level = 3` | parse/check del manifiesto emitido (no sidecar meta; no smoke) |
| `perf-neg-e0250` (opcional) | `--profile fantasma` / desconocido | **E0250** `unknown build profile` |

Corpus: **`ejemplos/perf/`** dedicada (no reutilizar paths `ejemplos/f2/`).

Umbrales de **tiempo / wall-clock**: **OUT de v0**. Si en el futuro se miden: host sin reloj fiable → `inconclusive`, nunca `accepted` (ROADMAP). v0 no inventa PASS por “fue rápido”.

### 4. Diagnósticos

| Código | Mensaje EN canónico (draft) | Cuándo |
|--------|----------------------------|--------|
| **E0250** | `unknown build profile` | perfil ≠ `debug`\|`release` |

No colisiona E024x (async) ni E023x (unsafe/contract).

### 5. OUT

- `unsafe` / FFI / `asm` / SIMD / byte-hacking en surface o emit de usuario
- Perfil `perf` fantasma sin flags reales en Cargo
- LTO fat / `codegen-units = 1` como default v0
- Mutar política Clippy (`-D warnings`) para “hacer pasar” perf
- Oráculos de latencia con umbrales inventados

### 6. Safe-only

ADR-022 / E0231: emit usuario `#![forbid(unsafe_code)]`. Perf no es escape hatch.

### 7. Forma (CLI)

```bash
arita build ejemplos/f2/01-let-int.arita
arita build --profile release ejemplos/perf/01-release-hello.arita
```

Paths bajo `ejemplos/perf/`; crear en IMPL con stdout fijado.

## Decisiones cerradas (2026-09-13 — Ingeniero GO)

1. **CLI-only** v0: `arita build --profile debug|release` (default debug). `profile` en `arita.toml` = OUT v0 / futuro.
2. Evidencia opt-level: **Cargo.toml generado** (`[profile.release] opt-level = 3`); **no** sidecar meta.
3. Oráculos en **`ejemplos/perf/`** dedicada (no reusar paths f2).

## Consecuencias

- **Verify 2026-09-13:** `arita measure` Lex **accepted 66/66** (PERF-V0 landed). Parser/Codegen: HOLD → **review-only**.
- CUT IMPL autorizado: executor Ingeniero (stack completo, patrón ASYNC-V0).
- Parser/Codegen: HOLD crates salvo review-only.
- ROADMAP Fase 4b Perf: este ADR **aceptada**.
- PACK: no few-shot perf hasta oráculos E2E verdes.

## Checklist GO

- [x] Perfiles `debug` / `release` + `opt-level = 3` OK
- [x] LTO off v0 OK
- [x] Umbrales timed OUT v0 OK
- [x] E0250 OK
- [x] Abiertas 1–3 **cerradas**
- [x] GO DOC → **aceptada** + IMPL autorizado
- [x] Oráculos `ejemplos/perf/` E2E + measure wire
