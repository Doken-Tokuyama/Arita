# ADR-226 — Adopt RFC AI-native Language → Safe Rust

- **Estado:** **propuesta** (rev. 3 — tesis lenguaje-first <person> 2026-09-19; supersede énfasis rev. 2)
- **RFC canónico:** [`DOC/RFC-AINATIVE-VERIFIED-MODEL.md`](../RFC-AINATIVE-VERIFIED-MODEL.md) (**rev. 3**)
- **Face producto:** [`DOC/PRODUCT-VISION.md`](../PRODUCT-VISION.md)
- **Amplía:** ADR-225 (mapa R0–R8 sigue; identidad = lenguaje → Rust)
- **Perfiles (policy):** safe / service / sandboxed / high-assurance / ffi
- **Addenda lenguaje:** [227](227-lang-contract-indexget.md) · [228](228-lang-contract-insert.md) · [229](229-lang-contract-mutex.md)
- **Evidencia:** [230](230-evidence-cuts-v0.1.md) — **HOLD** reinicio hasta aviso+GO <person>
- **HOLD IMPL:** insert / sugar `[]` / Mutex; **no** reiniciar CUT/measure
- **Norte:** Core 0.1 vertical (módulos, record/enum/match, Result, ownership simple, files/JSON/CLI, scenarios, emit forbid(unsafe), bindings curados, 1 ref no trivial)
