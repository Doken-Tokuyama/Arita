# ARITA — Documentación

ARITA es un lenguaje de propósito general **pensado para que lo escriban las IAs** y que **compila a Rust seguro**. Esta carpeta reúne la visión, la arquitectura, la semántica, las guías y las decisiones de diseño.

**Estado:** Core 0.9 cerrado (834/834 oráculos). Core 0.10 (ERRORES, fase 1) en curso: once etapas de la fase 1 están cerradas, todas las previstas para la versión 1 (barra actual 889/889); siguen las comprobaciones previas a la versión 1. Ver [`../ROADMAP.md`](../ROADMAP.md).

## Empieza aquí

| Documento | Contenido |
|-----------|-----------|
| [00-VISION.md](00-VISION.md) | Visión: lenguaje nativo de IA que compila a Rust real |
| [PRODUCT-VISION.md](PRODUCT-VISION.md) | Visión de producto y estado de alto nivel |
| [RFC-AINATIVE-VERIFIED-MODEL.md](RFC-AINATIVE-VERIFIED-MODEL.md) | Tesis: lenguaje primero, verificable, a Rust seguro |
| [SEMANTICS-V0.1.md](SEMANTICS-V0.1.md) | Semántica y pipeline (AST → HIR → AIR → emisión) |
| [09-AI-PROGRAMMING.md](09-AI-PROGRAMMING.md) | Cómo debe programar una IA en ARITA y cómo verificar |
| [../BUILD.md](../BUILD.md) | Cómo compilar y probar |
| [CI.md](CI.md) | Puerta de integración continua local |

## Fundamentos

| Documento | Contenido |
|-----------|-----------|
| [01-GOALS-NONGOALS.md](01-GOALS-NONGOALS.md) | Objetivos y no-objetivos |
| [02-ARCHITECTURE.md](02-ARCHITECTURE.md) | Arquitectura |
| [03-LANGUAGE-SKETCH.md](03-LANGUAGE-SKETCH.md) | Boceto de sintaxis |
| [04-AI-ERGONOMICS.md](04-AI-ERGONOMICS.md) | Diseño pensado para IAs |
| [05-COMPILATION-TARGETS.md](05-COMPILATION-TARGETS.md) | Compilación multiplataforma |
| [06-RISKS.md](06-RISKS.md) | Riesgos |
| [07-REFERENCES.md](07-REFERENCES.md) | Referencias |
| [08-LICENSES-IDNI.md](08-LICENSES-IDNI.md) | Nota sobre licencias de proyectos de lógica externos |

## Evidencia y seguridad del código

| Documento | Contenido |
|-----------|-----------|
| [THREAT_MODEL.md](THREAT_MODEL.md) | Modelo de amenazas: evidencia falsa y "teatro" |
| [TRAPS-CATALOG.md](TRAPS-CATALOG.md) | Catálogo de trampas y su defensa medida |
| [EVIDENCE-GAPS-V0.1.md](EVIDENCE-GAPS-V0.1.md) | Huecos de evidencia frente a la semántica v0.1 |
| [EVIDENCE-OOB-NOLIT-SUITE.md](EVIDENCE-OOB-NOLIT-SUITE.md) | Suite de fuera de rango / sin literal |
| [CODEGEN-EMIT-WHITELIST.md](CODEGEN-EMIT-WHITELIST.md) | Métodos permitidos en la emisión de Rust |
| [REPAIR-ORACLE.md](REPAIR-ORACLE.md) | Corrección guiada por el compilador (propuesta de diseño) |
| [PERF-HOST-BORDER.md](PERF-HOST-BORDER.md) | Rendimiento: qué queda fuera del lenguaje |
| [LSP-EDITOR.md](LSP-EDITOR.md) | Servidor de lenguaje para editores (`arita lsp`) |

## Isla lógica

| Documento | Contenido |
|-----------|-----------|
| [LOGIC-WHEN-TO-USE.md](LOGIC-WHEN-TO-USE.md) | Cuándo usar la isla lógica |
| [LOGIC-INT-ARITH-OUT.md](LOGIC-INT-ARITH-OUT.md) | Aritmética entera fuera de la isla lógica |
| [LOGIC-PREDICATES-INT-SURFACE-NOTE.md](LOGIC-PREDICATES-INT-SURFACE-NOTE.md) | Predicados lógicos frente a la superficie del lenguaje |
| [../ejemplos/f3/](../ejemplos/f3/) | 12 oráculos de la isla lógica (`01` a `12`) |

## Paquetes de ejemplos para IAs (few-shot)

| Documento | Contenido |
|-----------|-----------|
| [PACK-F1.1-FEWSHOT.md](PACK-F1.1-FEWSHOT.md) | Superficie mínima |
| [PACK-F2-FEWSHOT.md](PACK-F2-FEWSHOT.md) | Variables, enteros, cadenas, vectores, pruebas |
| [PACK-F2.2-FEWSHOT.md](PACK-F2.2-FEWSHOT.md) | `match` |
| [PACK-F3-FEWSHOT.md](PACK-F3-FEWSHOT.md) | Isla lógica |
| [PACK-ASYNC-FEWSHOT.md](PACK-ASYNC-FEWSHOT.md) | `async fn` / `await` |
| [PACK-STD-METHOD-SURFACE-F2.md](PACK-STD-METHOD-SURFACE-F2.md) | Métodos de la biblioteca estándar |

## Guías

- [guides/aritmetica-int-v0.md](guides/aritmetica-int-v0.md) — aritmética de enteros
- [guides/option-result-helpers-v0.md](guides/option-result-helpers-v0.md) — ayudantes de `Option` y `Result`
- [guides/METHOD-CALL-PARSE.md](guides/METHOD-CALL-PARSE.md) — cómo se analizan las llamadas a métodos

## Decisiones de diseño

La carpeta [ADR/](ADR/) contiene los registros de decisiones de arquitectura (ADR), uno por decisión. Algunos puntos de partida:

- [ADR/001-emit-rust-mvp.md](ADR/001-emit-rust-mvp.md) — por qué se emite Rust
- [ADR/002-isla-logica-propia.md](ADR/002-isla-logica-propia.md) — motor lógico propio
- [ADR/022-safe-only.md](ADR/022-safe-only.md) — ARITA es solo Rust seguro
- [ADR/225-semantics-v0.1.md](ADR/225-semantics-v0.1.md) — mapa normativo de la semántica

## Ejemplos

Los programas de [`../ejemplos/`](../ejemplos/) son oráculos de extremo a extremo: se compilan, se ejecutan y se compara la salida. Índice en [`../ejemplos/README.md`](../ejemplos/README.md). `arita measure` ejecuta todos los oráculos más `clippy`; una prueba omitida nunca cuenta como superada.
