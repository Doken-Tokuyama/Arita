[English](ROADMAP.md) | Español

# ARITA — Hoja de ruta

ARITA es un lenguaje de propósito general pensado para que lo escriban las IAs y que compila a Rust seguro (sin `unsafe`). Esta hoja de ruta resume qué está hecho y hacia dónde va el proyecto. Es un documento de producto: el detalle de diseño está en [`DOC/`](DOC/README.md).

## Estado actual

- **ARITA v1: publicado el 08-10-2026.**
- **Core 0.9: CERRADO — 834/834 oráculos superados.**
- **Core 0.10 (ERRORES, fase 1):** están cerradas once etapas de la fase 1: eliminación de resultados de error descartados en rutas muertas, seguridad de las operaciones de tareas (`spawn`/`join`), comprobación de que `arita build` solo compila ficheros que pertenecen al paquete, limpieza de avisos de Clippy en el Rust emitido, eliminación de paréntesis innecesarios en el Rust emitido, asignación compuesta por índice sobre vectores de enteros (`v[i] += x`, `-=`, `*=`; un índice fuera de rango o un desbordamiento devuelven un error en lugar de provocar un pánico), aritmética entera con desbordamiento detectado en tiempo de ejecución (`+`, `-`, `*` sobre enteros no literales terminan con un fallo controlado, tanto en builds de depuración como de release, en lugar de envolver el valor en silencio), diagnóstico de llamadas a funciones no declaradas (una llamada a una función que no es del lenguaje, ni está declarada en el módulo, ni se importa con `use` se rechaza con un error propio, E0347, antes de generar Rust), alcance de los enteros conocidos (los valores enteros que el compilador conoce de antemano ya no se arrastran a través de ramas, bucles y bloques, de modo que programas válidos dejan de recibir falsos errores de desbordamiento o de división por cero), rechazo, con diagnóstico propio (E0346), de la concurrencia con Mutex en esta versión (el Mutex real está previsto para la versión 0.1.2) y resultados descartados (un `Result` ignorado, ya sea una llamada suelta, `let _` o una variable que no se usa, es un error de compilación, E0272). Con esto están cerradas todas las etapas previstas para la versión 1.
<!-- BARRA-CORE-0.10 --> **Core 0.10 — barra actual 889/889 (cerradas todas las etapas previstas para la versión 1).**

Un oráculo es una prueba reproducible (compilar, ejecutar y comparar el resultado esperado, más `clippy`). Una prueba omitida nunca cuenta como superada.

## Siguiente: versión 0.1.2

La versión 0.1.2 es la próxima release publicable; no hay una versión 1.1 separada. Plan fijado por el autor del proyecto el 09-10-2026. Orden de trabajo:

1. **Fase 1 (aplicada y en verificación):** B-286-6b, B-296-2, B-297-1, B-297-3 y B-286-6. B-286-6 entra por el `await` suelto: un `await h()` cuyo `Result` se descarta se rechaza con E0272. Estas limitaciones se cierran si pasa el oráculo 7 de esta fase (el del `await` suelto), y eso se registra al congelar el diseño. La fase cubre el tipo real de `await` y su uso como valor, los parámetros `Int` y `Bool` en las fns `Io` que no son `main`, las palabras clave de Rust en posición de expresión (E0007) y los nombres sin ligar (E0354), y la comprobación de tipo del inicializador literal o de constructor de un `let` con tipo escalar.
2. **Fase 2:** B-286-3, B-286-4 y B-286-6a, más el `Err` de `if let` y `while let` y el `if let Ok` que ignora el `Err` (sin `else` o con un `else` que lo ignora). El diseño ampliado de esta fase citará B-286-6 como cerrado por la fase 1 y solo añadirá lo que falte (por ejemplo, el aviso de B-286-6a).
3. **Fase 3:** todas las demás limitaciones conocidas y errores abiertos de la versión 1, incluidos B-297-5 a B-297-8 y B-286-8. Lo que resulte imposible se decidirá caso por caso.
4. **Un Mutex real:** de propuesta a un diseño implementable.
5. **Superficie async ampliada:** de propuesta a un diseño implementable.
6. **Corrección guiada por el compilador:** diagnósticos estructurados, sugerencias deterministas y `arita fix`. `arita check --json` emite los diagnósticos en un formato versionado (`arita.diagnostics.v1`); sugerencias deterministas por código de error, empezando por E0004; y un bucle de reparación con `arita fix`, que es determinista e idempotente y solo escribe cambios con un `--write` explícito. Los pasos de reparación no deterministas o de alto riesgo pasan a la versión 0.1.3. Documento de diseño: pendiente.

Sobre los fallos de arriba:

- B-286-3, B-286-4, B-286-6, B-286-6a, B-286-6b y B-297-1 se describen en las [limitaciones conocidas de v1 del README](README.es.md#limitaciones-conocidas-de-v1).
- B-296-2: las fns `Io` que no son `main` todavía no admiten parámetros (el parser los rechaza con E0006).
- B-297-3: `true`, `false`, `None` o `"hola"` en un `let` tipado `Int` pasan la comprobación de ARITA y fallan en rustc con E0308.

Bajo la fase 1:

- B-297-2: la cola E0342 debe comparar el tipo esperado y el encontrado; se cierra con el trabajo de la superficie async.

**La fase 3 también incluye** (limitaciones conocidas y errores abiertos más allá de los ids ya nombrados arriba):

- Patrones `Err(_)` / `Err(_eN)` más estrictos sin `let` (B-286-1).
- Diagnóstico para una expresión pura cuyo valor no se usa (B-286-9).
- Ampliar la lista fija de emisión prohibida (B-295-1).
- Toda palabra clave estricta o reservada de Rust usada como binding o parámetro se rechaza con E0007, en cualquier edición (B-286-8, ampliado).
- Nombres sin ligar tipados como `Int` en más posiciones (destino de asignación, receptor, asignación por índice) (B-293-1); funciones importadas tipadas mal como `Int` (B-293-2).
- Exención de lista cerrada para rutas `::`, bajo un diseño propio de métodos asociados (B-293-3).
- Spans más precisos en llamadas (B-286-11) y en rutas (B-297-5).
- `MIN / -1` debe reportar E0217, no E0100 (B-292-1).
- Un `let` interior no debe heredar en silencio tipo, mutabilidad ni estado moved de un binding exterior (B-294-1); E0217 sensible al flujo (B-294-2).
- Aritmética checked opcional con `?` (B-292-4).
- Asignación compuesta sobre un identificador entero (`x += 1`) (B-290-1).
- Resolución de paquetes: canonicalizar la búsqueda de `arita.toml` (B-287-1); respetar `--target` en la ruta del workspace (B-287-2); manifiestos anidados autónomos — gana el más cercano, con diagnóstico explícito (B-287-3).
- Tipado general de inicializadores de `let` más allá del corte de literales/constructores escalares (B-297-3b).
- Semántica del nombre de una fn usado como valor fuera de `serve` (B-297-7).
- Comprobación de aridad y tipo en llamadas a fns de usuario (B-297-8); defensa para que `await` como sentencia sobre una fn que devuelve `Result` no se cuele en codegen como E0006 (B-297-6).
- Pánico de tarea tragado por `join` (B-296-1).
- Fixture de contract / camino `?` que hoy rechaza un programa válido (B-286-2).
- Más lints de rustc/Clippy en oráculos positivos (B-289-1); oráculos para workspaces suplementarios (B-289-2).
- Harness de runners negativos, clases B y C (B-297-4).
- Ampliar E0215 a comparaciones tautológicas como `x == x` (teatro visible al usuario).
- Paquete: una fn pública no debe colapsar a un helper privado; fn inexistente sigue siendo E0347.
- `arita parse` sobre un binario válido de workspace debe terminar con exit 0.
- Edges documentados que fijan el stdout de ambos caminos `Err(0)`; aviso opcional en build si hay scenarios declarados sin ejecutar.

Bajo los puntos de Mutex y async:

- El diseño del Mutex cubre guards RAII, la política de `thread_local`, parámetros `Mutex<T>` y argumentos de `spawn` (hoy rechazados con E0346 donde aplica).
- Incluye `pub async fn` en bibliotecas y `async main` en binarios de paquete.

Bajo la corrección guiada por el compilador:

- También en 0.1.2: la protección mínima de escritura de `arita fix --write` (hash de contenido, re-comprobación en memoria, guarda de carrera).

**Criterio de release:** en cada etapa, todos los oráculos deben pasar sin pruebas omitidas, y la evidencia debe superar una verificación independiente. La versión 0.1.2 sale solo cuando todas las etapas estén cerradas.

## Versión 0.1.3

### Lenguaje

- **propuesto:** Endurecer el ban de unwrap en paths host si measure detecta un hueco.

### Ownership y concurrencia

- **propuesto:** Guía documentada sobre el coste de clone / ownership simple (hasta haber oráculo estable).

### Tooling

- **propuesto:** `arita build --diagnostics json` (diagnósticos de rustc en JSON).
- **propuesto:** La CI no debe cachear `target/` si Miri guarda ahí datos de entorno.

### Repair y distribución

- **propuesto:** Repair paso 2 (resto): almacén de revisiones y sandbox.
- **propuesto:** Controller de repair con presupuesto y detección de estancamiento.
- **propuesto:** Biblioteca manual de unos 20–30 templates de repair.
- **propuesto:** Memoria de repair ordenada por evidencia medida.
- **propuesto:** LLM solo para decisiones semánticas no mecánicas en el bucle de repair.
- **propuesto:** Promover repairs golden a reglas, tests y fixes deterministas.
- **propuesto:** Finetuning del modelo solo con miles de pares golden (versión 0.1.3 o después).
- **propuesto:** Los mutantes de swallow en repair nunca cuentan como aceptados (revisión humana).
- **propuesto:** Rechazar el 100 % de los hunks que toquen rutas protegidas.
- **propuesto:** Registro de falsos positivos por código con semilla fija.
- **diseño en curso:** Diseño ampliado del oráculo de repair (pasos de controller y memoria más allá del corte 0.1.2).

## Versión 0.2

### Lenguaje

- **propuesto:** Bucles `for` sobre colecciones (hace falta sintaxis nueva; hoy `for` se rechaza como sentencia ilegal con E0006).
- **soporte parcial existente, ampliación:** Isla lógica (inspirada en Tau/TML, motor propio).
- **propuesto:** Backend WASM.
- **propuesto:** Documento de semántica del lenguaje v0.1 (prosa normativa de producto).
- **propuesto:** Contratos / verificación formal opcional proporcional al riesgo.
- **propuesto:** Capa IR / AIR / CFG en el pipeline de compilación.

### Stdlib e IO

- **propuesto:** APIs de la biblioteca estándar orientadas a hot paths medibles (sigue sin `unsafe`).

### Tooling

- **propuesto:** UI del manifiesto de evidencia.

## Sin programar (en espera)

### Lenguaje

- **en espera:** Escritura por índice en `String` (`s[i] = x`, hoy E0314).
- **en espera:** Casos residuales de asignación por índice (E0006 / E0314) y separar E0314 (asignación por índice vs campo desconocido).
- **en espera:** `?` sobre `Option` y `?` en `main` con `Io` (fase posterior).
- **en espera:** `unwrap` / `expect` en la superficie del lenguaje (ban de emisión).
- **en espera:** Modelo de software verificado AI-native (nivel RFC), en espera hasta un visto bueno explícito del autor del proyecto.

### Ownership y concurrencia

- **en espera:** Ownership y aliasing más profundos (loans del receptor, parámetros `&mut` de colecciones, vertical relacionada).
- **en espera:** Runtime multihilo (hilos de sistema junto a las tareas).
- **en espera:** Lecturas y operaciones más ricas dentro de un lock (get por índice, `get`/`first`/`last`/`pop`, métodos `Result`/`Option`, asignación indexada sobre `Vec`/`List`).
- **en espera:** Compartir un Mutex por retorno, records, colecciones o `Result`/`Option` (hoy E0346); `Arc` explícito.
- **en espera:** Forma `?` del bloque lock; ampliar la whitelist del bloque (`match` / `if let` / `while let` / helpers puros).
- **en espera:** RwLock / Condvar; Mutex async; `try_lock` / lock con timeout; rangos de orden de lock declarados; atómicos de superficie; guard como valor; locks anidados; lint de sección crítica partida; `Mutex<Bytes>` / `Mutex<record>`.

### Async

- **en espera:** Parámetros no `Copy` / `Text` en fns `Io` que no son `main` y en `async fn`.

### Stdlib e IO

- **en espera:** Nuevos bindings de E/S.

### Red

- **en espera:** TLS y WebSocket.
- **en espera:** Sockets en bruto.
- **en espera:** Cierre por inactividad de servicios (enforce idle).

### Tooling

- **en espera:** Dependencias y publicación abiertas en crates.io (salvo `cargo publish --dry-run`).
- **en espera:** Servidor MCP / conector ACP para tooling de editor y agentes.
- **en espera:** LSP/IDE más completo (completion, hover, rename, ir a definición, formateo, code actions, pull diagnostics).

### Repair y distribución

- **en espera:** Packaging como distribución (binarios precompilados, instaladores).
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
| Core 0.10 | Errores, fase 1 (sumideros muertos, seguridad de `spawn`/`join`, pertenencia al paquete, Rust emitido sin avisos de Clippy ni paréntesis innecesarios, asignación compuesta por índice, aritmética entera con desbordamiento detectado, llamadas a funciones no declaradas, alcance de enteros conocidos, rechazo de Mutex en la versión 1 y resultados descartados) | Once etapas cerradas (todas las previstas para la versión 1) |

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
- Integración con editores y herramientas de desarrollo.

## Fuera del alcance actual

Hilos de sistema, dependencias abiertas de crates.io, TLS/WebSocket, cierre por inactividad de servicios y *bindings* de E/S adicionales no forman parte del lenguaje por ahora. Los hilos de sistema siguen fuera del alcance actual; la exclusión mutua entre tareas (un Mutex real) está prevista para la versión 0.1.2 (de propuesta a un diseño implementable).

## Cómo verificarlo

Desde la raíz del repositorio: `cargo test --workspace -- --test-threads=1` y `cargo run -p arita-cli -- measure`. Más detalle en [`BUILD.md`](BUILD.md) y [`DOC/CI.md`](DOC/CI.es.md).
