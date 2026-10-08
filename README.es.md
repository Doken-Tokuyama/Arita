[English](README.md) | Español

# ARITA

**ARITA** es un lenguaje de programación de propósito general **pensado para que lo escriban las IAs** y que **compila a Rust seguro**. La IA escribe ficheros `.arita`; el compilador los traduce, de forma determinista, a proyectos Rust reales (código *safe*, sin `unsafe`) que se compilan con `cargo` y funcionan sin depender de la IA que los escribió.

## Qué es (y qué no es)

- **Es** un lenguaje nuevo, con una semántica pequeña, total y canónica: pocas formas equivalentes de escribir lo mismo, semántica local, errores estructurados y reparables, y nada de "compila pero sorprende".
- **Es** un compilador que emite Rust real: `.arita` → análisis → representación intermedia tipada → emisión determinista → Rust → `cargo`.
- **No es** un dialecto de Rust ni un envoltorio de `cargo` donde "la IA edita Rust".
- **No es** una plataforma de herramientas para IAs ni un lenguaje de contratos de verificación formal.

## Principio de diseño: nada de teatro

Una pieza de código solo se considera aceptada cuando **oráculos declarados y reproducibles** lo demuestran sobre el artefacto exacto: compilar, ejecutar y comparar la salida esperada, además de pasar `clippy`. Compilar, tener tests verdes o producir una demo no basta. Una prueba omitida nunca cuenta como superada.

## Estado

- **ARITA v1 publicado el 08-10-2026.**
- **Core 0.9 CERRADO 834/834.** Escritura en colecciones por índice (`m[k] = v`, `v[i] = x`) sin pánico.
- **Core 0.10** (ERRORES, fase 1). <!-- BARRA-CORE-0.10 -->Barra actual: 889/889 (cerradas todas las etapas previstas para la versión 1). Ver [`ROADMAP.md`](ROADMAP.es.md).
- Los binarios precompilados llegarán con la release v0.1.1.

Las versiones anteriores del núcleo (0.1 a 0.8) están cerradas; la escalera completa está en la [hoja de ruta](ROADMAP.es.md).

## Limitaciones conocidas de v1

Huecos de diagnóstico conocidos de esta versión:

- **B-286-3** (P0 de v1.1): en el brazo `Err(e)` de un `match` sobre una lectura del host (`host.read_text`), pasar el payload del error a una variable que no se usa (`let _c: Int = e`) no produce diagnóstico y el programa se acepta. La misma forma sobre una función propia que devuelve `Result` sí da E0272.
- **B-286-4** (P0 de v1.1): ese mismo binding muerto del payload `Err` dentro de `if let Err(e) = …` o de `while let Err(e) = …` tampoco se diagnostica.
- **B-286-6** (P0 de v1.1): un `await h()` suelto (en el Rust generado, `h().await;`), cuando `h` devuelve `Result`, descarta el resultado sin diagnóstico.
- **B-286-6a** (P0 de v1.1): `arita build` no muestra los avisos de rustc sobre el Rust generado; por ejemplo, el `unused_must_use` del caso anterior.
- **B-286-6b** (P0 de v1.1): el `Result` de un `async fn` propio todavía no se puede consumir: `let r = await h()` da E0203 y `match await h()` no se analiza.
- **B-297-1** (P1 de v1.1): una palabra clave usada como valor (p. ej. `let x: Int = return`) no se diagnostica: el programa compila y termina antes de tiempo. En otros casos de identificadores no ligados el error lo da rustc en vez de ARITA.

En B-286-3, B-286-4 y B-286-6 el programa se acepta y el Rust generado es seguro y hace lo que dice el código, pero el error se pierde sin aviso.

### Pautas para escribir ARITA v1 (IA o humano)

- En v1, un `async fn` no debe devolver `Result`: su error no se puede consumir y se pierde en silencio. Maneja el error dentro de la fn.
- v1 no diagnostica el descarte del `Err` en `if let` / `while let` (tampoco en un `if let Ok(..)` cuyo `else` ignora el Err) ni en `match` sobre `host.read_text`. En `match` sobre fns propias sí salta E0272.

## Cómo compilar y probar

Requisitos: toolchain estable de Rust (`cargo`). Para el veredicto completo de aceptación hace falta además `clippy`.

Desde la raíz del repositorio:

```bash
cargo test --workspace -- --test-threads=1
cargo build -p arita-cli
./target/debug/arita build ejemplos/01-hello.arita
```

`arita build` imprime una línea `ok: target/arita-out/hello_<hash>` (el nombre lleva un sufijo con un hash). Ejecuta la ruta que imprime `ok:`, por ejemplo `./target/arita-out/hello_*`, y verás `hello`.

Ejemplo mínimo (`ejemplos/01-hello.arita`):

```arita
module hello

fn main() -> Io<()> {
  print("hello")
}
```

La isla lógica (módulos con `fact` / `rule` / `query`, evaluados con un motor Datalog propio, no con `rustc`) se ejecuta así:

```bash
./target/debug/arita logic ejemplos/f3/01-path-ok.arita   # imprime: true (código de salida 0)
```

Todos los subcomandos de la CLI y la puerta de integración continua local (`bash scripts/ci.sh`) están descritos en [`BUILD.md`](BUILD.md). La guía de verificación para IAs está en [`DOC/09-AI-PROGRAMMING.md`](DOC/09-AI-PROGRAMMING.md).

## Oráculos de ejemplo

Los programas de `ejemplos/` son oráculos de extremo a extremo (compilar, ejecutar y comparar la salida). Por ejemplo, la isla lógica F3 tiene **12 oráculos** (`f3-01` … `f3-12`) en [`ejemplos/f3/`](ejemplos/f3/): casos que deben cumplirse y casos negativos con códigos de error exactos (`E0301`, `E0303`, `E0304`). Índice completo en [`ejemplos/README.md`](ejemplos/README.md).

## Estructura del repositorio

| Ruta | Contenido |
|------|-----------|
| `crates/` | Workspace de Rust: sintaxis, representación intermedia tipada (HIR), generación de código, motor de la isla lógica, CLI (`arita-cli`) y crates de apoyo para ejemplos con host |
| `ejemplos/` | Programas `.arita` que sirven de oráculos, organizados por fase y por versión del núcleo (`f2`, `f2.2`, `f3`, `async`, `core01` … `core10`, etc.) |
| `DOC/` | Documentación: visión, arquitectura, semántica, modelo de amenazas, guías, paquetes de ejemplos para IAs y decisiones de diseño (`DOC/ADR/`) |
| `scripts/` | Puerta de integración continua local |
| `ROADMAP.md` | Hoja de ruta del producto |
| `BUILD.md` | Instrucciones de compilación y uso de la CLI |

## Documentación recomendada

- Visión corta: [`DOC/00-VISION.md`](DOC/00-VISION.md) y [`DOC/PRODUCT-VISION.md`](DOC/PRODUCT-VISION.md)
- Tesis y modelo: [`DOC/RFC-AINATIVE-VERIFIED-MODEL.md`](DOC/RFC-AINATIVE-VERIFIED-MODEL.md)
- Semántica y pipeline: [`DOC/SEMANTICS-V0.1.md`](DOC/SEMANTICS-V0.1.md)
- Cómo programar en ARITA con una IA: [`DOC/09-AI-PROGRAMMING.md`](DOC/09-AI-PROGRAMMING.md)
- Modelo de amenazas y evidencia: [`DOC/THREAT_MODEL.md`](DOC/THREAT_MODEL.md)
- Índice completo: [`DOC/README.md`](DOC/README.md)

## Seguridad

Para informar de una vulnerabilidad, consulta [SECURITY.es.md](SECURITY.es.md).

## Licencia

Licencia: Apache-2.0, ver [`LICENSE`](LICENSE).

## Autor y contacto

Autor: Antonio Cantallops Alba

Contacto general: **contact@exyonq.org**
