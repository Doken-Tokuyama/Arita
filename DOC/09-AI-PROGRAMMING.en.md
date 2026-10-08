[Español](09-AI-PROGRAMMING.md) | English

# ARITA — How an AI should program

- **Estado:** **aceptada** / **final** (GO Ingeniero Rust, 2026-09-13)
- **Fecha:** 2026-09-13
- **Nombre:** **ARITA** (no Veyra)
- **Audiencia:** agentes / LLMs que generan `.arita`
- **Relacionados:** ADR-005, `PACK-F1.1-FEWSHOT.md`, `PACK-F2-FEWSHOT.md`, `PACK-F2.2-FEWSHOT.md`, `PACK-F3-FEWSHOT.md`, `PACK-STD-METHOD-SURFACE-F2.md`, `THREAT_MODEL.md`, ADR-008, `ejemplos/`

## 1. What to load into context (order)

Load **in this order** (do not invent surface outside what was loaded):

| # | Doc / path | Purpose |
|---|------------|----------|
| 1 | `DOC/ADR/005-f1.1-surface-freeze.md` | Canonical F1.1 surface + `E0xxx` |
| 2 | `DOC/PACK-F1.1-FEWSHOT.md` | Long system prompt + **real few-shots** (few-shot source) |
| 3 | `DOC/THREAT_MODEL.md` | Verdicts `rejected` \| `inconclusive` \| `accepted`; **never** inconclusive→accepted |
| 4 | `ejemplos/01`–`05` (+ `ejemplos/README.md`) | F1.1 E2E oracles (compile+run) |
| 5 | `E0xxx` table (in ADR-005 or PACK) | Stable diagnostics; EN text after the code |
| 6 | **Only if the task needs `let` / `Int` / F2** | `DOC/PACK-F2-FEWSHOT.md` (**final**) + `DOC/ADR/006-f2-ownership-surface.md` + `ejemplos/f2/` (+ `neg/` / ADR-010 if anti-theater) |
| 6a | **Only if the task needs checked/saturating/wrapping / unwrap_or / is_*** | `DOC/PACK-STD-METHOD-SURFACE-F2.md` (**DOC v0**) + guides `DOC/guides/aritmetica-int-v0.md` / `option-result-helpers-v0.md`; E0206 on illegal receiver; ADR-098 **verified** **244/244** |
| 6b | **Only if the task needs `match` / F2.2** | `DOC/PACK-F2.2-FEWSHOT.md` (**final**) + `DOC/ADR/015-f22-match.md` + `ejemplos/f2.2/` |
| 7 | **Only if the task needs F3 / `spec`/`fact`/`rule`/`query`** | `DOC/PACK-F3-FEWSHOT.md` (**final**) + `DOC/ADR/012-logic-island-v0.md` + `ejemplos/f3/` |

Optional background: `DOC/09-AI-PROGRAMMING.md` (this doc), `DOC/ADR/008-evidence-architecture.md`.

**Default = F1.1.** Do not load F2/F3 “just in case”.

## 2. What NOT to invent

- Disguised Rust (lifetimes, `unwrap`, macros, `tokio`, external crates).
- Theater: `todo!`, `unimplemented!`, `assert true`, stubs, acceptance mocks, decorative PASS.
- Surface outside F1.1 if the task is F1.1 (`let`, other `fn`, `spec`, … → E0006).
- Success without an oracle: compile ≠ `accepted`; demo ≠ `accepted`.
- IDNI code / Tau-TML APIs as runtime.

## 3. Short system prompt (copy-paste)

Shorter than the PACK; the PACK remains the few-shot source.

### Español

```text
Eres un programador ARITA (no Rust). Por defecto escribe solo F1.1:

module <ident>
fn main() -> Io<()> {
  print("...")
}

Una forma canónica. Sin let/otros fn/spec/todo!/assert true. Diagnósticos E0xxx con texto en inglés.
No declares PASS: solo accepted si `arita measure` (o build+run + clippy) cumple oráculos; skip ≠ PASS.
skip ≠ PASS; never treat inconclusive as accepted.
Few-shots y oráculos: DOC/PACK-F1.1-FEWSHOT.md y ejemplos/01–05.
Si la tarea exige let/Int/F2, carga DOC/PACK-F2-FEWSHOT.md (+ ADR-006 + ejemplos/f2); si pide match/F2.2, DOC/PACK-F2.2-FEWSHOT.md (+ ADR-015 + ejemplos/f2.2); no inventes surface.
Si la tarea exige spec/fact/rule/query (F3), carga DOC/PACK-F3-FEWSHOT.md (+ ADR-012 + ejemplos/f3); módulos lógicos separados; cero IDNI.
```

### English

```text
You write ARITA (not Rust). Default surface is F1.1 only:

module <ident>
fn main() -> Io<()> {
  print("...")
}

One canonical shape. No let / other fns / spec / todo! / assert true. Diagnostics: E0xxx + English text.
Do not claim PASS. accepted only when `arita measure` (or build+run + clippy) passes oracles; skip ≠ PASS.
skip ≠ PASS; never inconclusive → accepted.
Few-shots/oracles: DOC/PACK-F1.1-FEWSHOT.md and ejemplos/01–05.
If the task needs let/Int/F2, load DOC/PACK-F2-FEWSHOT.md (+ ADR-006 + ejemplos/f2); if match/F2.2, DOC/PACK-F2.2-FEWSHOT.md (+ ADR-015 + ejemplos/f2.2); do not invent surface.
If the task needs spec/fact/rule/query (F3), load DOC/PACK-F3-FEWSHOT.md (+ ADR-012 + ejemplos/f3); separate logic modules; no IDNI.
```

## 4. How to verify (mandatory)

```bash
# desde <repo>
cargo run -p arita-cli -- build ejemplos/01-hello.arita
./target/arita-out/hello

# oráculo completo v0 (ejemplos + clippy-workspace)
cargo run -p arita-cli -- measure
```

| Result | Verdict |
|-----------|-----------|
| Build+run OK and stdout = oracle **and** clippy OK | **accepted** (all contract oracles) |
| parse/emit/rustc/run failure / stdout mismatch / clippy fail | **rejected** |
| Could not run / skip / broken harness / **clippy unavailable on host** | **inconclusive** (never promote to accepted) |

`arita measure` v0 **verified**: same contracts/oracles (ADR-008 / THREAT_MODEL). Overall `accepted` **needs clippy on host**. Exit: `0` accepted, `1` rejected, `2` inconclusive.

## Checklist GO (Ingeniero) — cerrado

- [x] Orden de contexto OK
- [x] NO inventar OK
- [x] Prompt corto ES/EN OK (PACK = few-shots)
- [x] Verify OK
- [x] GO → aceptada/final + Docs índice README

## Enlaces

- `DOC/PACK-F1.1-FEWSHOT.md`, `DOC/PACK-F2-FEWSHOT.md`, `DOC/PACK-F2.2-FEWSHOT.md`, `DOC/PACK-F3-FEWSHOT.md`, `DOC/ADR/005-f1.1-surface-freeze.md`
- `DOC/THREAT_MODEL.md`, `DOC/ADR/008-evidence-architecture.md`
- `ejemplos/`, `ejemplos/f2/`


## 5b. Method surface F2 (arith + Option/Result) — 1 line / mode

```text
checked → Option; saturating → clamp Int; wrapping → wrap Int; unwrap_or → T; is_some/none/ok/err → Bool
Wrong receiver → E0206. Mutex/`[]`/insert PARK. No inventar barra (SoT: STABLE_VERIFY / mirrors).
```

Dense detail: `DOC/PACK-STD-METHOD-SURFACE-F2.md`.
