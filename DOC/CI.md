# ARITA — CI

Local-first gate. No demo keys. **skip ≠ PASS.**

## Run locally

From the repository root:

```bash
bash scripts/ci.sh
# or: ./scripts/ci.sh
```

### macOS (aarch64)

```bash
cd <repository-root>
bash scripts/ci.sh
```

Needs Rust stable with `clippy`. Prefer `rustup component add rustfmt clippy` so fmt is checked (not skipped).

### Linux (CI)

```bash
cd <repository-root>
bash scripts/ci.sh
```

GitHub Actions (when a remote exists): `.github/workflows/ci.yml` runs the same script on `ubuntu-latest` and `macos-latest` with pinned Rust 1.97.1 (default; clippy + rustfmt) plus nightly-2026-09-13 (miri + rust-src; checked against commit 809936eac); on Linux it also adds the aarch64-unknown-linux-gnu target and cross linker.

## What the script does

1. **fmt** — `cargo fmt --check` if `rustfmt` works; otherwise **WARN** and continue (inconclusive-style; does **not** claim fmt passed).
2. **clippy** — workspace crates with `-D warnings` (`arita-syntax`, `arita-codegen`, `arita-hir`, `arita-logic`, `arita-cli`).
3. **test** — `cargo test --workspace -- --test-threads=1`.
4. **measure** — `cargo run -q -p arita-cli -- measure`.

## Measure bar / exit codes

| Exit | Meaning |
|------|---------|
| 0 | `accepted` |
| 1 | `rejected` |
| 2 | `inconclusive` / usage |

Overall `accepted` still follows product rules (e.g. clippy on host). Missing tools → skip or inconclusive, **never** invent PASS.


## Current local bar

- **Estado (local):** **verified** (fmt+clippy+test+measure exit 0; ROADMAP #57)
- **CUT:** `CI-LOCAL-VERIFY-20260915` (infra; after `TARGETS-CROSS-LEX-20260915`)
- **Measure:** **889/889 accepted** (0 skip) when the host has rustup targets + linkers (see `DOC/05-COMPILATION-TARGETS.md` / `.cargo/config.toml`)
- Without cross linkers: overall still `accepted` with `target-*` **inconclusive** gated (never fake PASS)
- En la CI pública de GitHub, un `target-*` inconclusive gated se acepta por diseño (ADR-034): no tumba el total y nunca cuenta como accepted.
- Remote GitHub matrix: **PARK** until remote exists

## Remote status

Workflow YAML is present; matrix on a real remote remains pending until GitHub is connected.

## Addendum — CI-LOCAL-VERIFY (**verified**)

- **CUT:** `CI-LOCAL-VERIFY-20260915` (infra P2; after `TARGETS-CROSS-LEX-20260915`)
- **IN:** `scripts/ci.sh` (fmt WARN-ok / clippy `-D warnings` / test / measure); `DOC/CI.md`; `.github/workflows/ci.yml` present
- **Local:** exit 0 + measure **89/89** accepted (**0 gated**); ROADMAP #57 closed
- **OUT / PARK:** GitHub remote matrix; fake PASS; skip≠PASS
- **Next:** ROADMAP #58 standby (trap crates PARK; CI on GitHub; ADR-036/ports PARK)
