# Perf host border — qué tuning queda fuera de la surface ARITA

- **Estado:** DOC v0 (ROADMAP Fase 4b; sin CUT IMPL)
- **Fecha:** 2026-09-14
- **Relacionados:** ADR-028 (`DOC/ADR/028-perf-v0.md`), ADR-022 / E0231 (safe-only), ADR-001 (emit-Rust), `DOC/THREAT_MODEL.md` (bench de teatro), `DOC/TRAPS-CATALOG.md` (§E / §I), ROADMAP Fase 4b Perf
- **Gobernanza:** documentación de frontera. **No** cambia crates/`measure`. Extender el surface o nuevos perfiles = **CUT** explícito.
- **Barra:** skip ≠ PASS; inconclusive ≠ accepted; sin theater de tiempos.

## Principio

ARITA expone **perfiles y evidencia de emit** (ADR-028). El resto del tuning de rendimiento vive en el **borde Rust/host**: toolchain, flags de `cargo`/`rustc`, herramientas de perfilado y código fuera del dialecto `.arita`. Safe-only (ADR-022): perf **no** es escape hatch a `unsafe` / `asm` / SIMD en surface ni en emit de usuario.

## IN — surface ARITA (v0 medido)

| Qué | Dónde / cómo | Evidencia |
|-----|--------------|-----------|
| CLI `--profile debug\|release` | `arita build --profile …` (default `debug`) | E0250 si perfil desconocido (`perf-neg-e0250`) |
| Emit flags de perfil | `[profile.release] opt-level = 3` en **Cargo.toml generado**; **LTO off** v0 | `perf-02-release-optlevel` |
| Run release real | build+run con `--profile release` | `perf-01-release-run` (stdout fijado) |
| APIs std / hot paths seguros | whitelist std (ADR-006 / ADR-026); sin escape a unsafe | oráculos F2 / std; E0206 |
| Oráculos measure | `ejemplos/perf/` + runners en measure | Lex **73/73** (no mutar crates/measure en este DOC) |

**IN explícito v0:** perfiles `debug`/`release`, `opt-level = 3` en release emitido, oráculos E2E anteriores.  
**No** IN v0: umbrales timed / wall-clock; `profile` en `arita.toml`; perfil surface `perf` fantasma.

## OUT / borde Rust–host (fuera de la surface ARITA)

Estas palancas pueden usarse en el **host** (crate Rust manual, `RUSTFLAGS`, CI, tooling) o quedar para un futuro CUT. **No** forman parte del dialecto `.arita` ni del contrato measure v0.

| Categoría | Ejemplos | Por qué fuera |
|-----------|----------|---------------|
| LTO / link-time | `lto = "fat"`, `lto = true`, `codegen-units = 1` | ADR-028: LTO off en v0; futuro perfil `perf` solo con CUT |
| Flags CPU-específicas | `target-cpu=native`, `-C target-feature=+avx2`, march/mtune | No portables; rompen reproducibilidad measure / cross |
| Profiling tools | `perf`, Instruments, samply, flamegraph, `cargo flamegraph`, Tracy | Observabilidad host; no oráculos ARITA; no PASS por “pareció rápido” |
| Unsafe micro-opts | `unsafe`, intrinsics, inline `asm`, SIMD manual en `.arita` o emit usuario | ADR-022 / E0231; `#![forbid(unsafe_code)]` |
| Byte-hacking / transmute theater | reinterpretar buffers, “hackear bytes” para latencia | ROADMAP Fase 4b OUT; bypass de contrato |
| Bench de teatro | microbench trivial, caché precargada, umbrales inventados, PASS por wall-clock ruidoso | `THREAT_MODEL` fila **Bench de teatro**; timed thresholds **OUT** PERF-V0 |
| Mutar política de evidencia | bajar Clippy `-D warnings`, skip oráculos, sidecar “meta” en vez de Cargo.toml real | anti-theater; ADR-028 OUT |

### Patrón borde (nota, no IMPL de este DOC)

Hot path que necesite deps libres / unsafe acotado: **crate host Rust** detrás de frontera safe; ARITA solo llama el bridge (ROADMAP Fase 4c — ítem patrón borde, distinto de este DOC). Ver también ADR-029 §8.

## Riesgos de theater (enlace threat model)

Cualquier “evidencia de perf” que no sea oráculo declarado sobre el artefacto exacto cae en modos ya catalogados:

- **Bench de teatro** — `DOC/THREAT_MODEL.md` (tabla modos fake).
- **Timed thresholds** — OUT v0; si un CUT futuro los añade: host sin reloj fiable → `inconclusive`, **nunca** `accepted`.
- **Fake release** — sin `opt-level = 3` real en emit → atrapado por `perf-02-release-optlevel` (`TRAPS-CATALOG` §I).

No inventar PASS. `skip ≠ PASS`.

## Cómo extender más adelante (CUT obligatorio)

Sin CUT + GO explícito: **no** ampliar surface ni measure.

| Extensión | Requiere |
|-----------|----------|
| Perfil surface `perf` (LTO / codegen-units) | CUT ADR (sucesor ADR-028); flags **reales** en Cargo.toml generado; oráculos E2E nuevos; LTO no “documentado” sin evidencia |
| `profile` en `arita.toml` | CUT CLI/config; default y precedencia vs `--profile` |
| Oráculos timed / latency | CUT measure; política inconclusive si reloj host no fiable; dataset/warmup declarados (`THREAT_MODEL`) |
| Más flags emit (`panic=abort`, strip, etc.) | CUT emit + oráculo de manifiesto/binario |
| APIs std hot-path nuevas | CUT std (patrón ADR-026) + oráculos; sin unsafe |
| Bridge host con deps libres | ROADMAP 4c patrón borde + CUT deps/bridge (ADR-029+) |

**Prohibido sin CUT:** tocar crates/`measure` “para enseñar” LTO; promover inconclusive→accepted; añadir SIMD/`asm` al dialecto.

## Resumen una línea

**ARITA mide perfiles y emit seguro; LTO gordo, CPU flags, profilers, unsafe y benches de teatro viven en el borde host — y solo cruzan la frontera con CUT.**
