# Compilar y probar ARITA

## Requisitos

- Toolchain estable de Rust con `cargo` (y `clippy`; opcionalmente `rustfmt`).
- Para la verificación completa de oráculos (`arita measure`) hace falta `clippy` en el equipo.

## Compilar la CLI

Desde la raíz del repositorio:

```bash
cargo build -p arita-cli
```

El binario queda en `target/debug/arita`. También puedes usar `cargo run -p arita-cli -- <subcomando>`.

## Compilar un programa `.arita`

```bash
./target/debug/arita build ejemplos/01-hello.arita
```

La salida es una línea `ok: <ruta>` con la ruta del binario generado, que lleva un sufijo con un hash (por ejemplo `target/arita-out/hello_<hash>`). Ejecuta esa ruta:

```bash
./target/arita-out/hello_*      # imprime: hello
```

Perfil y destino opcionales: `arita build [--profile debug|release] [--target <triple>] <fichero.arita>`.

## Subcomandos de la CLI

```text
arita build [--profile debug|release] [--target <triple>] <fichero.arita>
arita parse <fichero.arita>
arita test <fichero.arita>
arita logic <fichero.arita>
arita contract <ruta.json|ruta.arita>
arita contract --verify-attest <ruta.json>
arita attest verify <ruta.json>
arita measure
arita lsp
arita version
```

- `arita test` ejecuta los bloques `test` de un programa (por ejemplo `ejemplos/f2/07-assert.arita`).
- `arita logic` evalúa un módulo de la isla lógica (`ejemplos/f3/01-path-ok.arita` imprime `true`).
- `arita measure` ejecuta todos los oráculos y `clippy`. Códigos de salida: `0` aceptado, `1` rechazado, `2` no concluyente.

## Probar el workspace

```bash
cargo test --workspace -- --test-threads=1
```

## Puerta completa de integración continua (local)

```bash
bash scripts/ci.sh
```

Ejecuta, por este orden: comprobación de formato (si `rustfmt` está disponible; si no, avisa y continúa), `clippy` con `-D warnings`, los tests del workspace y `arita measure`. Más detalle en [`DOC/CI.md`](DOC/CI.md).

## Más información

- Cómo programar en ARITA con una IA y cómo verificar: [`DOC/09-AI-PROGRAMMING.md`](DOC/09-AI-PROGRAMMING.md)
- Ejemplos: [`ejemplos/README.md`](ejemplos/README.md)
