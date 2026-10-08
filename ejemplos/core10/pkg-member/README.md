# core10/pkg-member — ADR-287 (B-286-10)

`arita build <input.arita>` only compiles an `input` that is a member of the `[workspace]`
(canonical path match). Otherwise: `E0332`, exit 1, empty stdout, nothing mutated.

- Members (`members = ["lib", "bin"]`): `lib/lib.arita`, `bin/main.arita` (prints `member-main:` then `42`).
- Non-members: `bin/other.arita`, `bin/other-record.arita`, `bin/other-edge.arita` → `E0332`.

Tests: `crates/arita-cli/tests/pkg_member.rs` (PM-1..PM-7). They copy this fixture to a tmpdir;
the fixture itself is never built in place.
