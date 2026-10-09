English | [Español](README.es.md)

# ARITA

**ARITA** is a general-purpose programming language **designed to be written by AIs** that **compiles to safe Rust**. The AI writes `.arita` files; the compiler translates them, deterministically, into real Rust projects (*safe* code, no `unsafe`) that build with `cargo` and run without depending on the AI that wrote them.

## What it is (and what it is not)

- **It is** a new language with a small, total and canonical semantics: few equivalent ways to write the same thing, local semantics, structured and repairable errors, and no "it compiles but surprises you".
- **It is** a compiler that emits real Rust: `.arita` → parsing → typed intermediate representation → deterministic emission → Rust → `cargo`.
- **It is not** a Rust dialect or a `cargo` wrapper where "the AI edits Rust".
- **It is not** a tooling platform for AIs or a formal-verification contract language.

## Design principle: no theater

A piece of code is only considered accepted when **declared, reproducible oracles** prove it on the exact artifact: build, run and compare the expected output, plus pass `clippy`. Building, having green tests or producing a demo is not enough. A skipped test never counts as passed.

## Status

- **ARITA v1 published on 2026-10-08.**
- **Core 0.9 CLOSED 834/834.** Index-based writes to collections (`m[k] = v`, `v[i] = x`) without panics.
- **Core 0.10** (ERRORS, phase 1). <!-- BARRA-CORE-0.10 -->Current bar: 889/889 (all stages planned for version 1 are closed). See [`ROADMAP.md`](ROADMAP.md).
- Prebuilt binaries will come with the v0.1.1 release.

Earlier core versions (0.1 to 0.8) are closed; the full ladder is in the [roadmap](ROADMAP.md).

## Known limitations of v1

Known diagnostic gaps in this version:

- **B-286-3** (v0.1.2, phase 2): in the `Err(e)` arm of a `match` on a host read (`host.read_text`), passing the error payload to a variable that is never used (`let _c: Int = e`) produces no diagnostic and the program is accepted. The same form on a user function that returns `Result` does give E0272.
- **B-286-4** (v0.1.2, phase 2): that same dead binding of the `Err` payload inside `if let Err(e) = …` or `while let Err(e) = …` is not diagnosed either.
- **B-286-6** (v0.1.2, phase 1): a bare `await h()` (in the generated Rust, `h().await;`), when `h` returns `Result`, discards the result without a diagnostic.
- **B-286-6a** (v0.1.2, phase 2): `arita build` does not show rustc warnings about the generated Rust; for example, the `unused_must_use` of the previous case.
- **B-286-6b** (v0.1.2, phase 1): the `Result` of a user `async fn` cannot be consumed yet: `let r = await h()` gives E0203 and `match await h()` is not parsed.
- **B-297-1** (v0.1.2, phase 1): a keyword used as a value (e.g. `let x: Int = return`) is not diagnosed: the program compiles and exits early. In other cases of unbound identifiers the error comes from rustc instead of ARITA.

In B-286-3, B-286-4 and B-286-6 the program is accepted and the generated Rust is safe and does what the code says, but the error is lost without warning.

### Guidelines for writing ARITA v1 (AI or human)

- In v1, an `async fn` should not return `Result`: its error cannot be consumed and is silently lost. Handle the error inside the fn.
- v1 does not diagnose discarding the `Err` in `if let` / `while let` (nor in an `if let Ok(..)` whose `else` ignores the Err) or in a `match` on `host.read_text`. In a `match` on user fns, E0272 does fire.

## How to build and test

Requirements: stable Rust toolchain (`cargo`). The full acceptance verdict also needs `clippy`.

From the repository root:

```bash
cargo test --workspace -- --test-threads=1
cargo build -p arita-cli
./target/debug/arita build ejemplos/01-hello.arita
```

`arita build` prints a line `ok: target/arita-out/hello_<hash>` (the name carries a hash suffix). Run the path that `ok:` prints, for example `./target/arita-out/hello_*`, and you will see `hello`.

Minimal example (`ejemplos/01-hello.arita`):

```arita
module hello

fn main() -> Io<()> {
  print("hello")
}
```

The logic island (modules with `fact` / `rule` / `query`, evaluated by an in-house Datalog engine, not by `rustc`) runs like this:

```bash
./target/debug/arita logic ejemplos/f3/01-path-ok.arita   # prints: true (exit code 0)
```

All CLI subcommands and the local continuous-integration gate (`bash scripts/ci.sh`) are described in [`BUILD.md`](BUILD.en.md). The verification guide for AIs is in [`DOC/09-AI-PROGRAMMING.md`](DOC/09-AI-PROGRAMMING.en.md).

## Example oracles

The programs in `ejemplos/` are end-to-end oracles (build, run and compare the output). For example, the F3 logic island has **12 oracles** (`f3-01` … `f3-12`) in [`ejemplos/f3/`](ejemplos/f3/): cases that must hold and negative cases with exact error codes (`E0301`, `E0303`, `E0304`). Full index in [`ejemplos/README.md`](ejemplos/README.md).

## Repository layout

| Path | Contents |
|------|-----------|
| `crates/` | Rust workspace: syntax, typed intermediate representation (HIR), code generation, logic-island engine, CLI (`arita-cli`) and support crates for host examples |
| `ejemplos/` | `.arita` programs that serve as oracles, organized by phase and by core version (`f2`, `f2.2`, `f3`, `async`, `core01` … `core10`, etc.) |
| `DOC/` | Documentation: vision, architecture, semantics, threat model, guides, example packs for AIs and design decisions (`DOC/ADR/`) |
| `scripts/` | Local continuous-integration gate |
| `ROADMAP.md` | Product roadmap |
| `BUILD.md` | Build instructions and CLI usage |

## Recommended reading

- Short vision: [`DOC/00-VISION.md`](DOC/00-VISION.en.md) and [`DOC/PRODUCT-VISION.md`](DOC/PRODUCT-VISION.en.md)
- Thesis and model: [`DOC/RFC-AINATIVE-VERIFIED-MODEL.md`](DOC/RFC-AINATIVE-VERIFIED-MODEL.en.md)
- Semantics and pipeline: [`DOC/SEMANTICS-V0.1.md`](DOC/SEMANTICS-V0.1.en.md)
- How to program in ARITA with an AI: [`DOC/09-AI-PROGRAMMING.md`](DOC/09-AI-PROGRAMMING.en.md)
- Threat model and evidence: [`DOC/THREAT_MODEL.md`](DOC/THREAT_MODEL.en.md)
- Full index: [`DOC/README.md`](DOC/README.en.md)

## Security

To report a vulnerability, see [SECURITY.md](SECURITY.md).

## License

License: Apache-2.0, see [`LICENSE`](LICENSE).

## Author and contact

Author: Antonio Cantallops Alba

General contact: **contact@exyonq.org**
