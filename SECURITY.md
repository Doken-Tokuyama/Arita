English | [Español](SECURITY.es.md)

# Security policy

## How to report a vulnerability

If you believe you have found a security vulnerability in ARITA (the compiler, the CLI, the Rust code it emits or any other component of this repository), **do not open a public issue**. Write to:

**security@exyonq.org**

Include, if you can:

- a description of the problem and its potential impact;
- the steps to reproduce it (ideally a minimal `.arita` file and the command used);
- the affected version or *commit* and your environment (operating system, Rust version).

## What you can expect

- We will acknowledge receipt of your message within a reasonable time.
- We will assess the report and keep you informed about the fix.
- We ask you to give us a reasonable time to fix the problem before disclosing it publicly.

## Scope

Relevant issues include, for example, flaws that allow the emitted Rust code to contain `unsafe`, that let a compiler check be bypassed so that incorrect code is accepted, or that make the CLI perform unintended actions with crafted input.

## General contact

For any other question: **contact@exyonq.org**.

See also the [README](README.md).
