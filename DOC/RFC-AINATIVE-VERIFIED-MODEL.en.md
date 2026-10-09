[Español](RFC-AINATIVE-VERIFIED-MODEL.md) | English

# RFC — ARITA AI-native Language → Safe Rust (v0.1)

- **Estado:** **propuesta** (rev. 3 — tesis <person> 2026-09-19: *lenguaje real*, no plataforma)
- **ID:** `RFC-AINATIVE-LANGUAGE-20260919`
- **Supersede:** rev. 2 (profiles/proportional) in *product emphasis*; profiles and ownership remain in force as tooling
- **HOLD:** reinicio de CUT/measure **solo tras aviso a <person> y su GO**

## 0. Canonical formulation

> **ARITA** is a **general-purpose AI-native** language that **compiles to Rust**: the AI writes ARITA as its primary language; ARITA rejects ambiguity, smoke, and non-verifiable operations; it emits **real Rust projects** (build / test / clippy / binary / lib / WASM / service) that are maintained **without depending on the AI**.

### Is not

| Anti-product | Why not |
|---------------|------------|
| Platform to control agents (MCP/ACP-first) | Editing tooling; not the language |
| Contract DSL / formal-verification-first | Optional contracts proportional to risk |
| Cargo wrapper / assistant that edits Rust | The native artifact is `.arita`, not “patches on Rust” |
| Free pseudocode transpiler | Without fixed semantics → smoke returns |

### Is

```text
IA escribe ARITA nativamente
        ↓
 lexer + parser ARITA
        ↓
 AST → HIR tipado → AIR/CFG semántico
        ↓
 checker: tipos, ownership, efectos, concurrencia, anti-fake
        ↓
 emisor determinista
        ↓
 Rust real (workspace Cargo)
        ↓
 cargo build / test / clippy / deploy…
```

## 1. AI-native (design against LLM failures)

Few equivalent forms · canonical syntax · local semantics · typed/versioned APIs · structured repairable diagnostics · no “compiles but surprises” · project visible in the manifest · one preferred path per common problem.

| Rust as-is | Problem for AI | ARITA |
|---------------|------------------|-------|
| Dense lifetimes / traits | Invents or clone-spam | Canonical ownership + correct Rust emit |
| `unwrap` / `panic!` / partial `[]` | Fake demo | Surface without them; fallible/total |
| Open crates.io | Ghost API | Curated bindings + typed manifest |
| Free `unsafe`/FFI | Escape from guarantees | Only audited `native`/`ffi` packages |

## 2. New language (rust-like), not a dialect

Familiar syntax; **deliberately smaller, total, canonical semantics**. Backend = safe Rust + interoperability. Do not inherit macros/lifetimes/panic `Index` as surface.

## 3. Anti-fake (mandatory chain)

1. Parse + resolve (real symbols/bindings)  
2. Type + semantic check  
3. Lower IR / AIR  
4. Deterministic Rust emit  
5. `cargo` + Clippy  
6. Tests / executable **acceptance scenarios**  
7. Policies (unsafe/panic/effects/deps)  
8. **Evidence manifest** (hash source↔emit↔result)

`scenario` / acceptance = **executable** specification (not formal `requires`/`ensures` by default).

## 4. Compiler architecture

AST (faithful to text) → Resolver → HIR (sugar out, DefId) → Typed HIR → **AIR/CFG** (moves, borrows, async, effects) → policy + test compiler + **structured Rust emitter** (not string concatenation).

`rustc` = **second barrier** for ownership/memory.

## 5. Ownership v0.1 (canonical for AI)

Owned by default · move unless Copy · `borrow` / `borrow mut` · no refs escaping in structs/return at the start · `share` / actor / `with_lock` · mutation with lexical scope · no exposed `Arc<Mutex<T>>`.

## 6. Evidence (CLI/CI first; UI later)

`target/arita/evidence/<source-hash>.json` with levels: `parsed` … `tested` / `bounded_verified` / `proved` / `blocked` / `unknown`.  
Forbidden to label DONE if there is a stub, acceptance without a test, hash mismatch, undeclared dep, or “verified” when there is only `tested`.

## 7. Core 0.1 (vertical, not self-host first)

Modules · record/enum/match · simple generics · Text/Bytes/checked numbers · List/Map · Option/Result · simple ownership · files/JSON/CLI · scenarios · emit `#![forbid(unsafe_code)]` · curated bindings · **one** non-trivial reference program (CLI/HTTP daemon).

**Success criterion:** task to an AI **without** teaching it Rust → writes ARITA → Rust binary → scenarios pass → if it fails, the diagnosis says what is missing.

## 8. Relation to rev. 2 / ADR-225–230

- Profiles `safe|service|sandboxed|high-assurance|ffi` remain as **compilation policy**.  
- R0–R8 / E0310… = surface rules; **GO restart** <person>: close R4 → Core 0.1 (ADR-232).  
- Insert / `[]` sugar / Mutex: **language** fallible/total contracts after ordered restart.

## 9. Checklist

- [x] Tesis lenguaje-first documentada (rev. 3)  
- [x] Arquitecto alinea ROADMAP + faces (2026-09-19)
- [ ] Docs signs index/THREAT_MODEL  
- [x] Aviso/GO <person> reinicio (R4 → Core 0.1)  
- [x] Repair oracle DOC (REPAIR-ORACLE.md + ADR-231 propuesta)
- [x] Pins Core 0.1 → ADR-232
- [x] R4 CLOSED Lex **583/583**
- [x] Core 0.1 CLOSED Lex **603/603**

## 10. Repair oracle (delta <person> 2026-09-19)

The compiler is a **structured oracle** for bounded autonomous repair — not an error string pasted onto the LLM.

Normative: [`REPAIR-ORACLE.md`](REPAIR-ORACLE.en.md) · ADR-231.

- Loop: verifiable goal → minimal change → ARITA/Rust checks → classify → repair/rollback/blocked  
- Pyramid 0–9 (parser → … → runtime); cheap success does **not** cover expensive failure  
- JSON diagnostics (`ARI-*`, spans, `allowed_repairs` / `forbidden_repairs`)  
- Future CLI: `arita repair` — **HOLD IMPL** until restart + GO  
- **No fake success:** without a valid autonomous repair ⇒ honest state (`blocked_by_ambiguity`, etc.)

## 11. Core 0.2 (`service` profile)

Pins: [ADR-233](ADR/233-core-0.2-pins.md). Prereq Core 0.1 CLOSED. Vertical = HTTP daemon + explicit async.

- [x] Pins Core 0.2 → ADR-233
- [ ] Core 0.2 CLOSED Lex
