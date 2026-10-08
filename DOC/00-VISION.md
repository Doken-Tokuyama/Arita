# ARITA — Visión

**ARITA** es un lenguaje **AI-native de propósito general** que **compila a Rust**:
la IA escribe `.arita` nativamente; el compilador produce **workspaces Rust reales**
(`cargo build` / test / clippy / binario / lib / WASM). El artefacto nativo es ARITA, no “parches sobre Rust”.

Canónico: [`RFC-AINATIVE-VERIFIED-MODEL.md`](RFC-AINATIVE-VERIFIED-MODEL.md) (**rev. 3**), [`PRODUCT-VISION.md`](PRODUCT-VISION.md), [`SEMANTICS-V0.1.md`](SEMANTICS-V0.1.md), [`REPAIR-ORACLE.md`](REPAIR-ORACLE.md) (ADR-231).

## Formulación

```text
IA escribe ARITA nativamente
        ↓
 lexer + parser → AST → HIR → AIR/CFG
        ↓
 checker (tipos, ownership, efectos, anti-fake)
        ↓
 emit determinista → Rust safe (Cargo)
        ↓
 scenarios ejecutables + manifiesto de evidencia (UI después)
```

## No es / Sí es

| No es | Sí es |
|-------|--------|
| Plataforma MCP/ACP-first para agentes | Lenguaje GP con semántica fija |
| DSL de contratos / verificación formal-first | Anti-fake + **scenarios** ejecutables (contratos opcionales) |
| Wrapper de Cargo / asistente que edita Rust | Surface canónica → emit workspace Rust |
| Transpilador de pseudocódigo libre | Core 0.1 **vertical** (no self-host primero) |

## Por qué Rust (+ isla lógica inspirada)

| Fuente | Aporta | No copiar ciego |
|--------|--------|-----------------|
| Rust / rustc | Backend safe, targets multi-OS/CPU, segunda barrera ownership | Lifetimes densos, `[]` panic, macros como superficie |
| Tau/TML (IDNI) | Ideas de specs/reglas decidibles (isla F3 propia) | Runtime/código IDNI — ver `08-LICENSES-IDNI.md` + ADR-002 |

## Repair oracle (delta)

Compilador = oráculo estructurado para corrección acotada (`pass`/`repair`/`rollback`/`blocked`) — ver [`REPAIR-ORACLE.md`](REPAIR-ORACLE.md). **HOLD IMPL** hasta GO.

## Éxito Core 0.1

Tarea a una IA **sin** enseñarle Rust → escribe ARITA → binario Rust → scenarios pasan → si falla, diagnóstico reparable.

## Fuera de alcance (v0)

- Self-hosting primero; reemplazar Rust; formal-proof-first; dashboard UI antes que evidencia CLI/CI.
- Embebido IDNI sin acuerdo; LLVM/C-ABI directo como backend MVP (ADR-001 = emit-Rust).

## HOLD

**No reiniciar** CUT/measure evidence sin **aviso a <person> y su GO** (RFC §8–9).
