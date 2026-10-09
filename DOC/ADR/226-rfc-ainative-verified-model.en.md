Translation of `226-rfc-ainative-verified-model.md`; the original is normative. / Traducción de `226-rfc-ainative-verified-model.md`; el original es el normativo.

# ADR-226 — Adopt RFC AI-native Language → Safe Rust

- **Estado:** **propuesta** (rev. 3 — tesis lenguaje-first <person> 2026-09-19; supersede énfasis rev. 2)
- **Canonical RFC:** [`DOC/RFC-AINATIVE-VERIFIED-MODEL.md`](../RFC-AINATIVE-VERIFIED-MODEL.en.md) (**rev. 3**)
- **Product face:** [`DOC/PRODUCT-VISION.md`](../PRODUCT-VISION.en.md)
- **Extends:** ADR-225 (R0–R8 map remains; identity = language → Rust)
- **Profiles (policy):** safe / service / sandboxed / high-assurance / ffi
- **Language addenda:** [227](227-lang-contract-indexget.en.md) · [228](228-lang-contract-insert.en.md) · [229](229-lang-contract-mutex.en.md)
- **Evidence:** [230](230-evidence-cuts-v0.1.en.md) — **HOLD** restart until notice+GO <person>
- **HOLD IMPL:** insert / sugar `[]` / Mutex; **no** reiniciar CUT/measure
- **North star:** Core 0.1 vertical (modules, record/enum/match, Result, simple ownership, files/JSON/CLI, scenarios, emit forbid(unsafe), curated bindings, 1 non-trivial ref)
