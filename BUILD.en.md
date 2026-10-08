[Español](BUILD.md) | English

# Build and test ARITA

## Requirements

- Stable Rust toolchain with `cargo` (and `clippy`; optionally `rustfmt`).
- Full oracle verification (`arita measure`) also needs `clippy` on the machine.

## Build the CLI

From the repository root:

```bash
cargo build -p arita-cli
```

The binary lands at `target/debug/arita`. You can also use `cargo run -p arita-cli -- <subcomando>`.

## Build a `.arita` program

```bash
./target/debug/arita build ejemplos/01-hello.arita
```

Output is one line `ok: <ruta>` with the path of the generated binary, which has a hash suffix (for example `target/arita-out/hello_<hash>`). Run that path:

```bash
./target/arita-out/hello_*      # prints: hello
```

Optional profile and target: `arita build [--profile debug|release] [--target <triple>] <fichero.arita>`.

## CLI subcommands

```text
arita build [--profile debug|release] [--target <triple>] <fichero.arita>
arita parse <fichero.arita>
arita test <fichero.arita>
arita logic <fichero.arita>
arita contract <ruta.json|ruta.arita>
arita contract --verify-attest <ruta.json>
arita attest verify <ruta.json>
arita measure
arita lsp
arita version
```

- `arita test` runs the `test` blocks of a program (for example `ejemplos/f2/07-assert.arita`).
- `arita logic` evaluates a logic-island module (`ejemplos/f3/01-path-ok.arita` prints `true`).
- `arita measure` runs all oracles and `clippy`. Exit codes: `0` accepted, `1` rejected, `2` inconclusive.

## Test the workspace

```bash
cargo test --workspace -- --test-threads=1
```

## Full continuous-integration gate (local)

```bash
bash scripts/ci.sh
```

Runs, in this order: format check (if `rustfmt` is available; otherwise warns and continues), `clippy` with `-D warnings`, workspace tests, and `arita measure`. More detail in [`DOC/CI.md`](DOC/CI.md).

## More information

- How to program ARITA with an AI and how to verify: [`DOC/09-AI-PROGRAMMING.md`](DOC/09-AI-PROGRAMMING.en.md)
- Examples: [`ejemplos/README.md`](ejemplos/README.md)
