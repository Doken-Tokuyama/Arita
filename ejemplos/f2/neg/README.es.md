Español | [English](README.md)

# Oráculos negativos F2 — ownership E020x + anti-theater E021x (ADR-006 / ADR-009)

Estos ficheros `.arita` **deben FALLAR** en `arita parse` / `parse()`+`check` con el código inglés estable listado.
No son programas E2E PASS. `skip ≠ PASS`.

## E020x — HIR ownership/borrow check v0 (`HIR-CHECK-V0-20260913`)

| Archivo | Código | Mensaje canónico | Trigger |
|------|------|-------------------|---------|
| `e0201-use-after-move.arita` | **E0201** | `use of moved value` | `String`/`Vec` move then Path use |
| `e0202-double-mut.arita` | **E0202** | `borrow conflict` | two overlapping `&mut` on same place |
| `e0206-bad-method.arita` | **E0206** | `method not in F2 std whitelist` | e.g. `Int.len()` / unknown method (ADR-026) |

## E021x — anti-theater (parse)

| Archivo | Código | Mensaje canónico | Trigger |
|------|------|-------------------|---------|
| `e0210-todo.arita` | **E0210** | `todo/unimplemented not allowed` | `todo!()` |
| `e0210-unimplemented.arita` | **E0210** | `todo/unimplemented not allowed` | `unimplemented!()` |
| `e0210-todo-kw.arita` | **E0210** | `todo/unimplemented not allowed` | bare `todo` keyword |
| `e0211-assert-true.arita` | **E0211** | `assert requires evidence` | `assert true` / `assert(true)` / lone `LitBool true` side |
| `e0212-empty-test.arita` | **E0212** | `empty test not allowed` | `test t { }` or comments only |
| `e0213-empty-fn.arita` | **E0213** | `stub function body not allowed` | valued `fn` (`ret ≠ Io<()>`) with empty body |
| `e0214-len-ge-zero.arita` | **E0214** | `vacuous length assert` | `assert v.len() >= 0` (ADR-030) |
| `e0214-len-eq-len.arita` | **E0214** | `vacuous length assert` | `assert v.len() == v.len()` |
| `e0214-is-empty-taut.arita` | **E0214** | `vacuous length assert` | `assert v.is_empty() || !v.is_empty()` |
| `e0215-eq-self.arita` | **E0215** | `vacuous comparison assert` | `assert 1 == 1` (ADR-038) |
| `e0215-le-self.arita` | **E0215** | `vacuous comparison assert` | `assert n <= n` |
| `e0216-div0.arita` | **E0216** | `integer division by zero` | `div(10,0)` / lit `/0` `%0` (ADR-044) |
| `e0217-add-overflow.arita` | **E0217** | `integer overflow` | LitInt `MAX + 1` (ADR-045; measure) |
| `e0217-runtime-max-plus.arita` | **E0217** | `integer overflow` | `let a=MAX; a+1` morph (companion) |
| `e0217-lit-sub-overflow.arita` | **E0217** | `integer overflow` | LitInt `MIN - 1` (companion) |
| `e0217-lit-mul-overflow.arita` | **E0217** | `integer overflow` | LitInt `MAX * 2` (companion) |

## Cómo comprobar

```bash
cargo run -p arita-cli -- parse ejemplos/f2/neg/e0201-use-after-move.arita
# → stderr: E0201: use of moved value ; exit ≠ 0

cargo run -p arita-cli -- parse ejemplos/f2/neg/e0202-double-mut.arita
# → stderr: E0202: borrow conflict ; exit ≠ 0

cargo run -p arita-cli -- parse ejemplos/f2/neg/e0210-todo.arita
# → E0210: todo/unimplemented not allowed

cargo run -p arita-cli -- parse ejemplos/f2/neg/e0211-assert-true.arita
# → E0211: assert requires evidence

cargo run -p arita-cli -- parse ejemplos/f2/neg/e0212-empty-test.arita
# → E0212: empty test not allowed

cargo run -p arita-cli -- parse ejemplos/f2/neg/e0213-empty-fn.arita
# → E0213: stub function body not allowed

cargo run -p arita-cli -- parse ejemplos/f2/neg/e0214-len-ge-zero.arita
# → E0214: vacuous length assert
```

Control positivo (debe seguir PASS): `ejemplos/f2/07-assert.arita` (`assert 2 + 2 == 4`).

CUT-ID: `HIR-CHECK-V0-20260913` / `E021X-NEG-20260913` / `TRAPS-LEN-THEATER-20260914`

## `arita measure`

Cableado como `NEG_ORACLES` en measure v0 de `arita-cli` para E021x; E0201/E0202 son negativos HIR-check (parse/build debe rechazar con mensaje codificado).
