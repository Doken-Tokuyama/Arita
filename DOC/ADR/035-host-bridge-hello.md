# ADR-035 — Host-border hello bridge (IMPL)

- **Estado:** **aceptada** + **verified** Lex measure **76/76**
- **CUT-ID:** `HOST-BORDER-HELLO-20260914` (SoT; alias histórico `HOST-BRIDGE-HELLO` descartado)
- **One-pager:** `DOC/ADR/033b-host-border-hello.md`
- **Fecha:** 2026-09-14
- **Autores:** ARITA Arquitecto (pins/GO) + Ingeniero Rust (IMPL executor)
- **Relacionados:** ADR-033 (DOC pattern), **ADR-033b** one-pager (mismo CUT), ADR-029 (`[deps]`), ADR-022 / E0260/E0261
- **Gobernanza:** CUT IMPL aparte de 033. Executor Ingeniero; Parser/Codegen HOLD review. Safe-only.
- **Barra:** E2E measure oráculo real; skip ≠ PASS.

## Contexto

ADR-033 aceptó el patrón DOC-only. <person>: pipeline continuo → hello mínimo medible.

## Pins GO (`HOST-BORDER-HELLO-20260914`)

1. Crate host: **`crates/arita-host-demo`** (workspace member).
2. Manifiesto ejemplo: `arita.toml` con **`[host-bridges]`** (no meter el host en `[deps]`):
   ```toml
   [host-bridges]
   demo = { crate = "arita-host-demo", path = "crates/arita-host-demo" }
   ```
3. Bridge surface: **una** fn, p.ej. `host_demo_mark() -> Io<()>` o `host_demo_ping() -> Int` que imprime/`return` valor fijado — **sin** path crate en `.arita`.
4. Host puede usar dep crates.io **interna** (opcional, p.ej. nada o crate safe trivial) — invisible al surface.
5. Emit/link: crate generado depende del host vía path; `#![forbid(unsafe_code)]` en código usuario; host puede tener unsafe **solo** interno (preferir host 100% safe en v0 hello).
6. Diagnósticos: E0261 si `.arita` nombra el host/crate; E0260 no aplica a `[host-bridges]` (sección distinta).
7. Oráculo measure:
   - `ejemplos/host/01-bridge-hello.arita` + `arita.toml` → stdout canónico (p.ej. `host-ok`)
   - opc. neg E0261

## OUT

- Ampliar API bridge sin CUT
- Meter host en `[deps]` whitelist
- Fake PASS sin execute

## Checklist

- [x] Pins layout + `[host-bridges]` + 1 fn OK
- [x] GO DOC+IMPL
- [x] Landed + measure (Ingeniero)


## IMPL evidence (`HOST-BORDER-HELLO-20260914`)

| Id | Path | Expect | Verdict |
|----|------|--------|---------|
| host-01-bridge-hello | `ejemplos/host/01-bridge-hello.arita` + `arita.toml` | E2E stdout `host-ok`; Cargo path-dep `arita-host-demo` | accepted (measure) |

**Surface syntax (v0):** reserved `host.<fn>(…)` → emit `arita_host_demo::<fn>(…)`. Surface never names host crate paths (E0261). `[host-bridges]` separate from `[deps]`.

- Verified Lex **76/76** accepted (`HOST-BORDER-HELLO-20260914`). HOLD crates → review-only.
