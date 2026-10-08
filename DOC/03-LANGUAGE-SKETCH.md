# Boceto de lenguaje (no final)

## Principios de diseño para IAs

1. Una forma canónica de cada constructo (pocas alternativas).
2. Efectos explícitos (`io`, `pure`).
3. Nada de `todo!` / `unimplemented!` en dialecto base (keywords prohibidas o hard-error).
4. Tests: asserts con evidencia (prohibido `assert true`).

## Ejemplo ilustrativo (borrador)

```arita
module hello

fn add(a: Int, b: Int) -> Int = a + b

spec AddComm {
  fact add(a, b) = add(b, a)   // semántica lógica / propiedad
}

fn main() -> Io<()> {
  print(add(2, 3))
}

test add_smoke {
  assert add(2, 3) == 5
}
```

## Relación con Tau/TML

- `spec` / `fact` / `rule` → lower a IR compatible con motor TML embebido **o** verificación offline.
- El código `fn` ejecutable no es Tau; es systems-lang restringido.
- La IA puede generar **primero** el `spec` y **después** el `fn` (workflow Camino A).

## Relación con Rust

- Codegen: Rust 2024 edition, `#![forbid(unsafe_code)]` en todo emit de usuario (ADR-022 safe-only; sin módulos unsafe).
- Ownership: subset (sin lifetimes explícitos en v0 si el codegen los infiere).
