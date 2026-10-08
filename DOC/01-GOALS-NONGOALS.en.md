[Español](01-GOALS-NONGOALS.md) | English

# Goals and non-goals

## Goals

1. **AI-native surface:** predictable syntax, ASCII keywords, no unbounded macros at the start.
2. **Safe-only:** no `unsafe` in the dialect (ADR-022). There is no later unsafe phase in the measured product.
3. **Cross-compile:** Linux, Windows, macOS × x86_64, aarch64.
4. **Logic island:** `spec` / `query` blocks with TML-inspired semantics (facts + rules).
5. **Measurable:** `arita measure` / tests that do not accept theater (empty asserts, stubs).

## Non-goals (early phases)

- Source-level compatibility with Rust.
- Source-level compatibility with raw `.tau` / TML.
- General GC as the main runtime (prefer ownership).
- Full IDE on day 1 (yes: CLI + minimal LSP in a middle phase).
