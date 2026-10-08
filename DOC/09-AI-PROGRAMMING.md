# ARITA — Cómo debe programar una IA

- **Estado:** **aceptada** / **final** (GO Ingeniero Rust, 2026-09-13)
- **Fecha:** 2026-09-13
- **Nombre:** **ARITA** (no Veyra)
- **Audiencia:** agentes / LLMs que generan `.arita`
- **Relacionados:** ADR-005, `PACK-F1.1-FEWSHOT.md`, `PACK-F2-FEWSHOT.md`, `PACK-F2.2-FEWSHOT.md`, `PACK-F3-FEWSHOT.md`, `PACK-STD-METHOD-SURFACE-F2.md`, `THREAT_MODEL.md`, ADR-008, `ejemplos/`

## 1. Qué cargar en contexto (orden)

Cargar **en este orden** (no inventar surface fuera de lo cargado):

| # | Doc / path | Para qué |
|---|------------|----------|
| 1 | `DOC/ADR/005-f1.1-surface-freeze.md` | Surface F1.1 canónica + `E0xxx` |
| 2 | `DOC/PACK-F1.1-FEWSHOT.md` | System prompt largo + **few-shots reales** (fuente few-shot) |
| 3 | `DOC/THREAT_MODEL.md` | Veredictos `rejected` \| `inconclusive` \| `accepted`; **nunca** inconclusive→accepted |
| 4 | `ejemplos/01`–`05` (+ `ejemplos/README.md`) | Oráculos E2E F1.1 (compile+run) |
| 5 | Tabla `E0xxx` (en ADR-005 o PACK) | Diagnósticos estables; texto EN tras el código |
| 6 | **Solo si la tarea pide `let` / `Int` / F2** | `DOC/PACK-F2-FEWSHOT.md` (**final**) + `DOC/ADR/006-f2-ownership-surface.md` + `ejemplos/f2/` (+ `neg/` / ADR-010 si anti-theater) |
| 6a | **Solo si la tarea pide checked/saturating/wrapping / unwrap_or / is_*** | `DOC/PACK-STD-METHOD-SURFACE-F2.md` (**DOC v0**) + guías `DOC/guides/aritmetica-int-v0.md` / `option-result-helpers-v0.md`; E0206 en receptor ilegal; ADR-098 **verified** **244/244** |
| 6b | **Solo si la tarea pide `match` / F2.2** | `DOC/PACK-F2.2-FEWSHOT.md` (**final**) + `DOC/ADR/015-f22-match.md` + `ejemplos/f2.2/` |
| 7 | **Solo si la tarea pide F3 / `spec`/`fact`/`rule`/`query`** | `DOC/PACK-F3-FEWSHOT.md` (**final**) + `DOC/ADR/012-logic-island-v0.md` + `ejemplos/f3/` |

Opcional de fondo: `DOC/09-AI-PROGRAMMING.md` (este doc), `DOC/ADR/008-evidence-architecture.md`.

**Default = F1.1.** No cargar F2/F3 “por si acaso”.

## 2. Qué NO inventar

- Rust disfrazado (lifetimes, `unwrap`, macros, `tokio`, crates externos).
- Theater: `todo!`, `unimplemented!`, `assert true`, stubs, mocks de aceptación, PASS decorativo.
- Surface fuera de F1.1 si la tarea es F1.1 (`let`, otros `fn`, `spec`, … → E0006).
- Éxito sin oráculo: compilar ≠ `accepted`; demo ≠ `accepted`.
- Código IDNI / APIs Tau-TML como runtime.

## 3. System prompt corto (copy-paste)

Más corto que el PACK; el PACK sigue siendo la fuente de few-shots.

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

## 4. Cómo verificar (obligatorio)

```bash
# desde <repo>
cargo run -p arita-cli -- build ejemplos/01-hello.arita
./target/arita-out/hello

# oráculo completo v0 (ejemplos + clippy-workspace)
cargo run -p arita-cli -- measure
```

| Resultado | Veredicto |
|-----------|-----------|
| Build+run OK y stdout = oráculo **y** clippy OK | **accepted** (todos los oráculos del contrato) |
| Fallo de parse/emit/rustc/run / stdout mismatch / clippy fail | **rejected** |
| No se pudo ejecutar / skip / harness roto / **clippy no disponible en host** | **inconclusive** (nunca promover a accepted) |

`arita measure` v0 **verificado**: mismos contratos/oráculos (ADR-008 / THREAT_MODEL). Overall `accepted` **needs clippy on host**. Exit: `0` accepted, `1` rejected, `2` inconclusive.

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


## 5b. Method surface F2 (arith + Option/Result) — 1 línea / modo

```text
checked → Option; saturating → clamp Int; wrapping → wrap Int; unwrap_or → T; is_some/none/ok/err → Bool
Wrong receiver → E0206. Mutex/`[]`/insert PARK. No inventar barra (SoT: STABLE_VERIFY / mirrors).
```

Detalle denso: `DOC/PACK-STD-METHOD-SURFACE-F2.md`.
