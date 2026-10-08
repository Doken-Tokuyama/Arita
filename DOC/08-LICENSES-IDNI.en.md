[Español](08-LICENSES-IDNI.md) | English

# IDNI licenses (Tau / TML / parser) — F0 note

- **Fecha:** 2026-09-13
- **Estado:** revisado (Fase 0)
- **Relacionados:** `DOC/ADR/002-isla-logica-propia.md`, `DOC/ADR/001-emit-rust-mvp.md`

## Findings (public repos)

| Repo | Finding |
|------|----------|
| [IDNI/tau-lang](https://github.com/IDNI/tau-lang) | Proprietary IDNI AG license. Limited free use (e.g. Tau Net / educational / non-commercial / eval). **Forbids redistributing** the software. Commercial use = separate agreement. SPDX on GitHub: `NOASSERTION` / Other. |
| [IDNI/parser](https://github.com/IDNI/parser) | Same style; strict terms (no Tau Net-equivalent carve-out). Do not redistribute / no commercial use without agreement. |
| [IDNI/TML](https://github.com/IDNI/TML) | No `LICENSE` at root; `license: null` on GitHub; activity ~2023. Legal status **opaque**. |

*Factual summary for a product decision; not legal advice.*

## Consequence for ARITA

1. **Do not** copy IDNI code or submodule/link Tau/TML/parser into the redistributable product **without** a commercial license.
2. **Do** take semantic inspiration from Tau/TML (shape of specs, mental model). That is **not** linking: there is no link-license block while ARITA neither links nor redistributes IDNI binaries/code.
3. Architecture default: **own logic island** (ADR-002). Product parser: **pest or lalrpop** (decide in F1), not IDNI-parser.
4. If <person> closes a commercial IDNI agreement, open a binding ADR; until then this note + ADR-002 govern.

## What to document for AIs / few-shots

Do not present Tau/TML APIs as the ARITA runtime.
