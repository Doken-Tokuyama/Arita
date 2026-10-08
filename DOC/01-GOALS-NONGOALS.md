# Objetivos y no-objetivos

## Objetivos

1. **AI-native surface:** sintaxis predecible, keywords ASCII, sin macros ilimitadas al inicio.
2. **Safe-only:** sin `unsafe` en el dialecto (ADR-022). No hay fase posterior de unsafe en el producto medido.
3. **Cross-compile:** Linux, Windows, macOS × x86_64, aarch64.
4. **Logic island:** bloques `spec` / `query` con semántica inspirada en TML (hechos + reglas).
5. **Measurable:** `arita measure` / tests que no acepten theater (asserts vacíos, stubs).

## No-objetivos (primeras fases)

- Compatibilidad source-level con Rust.
- Compatibilidad source-level con `.tau` / TML crudo.
- GC general como runtime principal (preferir ownership).
- IDE completa el día 1 (sí: CLI + LSP mínimo en fase media).
