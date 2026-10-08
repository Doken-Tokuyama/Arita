[Español](PERF-HOST-BORDER.md) | English

# Perf host border — what tuning stays outside the ARITA surface

- **Estado:** DOC v0 (ROADMAP Fase 4b; sin CUT IMPL)
- **Fecha:** 2026-09-14
- **Relacionados:** ADR-028 (`DOC/ADR/028-perf-v0.md`), ADR-022 / E0231 (safe-only), ADR-001 (emit-Rust), `DOC/THREAT_MODEL.md` (bench de teatro), `DOC/TRAPS-CATALOG.md` (§E / §I), ROADMAP Fase 4b Perf
- **Gobernanza:** documentación de frontera. **No** cambia crates/`measure`. Extender el surface o nuevos perfiles = **CUT** explícito.
- **Barra:** skip ≠ PASS; inconclusive ≠ accepted; sin theater de tiempos.

## Principle

ARITA exposes **profiles and emit evidence** (ADR-028). The rest of performance tuning lives on the **Rust/host border**: toolchain, `cargo`/`rustc` flags, profiling tools, and code outside the `.arita` dialect. Safe-only (ADR-022): perf is **not** an escape hatch to `unsafe` / `asm` / SIMD in surface or user emit.

## IN — ARITA surface (measured v0)

| What | Where / how | Evidence |
|-----|--------------|-----------|
| CLI `--profile debug\|release` | `arita build --profile …` (default `debug`) | E0250 if unknown profile (`perf-neg-e0250`) |
| Profile emit flags | `[profile.release] opt-level = 3` in **generated Cargo.toml**; **LTO off** v0 | `perf-02-release-optlevel` |
| Real release run | build+run with `--profile release` | `perf-01-release-run` (fixed stdout) |
| Safe std / hot-path APIs | std whitelist (ADR-006 / ADR-026); no escape to unsafe | F2 / std oracles; E0206 |
| Measure oracles | `ejemplos/perf/` + runners in measure | Lex **73/73** (do not mutate crates/measure in this DOC) |

**Explicit IN v0:** `debug`/`release` profiles, `opt-level = 3` in emitted release, prior E2E oracles.  
**Not** IN v0: timed / wall-clock thresholds; `profile` in `arita.toml`; phantom surface `perf` profile.

## OUT / Rust–host border (outside the ARITA surface)

These levers may be used on the **host** (manual Rust crate, `RUSTFLAGS`, CI, tooling) or wait for a future CUT. They are **not** part of the `.arita` dialect or the measure v0 contract.

| Category | Examples | Why outside |
|-----------|----------|---------------|
| LTO / link-time | `lto = "fat"`, `lto = true`, `codegen-units = 1` | ADR-028: LTO off in v0; future `perf` profile only with CUT |
| CPU-specific flags | `target-cpu=native`, `-C target-feature=+avx2`, march/mtune | Not portable; break measure / cross reproducibility |
| Profiling tools | `perf`, Instruments, samply, flamegraph, `cargo flamegraph`, Tracy | Host observability; not ARITA oracles; no PASS for “felt fast” |
| Unsafe micro-opts | `unsafe`, intrinsics, inline `asm`, manual SIMD in `.arita` or user emit | ADR-022 / E0231; `#![forbid(unsafe_code)]` |
| Byte-hacking / transmute theater | reinterpret buffers, “byte hacks” for latency | ROADMAP Fase 4b OUT; contract bypass |
| Theater bench | trivial microbench, warm cache, invented thresholds, PASS via noisy wall-clock | `THREAT_MODEL` row **Theater bench**; timed thresholds **OUT** PERF-V0 |
| Mutate evidence policy | weaken Clippy `-D warnings`, skip oracles, “meta” sidecar instead of real Cargo.toml | anti-theater; ADR-028 OUT |

### Border pattern (note, not IMPL of this DOC)

Hot path needing free deps / bounded unsafe: **Rust host crate** behind a safe border; ARITA only calls the bridge (ROADMAP Fase 4c — border-pattern item, distinct from this DOC). See also ADR-029 §8.

## Theater risks (threat-model link)

Any “perf evidence” that is not a declared oracle on the exact artifact falls into modes already cataloged:

- **Theater bench** — `DOC/THREAT_MODEL.md` (fake-mode table).
- **Timed thresholds** — OUT v0; if a future CUT adds them: host without a reliable clock → `inconclusive`, **never** `accepted`.
- **Fake release** — without real `opt-level = 3` in emit → caught by `perf-02-release-optlevel` (`TRAPS-CATALOG` §I).

Do not invent PASS. `skip ≠ PASS`.

## How to extend later (CUT required)

Without an explicit CUT + GO: **do not** widen surface or measure.

| Extension | Requires |
|-----------|----------|
| Surface `perf` profile (LTO / codegen-units) | ADR CUT (successor to ADR-028); **real** flags in generated Cargo.toml; new E2E oracles; LTO not “documented” without evidence |
| `profile` in `arita.toml` | CLI/config CUT; default and precedence vs `--profile` |
| Timed / latency oracles | measure CUT; inconclusive policy if host clock unreliable; declared dataset/warmup (`THREAT_MODEL`) |
| More emit flags (`panic=abort`, strip, etc.) | emit CUT + manifest/binary oracle |
| New hot-path std APIs | std CUT (ADR-026 pattern) + oracles; no unsafe |
| Host bridge with free deps | ROADMAP 4c border pattern + deps/bridge CUT (ADR-029+) |

**Forbidden without CUT:** touch crates/`measure` “to teach” LTO; promote inconclusive→accepted; add SIMD/`asm` to the dialect.

## One-line summary

**ARITA measures profiles and safe emit; fat LTO, CPU flags, profilers, unsafe and theater benches live on the host border — and only cross with a CUT.**
