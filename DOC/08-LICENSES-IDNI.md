# Licencias IDNI (Tau / TML / parser) — nota F0

- **Fecha:** 2026-09-13
- **Estado:** revisado (Fase 0)
- **Relacionados:** `DOC/ADR/002-isla-logica-propia.md`, `DOC/ADR/001-emit-rust-mvp.md`

## Hallazgos (repos públicos)

| Repo | Hallazgo |
|------|----------|
| [IDNI/tau-lang](https://github.com/IDNI/tau-lang) | Licencia propietaria IDNI AG. Uso gratis limitado (p. ej. Tau Net / educativo / no comercial / eval). **Prohíbe redistribuir** el software. Uso comercial = acuerdo aparte. SPDX en GitHub: `NOASSERTION` / Other. |
| [IDNI/parser](https://github.com/IDNI/parser) | Mismo estilo; términos estrictos (sin carve-out equivalente a Tau Net). No redistribuir / no comercial sin acuerdo. |
| [IDNI/TML](https://github.com/IDNI/TML) | Sin `LICENSE` en raíz; `license: null` en GitHub; actividad ~2023. Estado legal **opaco**. |

*Resumen factual para decisión de producto; no es asesoría legal.*

## Consecuencia para ARITA

1. **No** copiar código IDNI ni hacer submodule/link de Tau/TML/parser en el producto redistribuible **sin** licencia comercial.
2. **Sí** inspirarse en Tau/TML a nivel semántico (forma de specs, modelo mental). Eso **no** es link: no hay bloqueo por licencia de link mientras ARITA no enlace ni redistribuya binarios/código IDNI.
3. Default de arquitectura: **isla lógica propia** (ADR-002). Parser de producto: **pest o lalrpop** (decidir en F1), no IDNI-parser.
4. Si <person> cierra acuerdo comercial IDNI, abrir ADR de binding; hasta entonces mandan esta nota + ADR-002.

## Qué documentar a IAs / few-shots

No presentar APIs de Tau/TML como runtime de ARITA.
