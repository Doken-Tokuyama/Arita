[Español](THREAT_MODEL.md) | English

# ARITA — Threat model (anti-theater / fake evidence)

- **CUT-ID:** `EVIDENCE-CHAIN-20260913`
- **Estado:** **aceptada** (GO Ingeniero Rust, CUT `EVIDENCE-CHAIN-20260913`)
- **Fecha:** 2026-09-13
- **Fuente:** brief de amigo de <person> (visión “cadena de evidencia”); nombre del lenguaje = **ARITA** (no Veyra)
- **Relacionados:** `DOC/ADR/008-evidence-architecture.md`, ADR-002, ADR-005, ADR-006, barra E2E en `ROADMAP.md`

Product (rev. 3): AI-native language → real Rust — see `RFC-AINATIVE-VERIFIED-MODEL.md`. Evidence JSON/manifest first; UI later.
Repair: compiler as structured oracle (`REPAIR-ORACLE.md` / ADR-231) — no fake success; **HOLD IMPL**.

## Guiding principle

There is no “correct” because it looks right, compiles, has green tests, or produces a demo. Only **`accepted`** exists when declared, reproducible, anti-bypass oracles demonstrate it on the **exact artifact**.

The enemy is not only a bug: it is AI output that optimizes to **appear** to have complied.

## Verdict states (mandatory)

```text
rejected     = hay contradicción, falta evidencia o hay bypass
inconclusive = no se pudo obtener evidencia suficiente
accepted     = todos los oráculos requeridos aprobaron el artefacto exacto
```

**Never** convert `inconclusive` → `accepted`.  
`skip ≠ PASS`. Do not invent PASS. `arita measure` uses these three states (or machine-readable equivalents); a skip/timeout/broken harness = `inconclusive` or `rejected`, never `accepted`.

## Fake/theater modes that must be blocked

| Pattern | Example | Defense in the language/platform |
|---|---|---|
| Disguised stub | `return Ok(default())` on an uncovered branch | Semantic coverage obligations and mutation tests |
| Mock replacing the real system | Test that simulates storage/network instead of the required backend | Mandatory integration oracle and explicit ban on mocks in acceptance profiles |
| Tautological test | The test replicates the same faulty implementation logic | Independent oracle, metamorphic properties, reference implementation or model |
| Memorized fixtures | Detects known inputs and returns expected answers | Secret/deterministic seed generation, hidden corpus, and metamorphic testing |
| Falsified result | CLI prints “passed” without running | Isolated runner, structured logs, exit status, traces, and binary hash |
| Dead code | Implements a correct function but production path uses another | Path-coverage instrumentation and verification of the executed symbol/artifact |
| Guarantee / policy bypass | `unsafe`, FFI, shell, network, or file reads to skip rules | Capability system, explicit effects, and sandbox |
| Theater bench | Measures a trivial path, warm cache, or unreal data | Declared dataset and config, separate warmup, environment fingerprint, metric audit |
| “Proof by assertion” | `assert!(true)`, `unwrap`, `todo!`, `unimplemented!` | Linter as error, unsatisfied-obligation IR, extensible denylist |
| Generated code not executed | A different artifact than the delivered one is validated | Content-addressing and attestation binding source, IR, binary, environment, and result |

## ARITA v0 mapping (already underway)

Does not erase F1/F2/measure: anti-fake of the **language** (executable scenarios + evidence). Formal contracts = optional/proportional (RFC rev. 3), not the product.

| Threat (table) | v0 defense today | Horizon |
|-----------------|----------------|-----------|
| Proof by assertion / stubs | ADR-005/006 anti-theater (`E021x`); denylist `todo!`/`assert true`; **`arita measure` v0** oracle **clippy-workspace** = `cargo clippy -p arita-syntax -p arita-codegen -p arita-cli -- -D warnings` (**required** for overall `accepted`; without clippy on host → `inconclusive`) | Obligation IR |
| Falsified result | `arita measure` / `arita build` + real binary run; skip ≠ PASS; machine-readable JSON | hashes / attestation |
| Generated code not executed | measure/build on emit+`rustc` of the same pipeline (v0 verified) | content-addressing |
| Contract bypass | no `unsafe` in base dialect; explicit `Io` effects (F1.1/F2) | capabilities / sandbox |
| Remaining rows | partially covered or pending | logic island + oracles (F3+) |


## Measure v0 verification

- **Estado:** `arita measure` v0 **verificado** (Ingeniero Rust, 2026-09-13).
- Oracles: `ejemplos/` 01–05 + `f2/01-let-int` (build+run+stdout) **and** `clippy-workspace`.
- **Pin:** overall `accepted` **needs clippy on host**. Clippy absent/not runnable → `inconclusive` (never promoted to `accepted`).
- Usage: `cargo run -p arita-cli -- measure` from the workspace root.

## Checklist GO — cerrado

- [x] Tabla theater OK
- [x] Estados + regla no `inconclusive`→`accepted` OK
- [x] Mapeo v0 OK
- [x] GO `EVIDENCE-CHAIN-20260913` → aceptada + Docs índice

### Vacuous len trap (2026-09-14)

`assert …len() >= 0` / `is_empty` tautologies → **E0214** (ADR-030; do not dilute E0211).

`assert n == n` / `n <= n` / lit==lit → **E0215** (ADR-038; do not dilute E0211/E0214).

`/` `%` with zero divisor (lit or call) → **E0216** (ADR-044; HIR before emit; no rustc/panic theater).

`+/-/*` i64 overflow (lit MAX+1 / silent wrap release) → **E0217** (ADR-045; HIR before emit).
