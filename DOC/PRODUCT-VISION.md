# ARITA — Visión de producto

> **ARITA** es un lenguaje de propósito general **pensado para que lo escriban las IAs** y que **compila a Rust seguro**: la IA escribe ARITA como lenguaje principal; el compilador rechaza la ambigüedad, el humo y las operaciones no verificables, y emite **proyectos Rust reales** (binario, biblioteca, servicio…) que se sostienen **sin depender de la IA**.

## Qué no es

| No es | Por qué |
|-------|---------|
| Una plataforma de herramientas para IAs | Las herramientas de edición no son el lenguaje |
| Un DSL de contratos o de verificación formal | Los contratos son opcionales y vendrán proporcionados al riesgo |
| Un envoltorio de Cargo ni "una IA que edita Rust" | El artefacto nativo es el `.arita` |
| Un transpilador de pseudocódigo libre | Sin semántica fija vuelve el código que "parece que funciona" |

## Arquitectura

`.arita` → analizador léxico/sintáctico → AST → HIR tipado → representación intermedia (IR/CFG) → comprobador → emisor determinista → Rust real → `cargo`.

## Diseño pensado para IAs

Pocas formas equivalentes de escribir lo mismo · sintaxis canónica · semántica local · APIs tipadas y versionadas · errores estructurados y reparables · nada de "compila pero sorprende" · proyecto visible · un camino preferido por problema.

ARITA **no** copia Rust al pie de la letra: no hay *lifetimes* explícitos, rasgos (*traits*), macros, `unwrap`, indexación que entre en pánico, `async` implícito, dependencias abiertas ni `unsafe`.

## Decisión

Un lenguaje **nuevo**, parecido a Rust pero con una semántica más pequeña, total y canónica, y con Rust seguro como *backend*. No es un dialecto de Rust.

## Anti-teatro

análisis → resolución → tipos → IR → emisión → `cargo` → pruebas de aceptación → políticas → evidencia.
Los `scenario` / `acceptance` son especificaciones **ejecutables**. Una prueba omitida nunca cuenta como superada.

## Evidencia

Manifiesto JSON por hash de fuente. Niveles de confianza desde "analizado" hasta "demostrado", más "bloqueado" y "desconocido". Prohibido declarar algo hecho sin evidencia.

## Estado

- **Core 0.9: CERRADO — 834/834.** Escritura en colecciones por índice (`m[k] = v`, `v[i] = x`) sin pánico.
- **Core 0.10 (ERRORES, fase 1): en curso.** Once etapas de la fase 1 están cerradas, todas las previstas para la versión 1; siguen las comprobaciones previas a la versión 1 y la declaración de la versión 1. La barra vigente está en la [hoja de ruta](../ROADMAP.md).
- Núcleo 0.1 a 0.8: cerrados. Ver la escalera completa en [`../ROADMAP.md`](../ROADMAP.md).

## Corrección guiada por el compilador

La idea: el compilador como **oráculo estructurado** para corrección autónoma acotada, no un mensaje de error pegado a un modelo. Hoy es una propuesta de diseño: [`REPAIR-ORACLE.md`](REPAIR-ORACLE.md).

## Más información

[`RFC-AINATIVE-VERIFIED-MODEL.md`](RFC-AINATIVE-VERIFIED-MODEL.md) · [`SEMANTICS-V0.1.md`](SEMANTICS-V0.1.md) · [`00-VISION.md`](00-VISION.md) · [`THREAT_MODEL.md`](THREAT_MODEL.md)
