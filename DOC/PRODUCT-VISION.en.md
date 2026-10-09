[Español](PRODUCT-VISION.md) | English

# ARITA — Product vision

> **ARITA** is a general-purpose language **designed to be written by AIs** that **compiles to safe Rust**: the AI writes ARITA as its primary language; the compiler rejects ambiguity, smoke, and non-verifiable operations, and emits **real Rust projects** (binary, library, service…) that stand **without depending on the AI**.

## What it is not

| Is not | Why |
|-------|---------|
| An AI tooling platform | Editing tools are not the language |
| A contract or formal-verification DSL | Contracts are optional and will be proportional to risk |
| A Cargo wrapper or “an AI that edits Rust” | The native artifact is the `.arita` |
| A free pseudocode transpiler | Without fixed semantics, “looks like it works” code returns |

## Architecture

`.arita` → lexical/syntactic analyzer → AST → typed HIR → intermediate representation (IR/CFG) → checker → deterministic emitter → real Rust → `cargo`.

## Design for AIs

Few equivalent ways to write the same thing · canonical syntax · local semantics · typed versioned APIs · structured repairable errors · no “compiles but surprises” · visible project · one preferred path per problem.

ARITA does **not** copy Rust literally: no explicit *lifetimes*, *traits*, macros, `unwrap`, panic-on-index, implicit `async`, open dependencies, or `unsafe`.

## Decision

A **new** language, Rust-like but with a smaller, total, canonical semantics, and safe Rust as the *backend*. It is not a Rust dialect.

## Anti-theater

analysis → resolution → types → IR → emission → `cargo` → acceptance tests → policies → evidence.
`scenario` / `acceptance` are **executable** specifications. A skipped test never counts as passed.

## Evidence

JSON manifest keyed by source hash. Confidence levels from “analyzed” to “proved”, plus “blocked” and “unknown”. Forbidden to declare something done without evidence.

## Estado

- **Core 0.9: CLOSED — 834/834.** Index writes into collections (`m[k] = v`, `v[i] = x`) without panic.
- **Core 0.10 (ERRORS, phase 1): in progress.** Eleven phase-1 stages are closed, all those planned for version 1; the pre-version-1 checks remain, and the version-1 declaration. The current bar is in the [roadmap](../ROADMAP.md).
- Core 0.1 through 0.8: closed. See the full ladder in [`../ROADMAP.md`](../ROADMAP.md).

## Compiler-guided repair

The idea: the compiler as a **structured oracle** for bounded autonomous repair, not an error message pasted onto a model. Today it is a design proposal: [`REPAIR-ORACLE.md`](REPAIR-ORACLE.en.md).

## More information

[`RFC-AINATIVE-VERIFIED-MODEL.md`](RFC-AINATIVE-VERIFIED-MODEL.en.md) · [`SEMANTICS-V0.1.md`](SEMANTICS-V0.1.en.md) · [`00-VISION.md`](00-VISION.en.md) · [`THREAT_MODEL.md`](THREAT_MODEL.en.md)
