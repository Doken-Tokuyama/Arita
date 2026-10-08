# ARITA — Threat model (anti-theater / fake evidence)

- **CUT-ID:** `EVIDENCE-CHAIN-20260913`
- **Estado:** **aceptada** (GO Ingeniero Rust, CUT `EVIDENCE-CHAIN-20260913`)
- **Fecha:** 2026-09-13
- **Fuente:** brief de amigo de <person> (visión “cadena de evidencia”); nombre del lenguaje = **ARITA** (no Veyra)
- **Relacionados:** `DOC/ADR/008-evidence-architecture.md`, ADR-002, ADR-005, ADR-006, barra E2E en `ROADMAP.md`

Producto (rev. 3): lenguaje AI-native → Rust real — ver `RFC-AINATIVE-VERIFIED-MODEL.md`. Evidence JSON/manifest primero; UI después.
Repair: compilador como oráculo estructurado (`REPAIR-ORACLE.md` / ADR-231) — no fake success; **HOLD IMPL**.

## Principio rector

No existe “correcto” por parecerlo, compilar, tener tests verdes o producir una demo. Solo existe **`accepted`** cuando los oráculos declarados, reproducibles y anti-bypass lo demuestran sobre el **artefacto exacto**.

El enemigo no es solo un bug: es la salida de una IA que optimiza para **aparentar** haber cumplido.

## Estados de veredicto (obligatorios)

```text
rejected     = hay contradicción, falta evidencia o hay bypass
inconclusive = no se pudo obtener evidencia suficiente
accepted     = todos los oráculos requeridos aprobaron el artefacto exacto
```

**Nunca** convertir `inconclusive` → `accepted`.  
`skip ≠ PASS`. No inventar PASS. `arita measure` usa estos tres estados (o equivalentes machine-readable); un skip/timeout/ harness roto = `inconclusive` o `rejected`, nunca `accepted`.

## Modos de fake/theater que se deben bloquear

| Patrón | Ejemplo | Defensa en el lenguaje/plataforma |
|---|---|---|
| Stub disimulado | `return Ok(default())` en una rama no cubierta | Obligaciones de cobertura semántica y pruebas de mutación |
| Mock sustituyendo sistema real | Test que simula almacenamiento/red en vez de usar el backend requerido | Oráculo de integración obligatorio y prohibición explícita de mocks en perfiles de aceptación |
| Test tautológico | El test replica la misma lógica defectuosa de la implementación | Oráculo independiente, propiedades metamórficas, implementación de referencia o modelo |
| Fixtures memorizados | Detecta entradas conocidas y devuelve respuestas esperadas | Generación secreta/determinista por semilla, corpus oculto y metamorphic testing |
| Resultado falsificado | CLI imprime “passed” sin ejecutar | Ejecutor aislado, logs estructurados, exit status, trazas y hash de binario |
| Código muerto | Implementa una función correcta pero el camino de producción usa otra | Instrumentación de cobertura de rutas y verificación de símbolo/artefacto ejecutado |
| Bypass de garantías / policy | `unsafe`, FFI, shell, red o lectura de ficheros para saltarse reglas | Sistema de capacidades, efectos explícitos y sandbox |
| Bench de teatro | Mide una ruta trivial, caché precargada o datos irreales | Dataset y configuración declarados, warmup separado, huella de entorno y auditoría de métricas |
| “Proof by assertion” | `assert!(true)`, `unwrap`, `todo!`, `unimplemented!` | Linter como error, IR de obligaciones no satisfecha, denylist extensible |
| Código generado no ejecutado | Se valida un artefacto distinto al entregado | Content-addressing y attestation que une fuente, IR, binario, entorno y resultado |

## Mapeo v0 ARITA (ya en marcha)

No borra F1/F2/medida: anti-fake del **lenguaje** (scenarios ejecutables + evidencia). Contratos formales = opcionales/proporcionales (RFC rev. 3), no el producto.

| Amenaza (tabla) | Defensa v0 hoy | Horizonte |
|-----------------|----------------|-----------|
| Proof by assertion / stubs | ADR-005/006 anti-theater (`E021x`); denylist `todo!`/`assert true`; **`arita measure` v0** oráculo **clippy-workspace** = `cargo clippy -p arita-syntax -p arita-codegen -p arita-cli -- -D warnings` (**requerido** para overall `accepted`; sin clippy en host → `inconclusive`) | IR de obligaciones |
| Resultado falsificado | `arita measure` / `arita build` + run binario real; skip ≠ PASS; JSON machine-readable | hashes / attestation |
| Código generado no ejecutado | measure/build sobre emit+`rustc` del mismo pipeline (v0 verificado) | content-addressing |
| Bypass de contrato | sin `unsafe` en dialecto base; efectos `Io` explícitos (F1.1/F2) | capacidades / sandbox |
| Resto de filas | parcialmente cubierto o pendiente | isla lógica + oráculos (F3+) |


## Verificación measure v0

- **Estado:** `arita measure` v0 **verificado** (Ingeniero Rust, 2026-09-13).
- Oráculos: `ejemplos/` 01–05 + `f2/01-let-int` (build+run+stdout) **y** `clippy-workspace`.
- **Pin:** overall `accepted` **needs clippy on host**. Clippy ausente/no runnable → `inconclusive` (nunca promovido a `accepted`).
- Uso: `cargo run -p arita-cli -- measure` desde la raíz del workspace.

## Checklist GO — cerrado

- [x] Tabla theater OK
- [x] Estados + regla no `inconclusive`→`accepted` OK
- [x] Mapeo v0 OK
- [x] GO `EVIDENCE-CHAIN-20260913` → aceptada + Docs índice

### Trampa vacuous len (2026-09-14)

`assert …len() >= 0` / tautologías `is_empty` → **E0214** (ADR-030; no diluir E0211).

`assert n == n` / `n <= n` / lit==lit → **E0215** (ADR-038; no diluir E0211/E0214).

`/` `%` con divisor cero (lit o call) → **E0216** (ADR-044; HIR antes de emit; no rustc/panic theater).

`+/-/*` overflow i64 (lit MAX+1 / silent wrap release) → **E0217** (ADR-045; HIR antes de emit).
