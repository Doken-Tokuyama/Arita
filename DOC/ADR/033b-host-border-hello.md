# ADR-033b — Host-border hello (IMPL one-pager)

- **Estado:** **aceptada** + **verified** Lex measure **76/76**
- **CUT-ID:** `HOST-BORDER-HELLO-20260914` ← SoT Ingeniero
- **Alias DOC:** ADR-035 (`035-host-bridge-hello.md`) — mismo CUT; preferir este id
- **Fecha:** 2026-09-14
- **Padre:** ADR-033 (patrón DOC-only)

## Pins (no bloquean IMPL)

| # | Pin |
|---|-----|
| 1 | `crates/arita-host-demo` en workspace |
| 2 | `[host-bridges]` en `arita.toml` (no mezclar con `[deps]`) |
| 3 | 1 fn bridge; sin path crate en `.arita` |
| 4 | Oráculo E2E measure (`ejemplos/host/…`); skip ≠ PASS |
| 5 | Safe-only surface/emit usuario; E0261 si surface nombra host |

Detalle ampliado: `DOC/ADR/035-host-bridge-hello.md`.

## OUT

Ampliar API sin CUT; fake PASS.

## Checklist

- [x] One-pager
- [x] CUT alineado Ingeniero
- [x] Measure landed (Ingeniero)
