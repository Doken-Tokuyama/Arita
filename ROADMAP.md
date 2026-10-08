# ARITA — Hoja de ruta

ARITA es un lenguaje de propósito general pensado para que lo escriban las IAs y que compila a Rust seguro (sin `unsafe`). Esta hoja de ruta resume qué está hecho y hacia dónde va el proyecto. Es un documento de producto: el detalle de diseño está en [`DOC/`](DOC/README.md).

## Estado actual

- **Core 0.9: CERRADO — 834/834 oráculos superados.**
- **Core 0.10 (ERRORES, fase 1): en curso.** Ya están cerradas once etapas de la fase 1: eliminación de resultados de error descartados en rutas muertas, seguridad de las operaciones de tareas (`spawn`/`join`), comprobación de que `arita build` solo compila ficheros que pertenecen al paquete, limpieza de avisos de Clippy en el Rust emitido, eliminación de paréntesis innecesarios en el Rust emitido, asignación compuesta por índice sobre vectores de enteros (`v[i] += x`, `-=`, `*=`; un índice fuera de rango o un desbordamiento devuelven un error en lugar de provocar un pánico), aritmética entera con desbordamiento detectado en tiempo de ejecución (`+`, `-`, `*` sobre enteros no literales terminan con un fallo controlado, tanto en builds de depuración como de release, en lugar de envolver el valor en silencio), diagnóstico de llamadas a funciones no declaradas (una llamada a una función que no es del lenguaje, ni está declarada en el módulo, ni se importa con `use` se rechaza con un error propio, E0347, antes de generar Rust), alcance de los enteros conocidos (los valores enteros que el compilador conoce de antemano ya no se arrastran a través de ramas, bucles y bloques, de modo que programas válidos dejan de recibir falsos errores de desbordamiento o de división por cero), rechazo, con diagnóstico propio (E0346), de la concurrencia con Mutex en esta versión (el Mutex real llegará en la 1.1) y resultados descartados (un `Result` ignorado, ya sea una llamada suelta, `let _` o una variable que no se usa, es un error de compilación, E0272). Con esto están cerradas todas las etapas previstas para la versión 1. Lo siguiente: las comprobaciones previas a la versión 1 y, después, la declaración de la versión 1.
<!-- BARRA-CORE-0.10 --> **Core 0.10 en curso — barra actual 889/889 (cerradas todas las etapas previstas para la versión 1).**

Un oráculo es una prueba reproducible (compilar, ejecutar y comparar el resultado esperado, más `clippy`). Una prueba omitida nunca cuenta como superada.

## Escalera de versiones del núcleo ("Core")

Cada versión del núcleo añade una capacidad vertical y se da por cerrada solo cuando todos sus oráculos pasan.

| Versión | Capacidad | Estado |
|---------|-----------|--------|
| Core 0.1 | Vertical base: registros, enumeraciones, `match`, tipos básicos, `Result`, propiedad simple, ficheros/JSON/CLI, escenarios ejecutables | Cerrado (603/603) |
| Core 0.2 | Perfil `service`: tareas, temporizadores y cancelación, enlaces HTTP, servicio de referencia | Cerrado (632/632) |
| Core 0.3 | Composición cliente + servidor, propagación de errores, pipelines secuenciales | Cerrado (658/658) |
| Core 0.4 | Paquetes y bibliotecas (manifiesto, API de biblioteca, varios módulos) | Cerrado (683/683) |
| Core 0.5 | Colecciones con operaciones falibles (`insert` → `Result`, acceso por índice como `Option`) | Cerrado (705/705) |
| Core 0.6 | Programas de propósito general de extremo a extremo (conjuntos, mapas, escenarios) | Cerrado (728/728) |
| Core 0.7 | Entrada/salida sin teatro (lectura y *parsing* con errores exactos, argumentos de CLI) | Cerrado (752/752) |
| Core 0.8 | Propagación de errores: funciones que devuelven `Result` y operador `?` | Cerrado (778/778) |
| Core 0.9 | Escritura en colecciones por índice (`m[k] = v`, `v[i] = x`) sin pánico | Cerrado (834/834) |
| Core 0.10 | Errores, fase 1 (sumideros muertos, seguridad de `spawn`/`join`, pertenencia al paquete, Rust emitido sin avisos de Clippy ni paréntesis innecesarios, asignación compuesta por índice, aritmética entera con desbordamiento detectado, llamadas a funciones no declaradas, alcance de enteros conocidos, rechazo de Mutex en la versión 1 y resultados descartados) | En curso (once etapas cerradas; siguen las comprobaciones previas a la versión 1) |

## Fases del lenguaje

| Fase | Contenido | Estado |
|------|-----------|--------|
| 0 | Documentación base y decisiones de diseño | Hecho |
| 1 | Esqueleto de la cadena de herramientas: análisis, generación de Rust, CLI, `arita measure` | Hecho |
| 2 | Subconjunto de propiedad (ownership), biblioteca estándar mínima y *std* de métodos | Hecho |
| 3 | Isla lógica (`spec` / `fact` / `rule` / `query`) con motor propio; 12 oráculos | Hecho |
| 4 | Multiplataforma (Linux, macOS, Windows; x86_64 y ARM), `async`, rendimiento y dependencias Rust acotadas | Hecho en su alcance actual |
| 5 | Auto-alojamiento (opcional) | Primeros pasos (ejemplos de arranque) |

## Principios que no cambian

1. **Seguro por construcción:** el Rust emitido no usa `unsafe`.
2. **Sin teatro:** nada se acepta por parecer correcto; solo si lo demuestran oráculos reproducibles sobre el artefacto exacto.
3. **Pocas formas de escribir lo mismo:** sintaxis canónica y errores estructurados con códigos estables (`E0xxx`).
4. **Independiente de la IA:** el proyecto Rust emitido funciona sin la IA que escribió el `.arita`.

## Más adelante (sin fecha)

- Ampliar el núcleo con nuevas capacidades verticales, siempre con oráculos antes de declararlas cerradas.
- Corrección guiada por el compilador (el compilador como oráculo estructurado de errores): hoy es una propuesta de diseño, ver [`DOC/REPAIR-ORACLE.md`](DOC/REPAIR-ORACLE.md).
- Integración con editores y herramientas de desarrollo.

## Fuera del alcance actual

Hilos y exclusión mutua, dependencias abiertas de crates.io, TLS/WebSocket, cierre por inactividad de servicios y *bindings* de E/S adicionales no forman parte del lenguaje por ahora.

## Cómo verificarlo

Desde la raíz del repositorio: `cargo test --workspace -- --test-threads=1` y `cargo run -p arita-cli -- measure`. Más detalle en [`BUILD.md`](BUILD.md) y [`DOC/CI.md`](DOC/CI.md).
