Español | [English](README.md)

# core10/pkg-member — ADR-287 (B-286-10)

`arita build <input.arita>` solo compila un `input` que sea miembro del `[workspace]`
(coincidencia de ruta canónica). Si no: `E0332`, exit 1, stdout vacío, nada mutado.

- Miembros (`members = ["lib", "bin"]`): `lib/lib.arita`, `bin/main.arita` (imprime `member-main:` y luego `42`).
- No miembros: `bin/other.arita`, `bin/other-record.arita`, `bin/other-edge.arita` → `E0332`.

Tests: `crates/arita-cli/tests/pkg_member.rs` (PM-1..PM-7). Copian este fixture a un tmpdir;
el fixture en sí nunca se construye in-place.
