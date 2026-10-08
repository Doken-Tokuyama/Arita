Español | [English](CI.md)

# ARITA — CI

Puerta local-first. Sin claves de demo. **skip ≠ PASS.**

## Ejecutar en local

Desde la raíz del repositorio:

```bash
bash scripts/ci.sh
# or: ./scripts/ci.sh
```

### macOS (aarch64)

```bash
cd <repository-root>
bash scripts/ci.sh
```

Hace falta Rust estable con `clippy`. Prefiere `rustup component add rustfmt clippy` para que el formato se compruebe (no se omita).

### Linux (CI)

```bash
cd <repository-root>
bash scripts/ci.sh
```

GitHub Actions (cuando exista un remoto): `.github/workflows/ci.yml` ejecuta el mismo script en `ubuntu-latest` y `macos-latest` con Rust 1.97.1 fijado (por defecto; clippy + rustfmt) más nightly-2026-09-13 (miri + rust-src; comprobado contra el commit 809936eac); en Linux también añade el target aarch64-unknown-linux-gnu y el linker cruzado.

## Qué hace el script

1. **fmt** — `cargo fmt --check` si `rustfmt` funciona; si no, **WARN** y continúa (estilo inconclusive; **no** afirma que fmt haya pasado).
2. **clippy** — crates del workspace con `-D warnings` (`arita-syntax`, `arita-codegen`, `arita-hir`, `arita-logic`, `arita-cli`).
3. **test** — `cargo test --workspace -- --test-threads=1`.
4. **measure** — `cargo run -q -p arita-cli -- measure`.

## Barra measure / códigos de salida

| Exit | Meaning |
|------|---------|
| 0 | `accepted` |
| 1 | `rejected` |
| 2 | `inconclusive` / usage |

El `accepted` global sigue las reglas de producto (p. ej. clippy en el host). Herramientas ausentes → skip o inconclusive; **nunca** inventar PASS.


## Current local bar

- **Estado (local):** **verified** (fmt+clippy+test+measure exit 0; ROADMAP #57)
- **CUT:** `CI-LOCAL-VERIFY-20260915` (infra; after `TARGETS-CROSS-LEX-20260915`)
- **Measure:** **889/889 accepted** (0 skip) when the host has rustup targets + linkers (see `DOC/05-COMPILATION-TARGETS.md` / `.cargo/config.toml`)
- Without cross linkers: overall still `accepted` with `target-*` **inconclusive** gated (never fake PASS)
- En la CI pública de GitHub, un `target-*` inconclusive gated se acepta por diseño (ADR-034): no tumba el total y nunca cuenta como accepted.
- Remote GitHub matrix: **PARK** until remote exists

## Remote status

El YAML del workflow está presente; la matriz en un remoto real sigue pendiente hasta conectar GitHub.

## Addendum — CI-LOCAL-VERIFY (**verified**)

- **CUT:** `CI-LOCAL-VERIFY-20260915` (infra P2; after `TARGETS-CROSS-LEX-20260915`)
- **IN:** `scripts/ci.sh` (fmt WARN-ok / clippy `-D warnings` / test / measure); `DOC/CI.md`; `.github/workflows/ci.yml` present
- **Local:** exit 0 + measure **89/89** accepted (**0 gated**); ROADMAP #57 closed
- **OUT / PARK:** GitHub remote matrix; fake PASS; skip≠PASS
- **Next:** ROADMAP #58 standby (trap crates PARK; CI on GitHub; ADR-036/ports PARK)
