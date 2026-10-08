English | [Español](ROADMAP.es.md)

# ARITA — Roadmap

ARITA is a general-purpose language designed to be written by AIs that compiles to safe Rust (no `unsafe`). This roadmap summarizes what is done and where the project is heading. It is a product document: the design detail is in [`DOC/`](DOC/README.md).

## Current status

- **ARITA v1: published on 2026-10-08.**
- **Core 0.9: CLOSED — 834/834 oracles passed.**
- **Core 0.10 (ERRORS, phase 1):** eleven stages of phase 1 are closed: elimination of error results discarded in dead paths, safety of task operations (`spawn`/`join`), a check that `arita build` only compiles files that belong to the package, cleanup of Clippy warnings in the emitted Rust, removal of unnecessary parentheses in the emitted Rust, index-based compound assignment on integer vectors (`v[i] += x`, `-=`, `*=`; an out-of-range index or an overflow returns an error instead of causing a panic), integer arithmetic with overflow detected at run time (`+`, `-`, `*` on non-literal integers end with a controlled failure, in both debug and release builds, instead of silently wrapping the value), diagnosis of calls to undeclared functions (a call to a function that is not part of the language, not declared in the module and not imported with `use` is rejected with its own error, E0347, before generating Rust), scope of known integers (integer values the compiler knows in advance are no longer carried across branches, loops and blocks, so valid programs no longer get false overflow or division-by-zero errors), rejection, with its own diagnostic (E0346), of Mutex concurrency in this version (the real Mutex will arrive in 1.1) and discarded results (an ignored `Result`, whether a bare call, `let _` or an unused variable, is a compile error, E0272). With this, all stages planned for version 1 are closed.
<!-- BARRA-CORE-0.10 --> **Core 0.10 — current bar 889/889 (all stages planned for version 1 are closed).**

An oracle is a reproducible test (build, run and compare the expected result, plus `clippy`). A skipped test never counts as passed.

## Next: version 1.1

- Close the known limitations of version 1: first B-286-6b, B-296-2, B-297-1 and B-297-3; then B-286-3, B-286-4, B-286-6 and B-286-6a.
  - B-286-3, B-286-4, B-286-6, B-286-6a, B-286-6b and B-297-1 are described in the README's [known limitations of v1](README.md#known-limitations-of-v1).
  - B-296-2: `Io` fns other than `main` cannot take parameters yet (the parser rejects them with E0006).
  - B-297-3: `true`, `false`, `None` or `"hola"` in a `let` typed `Int` pass ARITA's check and fail in rustc with E0308.
- A real Mutex (proposed).
- Extension of the async surface (proposed).

## Core version ladder ("Core")

Each core version adds a vertical capability and is only considered closed when all of its oracles pass.

| Version | Capability | Status |
|---------|-----------|--------|
| Core 0.1 | Base vertical: records, enums, `match`, basic types, `Result`, simple ownership, files/JSON/CLI, executable scenarios | Closed (603/603) |
| Core 0.2 | `service` profile: tasks, timers and cancellation, HTTP bindings, reference service | Closed (632/632) |
| Core 0.3 | Client + server composition, error propagation, sequential pipelines | Closed (658/658) |
| Core 0.4 | Packages and libraries (manifest, library API, multiple modules) | Closed (683/683) |
| Core 0.5 | Collections with fallible operations (`insert` → `Result`, index access as `Option`) | Closed (705/705) |
| Core 0.6 | End-to-end general-purpose programs (sets, maps, scenarios) | Closed (728/728) |
| Core 0.7 | Input/output without theater (reading and *parsing* with exact errors, CLI arguments) | Closed (752/752) |
| Core 0.8 | Error propagation: functions that return `Result` and the `?` operator | Closed (778/778) |
| Core 0.9 | Index-based writes to collections (`m[k] = v`, `v[i] = x`) without panics | Closed (834/834) |
| Core 0.10 | Errors, phase 1 (dead sinks, `spawn`/`join` safety, package membership, emitted Rust without Clippy warnings or unnecessary parentheses, index-based compound assignment, integer arithmetic with overflow detection, calls to undeclared functions, scope of known integers, rejection of Mutex in version 1 and discarded results) | Eleven stages closed (all those planned for version 1) |

## Language phases

| Phase | Contents | Status |
|------|-----------|--------|
| 0 | Base documentation and design decisions | Done |
| 1 | Toolchain skeleton: parsing, Rust generation, CLI, `arita measure` | Done |
| 2 | Ownership subset, minimal standard library and method *std* | Done |
| 3 | Logic island (`spec` / `fact` / `rule` / `query`) with an in-house engine; 12 oracles | Done |
| 4 | Cross-platform (Linux, macOS, Windows; x86_64 and ARM), `async`, performance and bounded Rust dependencies | Done within its current scope |
| 5 | Self-hosting (optional) | First steps (bootstrap examples) |

## Principles that do not change

1. **Safe by construction:** the emitted Rust does not use `unsafe`.
2. **No theater:** nothing is accepted for looking correct; only if reproducible oracles prove it on the exact artifact.
3. **Few ways to write the same thing:** canonical syntax and structured errors with stable codes (`E0xxx`).
4. **Independent of the AI:** the emitted Rust project works without the AI that wrote the `.arita`.

## Later (no date)

- Extend the core with new vertical capabilities, always with oracles before declaring them closed.
- Compiler-guided repair (the compiler as a structured error oracle): today it is a design proposal, see [`DOC/REPAIR-ORACLE.md`](DOC/REPAIR-ORACLE.md).
- Integration with editors and development tools.

## Out of current scope

System threads, open crates.io dependencies, TLS/WebSocket, idle shutdown of services and additional I/O *bindings* are not part of the language for now. System threads remain out of current scope; mutual exclusion between tasks (a real Mutex) is proposed for v1.1.

## How to verify it

From the repository root: `cargo test --workspace -- --test-threads=1` and `cargo run -p arita-cli -- measure`. More detail in [`BUILD.md`](BUILD.md) and [`DOC/CI.md`](DOC/CI.md).
