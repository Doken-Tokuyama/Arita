Translation of `033b-host-border-hello.md`; the original is normative. / Traducción de `033b-host-border-hello.md`; el original es el normativo.

# ADR-033b — Host-border hello (IMPL one-pager)

- **Estado:** **aceptada** + **verified** Lex measure **76/76**
- **CUT-ID:** `HOST-BORDER-HELLO-20260914` ← SoT Ingeniero
- **Alias DOC:** ADR-035 (`035-host-bridge-hello.md`) — same CUT; prefer this id
- **Fecha:** 2026-09-14
- **Padre:** ADR-033 (patrón DOC-only)

## Pins (no bloquean IMPL)

| # | Pin |
|---|-----|
| 1 | `crates/arita-host-demo` en workspace |
| 2 | `[host-bridges]` en `arita.toml` (no mezclar with `[deps]`) |
| 3 | 1 fn bridge; no crate path en `.arita` |
| 4 | Oracle E2E measure (`ejemplos/host/…`); skip ≠ PASS |
| 5 | Safe-only surface/emit usuario; E0261 si surface nombra host |

Detalle ampliado: `DOC/ADR/035-host-bridge-hello.md`.

## OUT

Widen API without CUT; fake PASS.

## Checklist

- [x] One-pager
- [x] CUT alineado Ingeniero
- [x] Measure landed (Ingeniero)
