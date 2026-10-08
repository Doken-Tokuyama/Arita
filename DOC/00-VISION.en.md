[Español](00-VISION.md) | English

# ARITA — Vision

**ARITA** is a **general-purpose AI-native** language that **compiles to Rust**:
the AI writes `.arita` natively; the compiler produces **real Rust workspaces**
(`cargo build` / test / clippy / binary / lib / WASM). The native artifact is ARITA, not “patches on top of Rust”.

Canonical: [`RFC-AINATIVE-VERIFIED-MODEL.md`](RFC-AINATIVE-VERIFIED-MODEL.en.md) (**rev. 3**), [`PRODUCT-VISION.md`](PRODUCT-VISION.en.md), [`SEMANTICS-V0.1.md`](SEMANTICS-V0.1.en.md), [`REPAIR-ORACLE.md`](REPAIR-ORACLE.en.md) (ADR-231).

## Formulation

```text
IA escribe ARITA nativamente
        ↓
 lexer + parser → AST → HIR → AIR/CFG
        ↓
 checker (tipos, ownership, efectos, anti-fake)
        ↓
 emit determinista → Rust safe (Cargo)
        ↓
 scenarios ejecutables + manifiesto de evidencia (UI después)
```

## Is not / Is

| Is not | Is |
|-------|--------|
| MCP/ACP-first platform for agents | GP language with fixed semantics |
| Contract DSL / formal-verification-first | Anti-fake + executable **scenarios** (optional contracts) |
| Cargo wrapper / assistant that edits Rust | Canonical surface → emit Rust workspace |
| Free pseudocode transpiler | **Vertical** Core 0.1 (not self-host first) |

## Why Rust (+ inspired logic island)

| Source | Provides | Do not copy blindly |
|--------|--------|-----------------|
| Rust / rustc | Safe backend, multi-OS/CPU targets, second ownership barrier | Dense lifetimes, panic `[]`, macros as surface |
| Tau/TML (IDNI) | Ideas of decidable specs/rules (own F3 island) | IDNI runtime/code — see `08-LICENSES-IDNI.md` + ADR-002 |

## Repair oracle (delta)

Compiler = structured oracle for bounded repair (`pass`/`repair`/`rollback`/`blocked`) — see [`REPAIR-ORACLE.md`](REPAIR-ORACLE.en.md). **HOLD IMPL** until GO.

## Core 0.1 success

Task to an AI **without** teaching it Rust → writes ARITA → Rust binary → scenarios pass → if it fails, a repairable diagnosis.

## Out of scope (v0)

- Self-hosting first; replace Rust; formal-proof-first; UI dashboard before CLI/CI evidence.
- Embedded IDNI without agreement; LLVM/C-ABI direct as MVP backend (ADR-001 = emit-Rust).

## HOLD

**Do not restart** CUT/measure evidence without **notice to <person> and their GO** (RFC §8–9).
