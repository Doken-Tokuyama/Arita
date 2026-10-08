[Español](03-LANGUAGE-SKETCH.md) | English

# Language sketch (not final)

## Design principles for AIs

1. One canonical form per construct (few alternatives).
2. Explicit effects (`io`, `pure`).
3. No `todo!` / `unimplemented!` in the base dialect (forbidden keywords or hard-error).
4. Tests: asserts with evidence (`assert true` forbidden).

## Illustrative example (draft)

```arita
module hello

fn add(a: Int, b: Int) -> Int = a + b

spec AddComm {
  fact add(a, b) = add(b, a)   // logical semantics / property
}

fn main() -> Io<()> {
  print(add(2, 3))
}

test add_smoke {
  assert add(2, 3) == 5
}
```

## Relation to Tau/TML

- `spec` / `fact` / `rule` → lower to IR compatible with an embedded TML engine **or** offline verification.
- Executable `fn` code is not Tau; it is a restricted systems language.
- The AI may generate the `spec` **first** and the `fn` **afterwards** (Camino A workflow).

## Relation to Rust

- Codegen: Rust 2024 edition, `#![forbid(unsafe_code)]` on all user emit (ADR-022 safe-only; no unsafe modules).
- Ownership: subset (no explicit lifetimes in v0 if codegen infers them).
