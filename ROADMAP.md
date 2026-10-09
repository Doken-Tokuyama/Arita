English | [Español](ROADMAP.es.md)

# ARITA — Roadmap

ARITA is a general-purpose language designed to be written by AIs that compiles to safe Rust (no `unsafe`). This roadmap summarizes what is done and where the project is heading. It is a product document: the design detail is in [`DOC/`](DOC/README.en.md).

## Current status

- **ARITA v1: published on 2026-10-08.**
- **Core 0.9: CLOSED — 834/834 oracles passed.**
- **Core 0.10 (ERRORS, phase 1):** eleven stages of phase 1 are closed: elimination of error results discarded in dead paths, safety of task operations (`spawn`/`join`), a check that `arita build` only compiles files that belong to the package, cleanup of Clippy warnings in the emitted Rust, removal of unnecessary parentheses in the emitted Rust, index-based compound assignment on integer vectors (`v[i] += x`, `-=`, `*=`; an out-of-range index or an overflow returns an error instead of causing a panic), integer arithmetic with overflow detected at run time (`+`, `-`, `*` on non-literal integers end with a controlled failure, in both debug and release builds, instead of silently wrapping the value), diagnosis of calls to undeclared functions (a call to a function that is not part of the language, not declared in the module and not imported with `use` is rejected with its own error, E0347, before generating Rust), scope of known integers (integer values the compiler knows in advance are no longer carried across branches, loops and blocks, so valid programs no longer get false overflow or division-by-zero errors), rejection, with its own diagnostic (E0346), of Mutex concurrency in this version (the real Mutex is planned for version 0.1.2) and discarded results (an ignored `Result`, whether a bare call, `let _` or an unused variable, is a compile error, E0272). With this, all stages planned for version 1 are closed.
<!-- BARRA-CORE-0.10 --> **Core 0.10 — current bar 889/889 (all stages planned for version 1 are closed).**

An oracle is a reproducible test (build, run and compare the expected result, plus `clippy`). A skipped test never counts as passed.

## Next: version 0.1.2

Version 0.1.2 is the next publishable release; there is no separate version 1.1. Plan set by the project author on 2026-10-09. Work order:

1. **Phase 1 (applied, being verified):** B-286-6b, B-296-2, B-297-1, B-297-3 and B-286-6. B-286-6 is included because of the bare `await`: an `await h()` whose `Result` is discarded is rejected with E0272. These limitations are closed if oracle 7 of this phase (the bare `await` case) passes, and that is recorded when the design is frozen. The phase covers the real type of `await` and its use as a value, `Int` and `Bool` parameters in `Io` fns other than `main`, Rust keywords in expression position (E0007) and unbound names (E0354), and the type check of a literal or constructor initializer in a `let` with a scalar type.
2. **Phase 2:** B-286-3, B-286-4 and B-286-6a, plus the `Err` of `if let` and `while let` and an `if let Ok` that ignores the `Err` (without `else`, or with an `else` that ignores it). The extended design for this phase will cite B-286-6 as closed by phase 1 and will only add what is missing (for example, the warning of B-286-6a).
3. **Phase 3:** all the other known limitations and open bugs of version 1, including B-297-5 to B-297-8 and B-286-8. Anything that turns out to be impossible will be decided case by case.
4. **A real Mutex:** from proposal to an implementable design.
5. **Extended async surface:** from proposal to an implementable design.
6. **Compiler-guided repair:** structured diagnostics, deterministic suggestions and `arita fix`. `arita check --json` emits diagnostics in a versioned format (`arita.diagnostics.v1`); deterministic suggestions per error code, starting with E0004; and a repair loop with `arita fix`, which is deterministic and idempotent and only writes changes with an explicit `--write`. Non-deterministic or high-risk repair steps move to version 0.1.3. Design document: pending.

About the bugs above:

- B-286-3, B-286-4, B-286-6, B-286-6a, B-286-6b and B-297-1 are described in the README's [known limitations of v1](README.md#known-limitations-of-v1).
- B-296-2: `Io` fns other than `main` cannot take parameters yet (the parser rejects them with E0006).
- B-297-3: `true`, `false`, `None` or `"hola"` in a `let` typed `Int` pass ARITA's check and fail in rustc with E0308.

Under phase 1:

- B-297-2: the E0342 queue must compare the expected and found types; closed with the async surface work.

**Phase 3 also includes** (known limitations and open bugs beyond the ids already named above):

- Stricter `Err(_)` / `Err(_eN)` patterns without `let` (B-286-1).
- Diagnostic for a pure expression whose value is unused (B-286-9).
- Widen the fixed emit-ban list (B-295-1).
- Every strict or reserved Rust keyword used as a binding or parameter is rejected with E0007, in every edition (B-286-8, widened).
- Unbound names typed as `Int` in more positions (assignment target, receiver, index assign) (B-293-1); imported functions mistyped as `Int` (B-293-2).
- Closed-list exemption for `::` paths, under a dedicated associated-methods design (B-293-3).
- Richer spans on calls (B-286-11) and on paths (B-297-5).
- `MIN / -1` must report E0217, not E0100 (B-292-1).
- Inner `let` must not silently inherit type, mutability or moved-state from an outer binding (B-294-1); flow-sensitive E0217 (B-294-2).
- Optional checked arithmetic with `?` (B-292-4).
- Compound assignment on a bare integer identifier (`x += 1`) (B-290-1).
- Package resolution: canonicalize `arita.toml` lookup (B-287-1); honor `--target` on the workspace path (B-287-2); nested autonomous manifests — nearest wins, with an explicit diagnostic (B-287-3).
- General typing of `let` initializers beyond the scalar literal/constructor slice (B-297-3b).
- Semantics of a function name used as a value outside `serve` (B-297-7).
- Arity and type checks on calls to user functions (B-297-8); defense so `await` as a statement on a `Result`-returning fn does not slip into codegen as E0006 (B-297-6).
- Task panic swallowed by `join` (B-296-1).
- Contract / `?` path fixture that currently rejects a valid program (B-286-2).
- Further rustc/Clippy lints on positive oracles (B-289-1); oracles for supplemental workspaces (B-289-2).
- Negative-runner harness classes B and C (B-297-4).
- Widen E0215 for tautological comparisons such as `x == x` (user-visible theater).
- Package: a public fn must not collapse to a private helper; missing fn still E0347.
- `arita parse` on a valid workspace binary must exit 0.
- Documented edges that pin stdout for both `Err(0)` paths; optional build warning when scenarios are declared but not run.

Under the Mutex and async points:

- Mutex design covers RAII guards, `thread_local` policy, `Mutex<T>` parameters and `spawn` arguments (today rejected with E0346 where applicable).
- Includes `pub async fn` in libraries and `async main` in package binaries.

Under compiler-guided repair:

- Also in 0.1.2: the minimum write-protection for `arita fix --write` (content hash, in-memory re-check, race guard).


Under AI agent integrations (go-ahead from the project author, 2026-10-09 19:30):

- Agent instruction pack: a canonical `AGENTS.md`, a short `CLAUDE.md` that points to it, few-shot `.arita` examples with `arita check --json` output, and the E0xxx code table (cause and typical fix). Aimed at tools that already load `AGENTS.md` (among others Cursor, OpenCode, Devin Desktop/CLI, GitHub Copilot, OpenAI Codex, Zed, Verdent, Antigravity, Freebuff/Codebuff and DeepSeek Harness); Claude Code uses `CLAUDE.md`.
- Post-edit hook templates that run `arita check --json` on `.arita` edits (adapters for the hook formats of those tools).
- `arita-mcp` over stdio with the tools `check`, `build`, `fix`, `explain` and `contract`.

**Release criteria:** in every stage, all oracles must pass with no skipped tests, and the evidence must pass an independent verification. Version 0.1.2 is released only when every stage is closed.

## Version 0.1.3 (extensions)

### Language

- **proposed:** Tighten the unwrap emit-ban on host paths if measure shows a gap.

### Ownership and concurrency

- **proposed:** Documented guidance on clone cost / simple ownership (until a stable oracle exists).

### Tooling

- **proposed:** `arita build --diagnostics json` (rustc diagnostics in JSON).
- **proposed:** CI must not cache `target/` when Miri stores environment data there.
- **proposed:** TextMate grammar and a VS Code extension published on Open VSX and the VS Code Marketplace.

### Repair and distribution

- **proposed:** Repair step 2 (rest): revision store and sandbox.
- **proposed:** Repair controller with budget and stagnation detection.
- **proposed:** Hand-written library of about 20–30 repair templates.
- **proposed:** Repair memory ranked by measured evidence.
- **proposed:** LLM only for non-mechanical semantic decisions in the repair loop.
- **proposed:** Promote golden repairs into rules, tests and deterministic fixes.
- **proposed:** Model finetuning only after thousands of golden pairs (version 0.1.3 or later).
- **proposed:** Swallow mutants in repair never count as accepted (human review).
- **proposed:** Reject 100% of hunks that touch protected paths.
- **proposed:** Per-code false-positive log with a fixed seed.
- **design in progress:** Broader repair-oracle design (controller and memory steps beyond the 0.1.2 slice).

## Version 0.2

### Language

- **proposed:** `for` loops over collections (needs new syntax; today `for` is rejected as an illegal statement with E0006).
- **existing partial support, extension:** Logic island (inspired by Tau/TML, in-house engine).
- **proposed:** WASM backend.
- **proposed:** v0.1 language semantics document (normative product prose).
- **proposed:** Optional contracts / formal verification proportional to risk.
- **proposed:** IR / AIR / CFG layer in the compilation pipeline.

### Standard library and I/O

- **proposed:** Standard-library APIs aimed at measurable hot paths (still without `unsafe`).

### Tooling

- **proposed:** UI for the evidence manifest.
- **proposed:** Language server over `arita check --json` (diagnostics and fix-oriented code actions).
- **proposed:** Zed extension with a tree-sitter grammar for `.arita`.

## Not scheduled (on hold)

### Language

- **on hold:** Indexed write into `String` (`s[i] = x`, today E0314).
- **on hold:** Residual index-assign cases (E0006 / E0314) and splitting E0314 (index-assign vs unknown field).
- **on hold:** `?` on `Option` and `?` in `main` with `Io` (later phase).
- **on hold:** `unwrap` / `expect` on the language surface (emit-ban).
- **on hold:** AI-native verified software model (RFC-level), kept on hold until an explicit go-ahead from the project author.

### Ownership and concurrency

- **on hold:** Deeper ownership and aliasing (receiver loans, `&mut` collection parameters, related vertical).
- **on hold:** Multi-thread runtime (system threads alongside tasks).
- **on hold:** Reads and richer operations inside a lock (index get, `get`/`first`/`last`/`pop`, `Result`/`Option` methods, indexed assign on `Vec`/`List`).
- **on hold:** Sharing a Mutex via return values, records, collections or `Result`/`Option` (today E0346); explicit `Arc`.
- **on hold:** `?` form of the lock block; wider lock-block whitelist (`match` / `if let` / `while let` / pure helpers).
- **on hold:** RwLock / Condvar; async Mutex; `try_lock` / timed lock; declared lock-order ranges; surface atomics; guard-as-value; nested locks; split critical-section lint; `Mutex<Bytes>` / `Mutex<record>`.

### Async

- **on hold:** Non-`Copy` / `Text` parameters on non-`main` `Io` fns and on `async fn`.

### Standard library and I/O

- **on hold:** New I/O bindings.

### Network

- **on hold:** TLS and WebSocket.
- **on hold:** Raw sockets.
- **on hold:** Idle shutdown of services (enforce idle).

### Tooling

- **on hold:** Open crates.io dependencies and publishing (except `cargo publish --dry-run`).
- **on hold:** ACP connector for editor and agent tooling.

### Repair and distribution

- **on hold:** Packaging as distribution (prebuilt binaries, installers).

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
- Integration with editors and development tools.

## Out of current scope

System threads, open crates.io dependencies, TLS/WebSocket, idle shutdown of services and additional I/O *bindings* are not part of the language for now. System threads remain out of current scope; mutual exclusion between tasks (a real Mutex) is planned for version 0.1.2 (from proposal to an implementable design).

## How to verify it

From the repository root: `cargo test --workspace -- --test-threads=1` and `cargo run -p arita-cli -- measure`. More detail in [`BUILD.md`](BUILD.en.md) and [`DOC/CI.md`](DOC/CI.md).
