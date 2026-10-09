Translation of `287-core-package-member-v0.md`; the original is normative. / Traducción de `287-core-package-member-v0.md`; el original es el normativo.

# ADR-287 — Core package-member v0: `arita build <input>` only builds an `input` that is a `[workspace]` member (B-286-10)

- **Estado:** **CLOSED (Ingeniero, GO CLOSED 2026-10-02; DOC/GATE-CORE10-PKG-MEMBER-20261002.md)** · `arita measure` 851/851 accepted (N = 851, k = 7) · historia: PINS APROBADOS (Ingeniero 01-10), v0.3 `b80cec57b6b9c94626d9c61cf8a2f53758bdf853a16b842280a4938a288ba1ec` verificada, IMPL tras S1b CLOSED; freeze `DOC/reviews/MEASURE-ADR287-PKG-MEMBER-FREEZE-20261002.sha`.
- **CUT-ID:** `CORE-0.10-PKG-MEMBER` (el del GATE y del freeze; el de apertura fue `CORE-PKG-MEMBER-20261001`)
- **Fecha:** 2026-10-01
- **Autores:** ARITA Arquitecto (pins) · prioridad y orden decididos por el Ingeniero 01-10 (S1b → B-286-10 → unused_parens → IndexMut → Mutex)
- **Padre / contexto:** [ADR-255](255-core-package-manifest-v0.md) · comentario de `main.rs` (~L263, «ADR-263: resolve runnable crates/bin_* artifact») — **referencia a verificar**: `263-core-ref-coll-v0.md` es REF-COLL y no respalda la resolución del artefacto `bin_*` (errata del Ingeniero 01-10) · [ADR-286](286-core-0.10-errores-fase1.md) fila **B-286-10 (P1)** (hallazgo de Ingeniero/Codegen 27-09; datos en `crates/arita-cli/src/main.rs:256-283`)
- **Cierra:** **B-286-10** (CLOSED 2026-10-02)
- **Código:** **E0332** `input is not a workspace member` (nuevo, familia manifest/package `E033x`; ver §3).
- **No reabre:** E0330 / E0331 (semántica actual intacta) · ADR-255 · ADR-286 · E0291 / E0340–E0343 · ADR-265 / 270 / 278 / 283.

## 1. Problem (facts, not hypotheses)

`arita build <input.arita>` today, if an `arita.toml` with `[workspace]` exists in any of the (up to 8) parent directories, **ignores `input`**:

- `main.rs:259`: `if let Some(_manifest) = package::load_package_manifest(arita_path)?` — `find_arita_toml` (`deps.rs:15-27`) walks up at most 8 levels; the found manifest wins.
- `main.rs:260`: `build_package_workspace(arita_path, profile)` builds **all** members (`package.rs:280-290`) after `remove_dir_all(target/arita-pkg)`.
- `main.rs:266-277`: `bin_guess` = the **first** `bin_*` that `read_dir` returns (non-deterministic order), unrelated to `input`.
- `main.rs:278-281`: `return Ok(BuildOutput{ bin: bin_guess, .. })`. The path that would emit `input` (`main.rs:284` onward) is never reached.
- `main.rs:29-31`: prints `ok: <bin>` and returns `SUCCESS`.

Repro confirmed by Codegen (27-09, uninstrumented `target/release/arita` binary, clean copy; real repo intact):

| input (non-member) | current result | should |
|---|---|---|
| `ejemplos/core04/lib-api/bin/main-record.arita` (module `demo_rec`) | exit 0, `ok: …/bin_demo`; that binary prints `3` | must not build; (the file is not a member) |
| `ejemplos/core04/ref-pkg-lib/bin/edge.arita` (module `arita_ref_pkg_lib_edge`) | exit 0, `ok: …/bin_arita_ref_pkg_lib`; prints `42` | must not build; (not a member) |

**Expected consequence after IMPL (no regression):** `arita build` of those two historical files will yield `E0332` (exit 1, empty stdout), because they are not members. The historical repros stay as documentation; new fixtures cover the negs (§5).

Both manifests declare `[workspace] members = ["lib","bin"]`; `member_arita_path` (`package.rs:204-211`) only resolves `<m>/{<m>,mod,main,lib}.arita` under the member dir — so `bin/main-record.arita` and `bin/edge.arita` are not members.

Additional side effect: the wrong build deletes the workspace’s `target/arita-pkg` (`package.rs:234-235`) even though `input` was not a member.

## 2. Decisions (pins)

**D1 — Membership.** After loading the manifest (`load_package_manifest` returns `Some`), the CLI resolves **each** member’s path with `member_arita_path(pkg_root, m)` (current candidate semantics, unchanged) and compares with `input` by **canonical path** (`std::fs::canonicalize` on both sides; resolves `.`, `..`, absolute/relative paths, and symlinks). `input` is a member iff its canonical path matches some member’s. A member whose `member_arita_path` fails (current E0331) still propagates that E0331 (not hidden). The workspace is the one `arita build` already resolves today (`find_arita_toml`, unchanged); membership = canonical path against its `members`. This ADR does **not** pin a “nearest manifest” rule.

**D2 — Non-member `input` ⇒ error, never success.** If `input` exists and is not a member: `Err("E0332: input <input> is not a member of workspace <arita.toml path> (members: <m1>, <m2>, …>)")`; stderr with that message, **empty stdout**, **exit 1**. Compiling it as a loose file is rejected (would silence a usage error: whoever puts a file under a workspace expects package semantics). The check happens **before any mutation**: no `remove_dir_all`, no `target/arita-pkg` creation, no `cargo`.

**D3 — Missing or unreadable `input` under a workspace.** `canonicalize(input)` fails ⇒ return the same I/O error the loose-file path produces today (`fs::read_to_string`, no E-code; exit 1). The workspace is never built for a nonexistent `input`. (Today it would be.)

**D4 — Member `input` ⇒ `ok:` names THAT member’s artifact.** The workspace is built as today, but `BuildOutput.bin` is selected from the crate emitted from `input`’s `.arita` (not the first `bin_*` from `read_dir`). If the member is `bin` (defines `main`), `ok:` points to its `bin_*`. If the member is `lib` (no `main`), that member has no `bin_*`: `ok:` points to the `target/<profile>/` directory (current fallback, `main.rs:277`), **never** another member’s `bin_*`. The path printed after `ok:` **contains no component starting with `bin_`**. With several bin members, the `input`’s wins. Deterministic selection (not `read_dir` order). How to implement it (return crate name from `build_package_workspace` or derive it from `input`’s module) is IMPL’s choice; the contract is this section’s and is verified by oracle (§5).

**D5 — Check scope.** Only the `target.is_none()` path of `build_with_entry_profile_target` (the one that enters the workspace today). Callers of `build_package_workspace` in `measure.rs` (≈30 sites) pass members by construction and **do not** change signature or behavior; the check lives in a new `package.rs` function (free name, e.g. `resolve_input_member`) invoked from `main.rs`. `find_arita_toml` is not touched (still lexical, 8 levels; see §7 known limits). Any other `build_with_entry_profile*` caller (contract / attest; ADR-263 reference in main.rs comment, to verify) inherits the check; the Measure PREP (§6) must list them.

**D6 — No changes to `arita test`, `parse`, `--target`.** `arita test` and `parse` do not load a manifest (unchanged). With `--target <triple>` the path skips the workspace (as today).

## 3. Error code

Reusing **E0331** was evaluated (`package.rs:29,39,218,227…`: unreadable/invalid manifest, empty `members`, member without `.arita`). It does not fit: here the manifest is **valid** and the defect is the user’s **input**; mixing both would stop oracles distinguishing “broken manifest” from “wrong file”. Therefore new **E0332**: `input is not a workspace member`. Checked before drafting that `E0332` does not appear in ADR 280–286 or in `crates/arita-cli/src` (whole-repo check done when creating this file). E0330 and E0331 unchanged. Catalog/DOC registration of the code: IMPL/Docs at CLOSED.

## 4. Cargo tests (arita-cli; indicative names; exact count and names fixed by IMPL)

Unit in `package.rs` (no cargo/rustc), on a temporary tree with `arita.toml` `[workspace] members = ["lib","bin"]`, `lib/lib.arita`, `bin/main.arita`:

1. `member_input_direct_path` — `bin/main.arita` resuelve a miembro `bin`.
2. `member_input_dotdot_and_dot_forms` — `./bin/../bin/main.arita` and the absolute form resolve to the same member.
3. `member_input_symlink` (`#[cfg(unix)]`) — symlink a `bin/main.arita` resuelve a `bin`.
4. `nonmember_input_is_e0332` — `bin/other.arita` ⇒ `Err` starting with `E0332:` containing the manifest path and members.
5. `nonmember_check_has_no_side_effects` — before the call `target/arita-pkg/marker` is created; after the `Err` the marker is still there (no `remove_dir_all`).
6. `missing_input_is_io_error_not_e0332` — nonexistent path ⇒ `Err` without `E0332` and without building the workspace.
7. `lib_member_has_no_bin_artifact` — input = `lib/lib.arita` ⇒ member `lib`; artifact selector returns the profile dir `target/<profile>/`, never another member’s `bin_*`.
8. `two_bin_members_pick_input_crate` — workspace with two members that have `main`; the selector returns the input’s crate, for both inputs, in both orders.

Migration: no existing test should change; if the PREP (§6) finds one that builds a non-member under a `[workspace]`, it is listed as BLOCKER.

## 5. `measure` oracles (proposed; exact N fixed by the Engineer)

New fixture (created by IMPL, not this ADR): `ejemplos/core10/pkg-member/` with `arita.toml`, `lib/lib.arita`, `bin/main.arita` (prints a known line), `bin/other.arita` (non-member; would print a different line) and two more non-members that replace the historical core04 repros: `bin/other-record.arita` (same shape as main-record: record + print, own module, different output from `bin/main.arita`) and `bin/other-edge.arita` (same shape as edge, different output). A “build must fail with Exxxx” oracle does **not** fit `NegOracle` (`measure.rs:2549-2556`, only `parse_lower_check`); it is implemented as an own function in `measure.rs` that invokes the real build flow and checks the result, registered in `cmd_measure`.

- **PM-1 (verde):** build of `bin/main.arita` ⇒ `Ok`; the `bin`’s `file_name` starts with `bin_`; running that binary produces **exactly** the expected `main.arita` output (proves it is not another member’s).
- **PM-2 (neg):** build of `bin/other.arita` ⇒ **exit 1, empty stdout** and stderr with exact code `E0332` (message starting with `E0332:`); no `ok:`. Also the fixture file-manifest sha256 **does not change** and, if `target/arita-pkg` existed before, it stays identical (marker intact).
- **PM-3 (verde):** build of `main.arita` via a different path form (`./bin/../bin/main.arita`) ⇒ `Ok` and same output as PM-1.
- **PM-4 (neg):** build of `ejemplos/core10/pkg-member/bin/other-record.arita` (NEW fixture) ⇒ **exit 1, empty stdout**, stderr with `E0332`; the **fixture** sha256 manifest and `target/arita-pkg` (if it existed) do not change.
- **PM-5 (neg):** build of `ejemplos/core10/pkg-member/bin/other-edge.arita` (NEW fixture) ⇒ **exit 1, empty stdout**, stderr with `E0332`; same non-mutation checks as PM-4 (fixture sha256 manifest and `target/arita-pkg` if it existed).
- **PM-6 (neg):** nonexistent `input` under the workspace ⇒ **exit 1, empty stdout**, I/O stderr **without** `E0332`; `target/arita-pkg` (if it existed) and the sha256 manifest do not change.
- **PM-7 (verde, caso lib D4):** build of `lib/lib.arita` (`lib` member, no `main`) ⇒ **exit 0**, stdout `ok: <target/<profile>/> profile directory`, and the printed path **contains no `bin_*` component**; empty stderr. Exact form of the original bug (input without its own `bin_*` ending in another member’s `bin_*`); cargo test 7 only tests the selector, PM-7 tests the CLI end-to-end (Engineer decision 01-10).

Neg oracles run as a process (`arita build`) or via a function that returns exit, stdout, and stderr separately; checking only `Err(String)` is not enough.

k = 7 (PM-1…PM-7; PM-7 added by Engineer decision 01-10). **N is not fixed in this ADR:** N is always prior N_CLOSED + k, computed by the Engineer at each slice’s GO IMPL. With the current order S1b (844) → PKG-MEMBER → S2 (k = 7), **provisional** figures are PKG-MEMBER = 844 + 7 = **851** and S2 = 851 + 7 = **858** (ADR-286 keeps 851 only while S2 immediately follows S1b; recalculated at GO IMPL).

Rules: skip ≠ PASS; no oracle counts if it uses another member’s binary; neg oracles must not modify the fixture tree (sha256 manifest before/after).

## 6. Measure PREP (blocks GO IMPL)

Read-only scan, report in `DOC/reviews/PREP_ADR287_PKG_MEMBER_SCAN_<fecha>.md` with md5/sha:

1. Every caller of `build_with_entry_profile`, `build_with_entry_profile_target` (main.rs, contract.rs, attest.rs, measure.rs, tests) and its real `input`.
2. For each: is there an `arita.toml` with `[workspace]` within ≤8 parent levels? is `input` a member (D1)? Any **non-member under a workspace** used as a green build ⇒ BLOCKER.
3. All repo `.arita` under a `[workspace]` that are **not** members (inventory; known examples: the two of §1).
4. Confirm no green oracle depends on `ok:` pointing at `read_dir`’s first `bin_*`.

**PREP result (Measure, md5 `03882290`, reported by the Orchestrator 01-10): 0 BLOCKERs.** Report: `DOC/reviews/PREP_ADR287_PACKAGE_MEMBER_SCAN_20261001.md` (md5 verified with `md5 -q`: `03882290bd02ae8ffd2f5fd914ebd588`, matches).

## 7. Out of scope / known limits

- `find_arita_toml` does not canonicalize: a relative `input` without a directory (`main.arita`) does not discover upper manifests and is compiled as a loose file — known limit, unchanged.
- `--target <triple>` skips the workspace path (D6). **B-287-2 (P3)**.
- **B-287-3 (P3):** build of an `input` under an autonomous nested manifest (an `arita.toml` closer than the one declaring the resolved workspace): today the resolved workspace is what `find_arita_toml` returns; there are 12 nested manifests of this kind in the repo (Engineer data 01-10). Not changed here; decided in a later slice.
- `member_arita_path` resolution rules (candidates) and manifests with differently shaped `members`: unchanged.
- Message with spans / rich diagnosis format: N/A (CLI error, not compiler).

## 8. Slice, order and GO

- **Single slice** `PKG-MEMBER` (size S). Order decided by the Engineer: S1b → **this** → unused_parens/B-282 → IndexMut → Mutex.
- Sequence: DOC (this ADR) → Measure PREP (§6) → Engineer verification → Engineer **GO IMPL** → implementation (Orchestrator/Codegen) → Lex → CLOSED with real evidence. Inventing PASS / N/N / CLOSED is forbidden.
- At CLOSED: ADR-286 close edit (B-286-10 row → CLOSED by ADR-287) and E0332 registration in catalogs, both by Architect/Docs.

## 9. Changelog

- 2026-10-01 — v0 DRAFT PINS (Architect). No Engineer verification yet; no own execution data (§1 facts are those reconfirmed by Codegen).
- 2026-10-01 — v0.1 (Architect): Engineer erratum: ADR-263/ADR-255 reference (§Parent, D5); N as prior N_CLOSED + k with provisional 850/857 (§5); neg PM-2/4/5/6 with exit 1 + empty stdout + exact stderr + no-mutation (§5); D4 lib case without `bin_*` (D4, test 7, PM note). Pins D1–D6 unchanged.
- 2026-10-01 — v0.2 (Architect): Engineer decision: PM-7 (lib case, end-to-end CLI) ⇒ k = 7; provisional N PKG-MEMBER 851, S2 858 (recalculated at GO IMPL); Status → PINS APPROVED, IMPL blocked until S1b CLOSED and Measure PREP §6 without BLOCKER. Pins D1–D6 unchanged.
- 2026-10-01 — v0.3 (Architect): Engineer decisions: D1 without a “nearest manifest” pin (workspace = what `find_arita_toml` resolves); B-287-3 (P3, 12 autonomous nested manifests); PM-4/PM-5 with NEW fixture `ejemplos/core10/pkg-member/` (`other-record`, `other-edge`), core04 repros historical only and E0332 expected as consequence; Measure PREP §6 = 0 BLOCKERs (md5 03882290). Pins D1–D6 otherwise unchanged.

## 10. Close (CLOSED 2026-10-02)

Datos tomados de `DOC/GATE-CORE10-PKG-MEMBER-20261002.md` (sha256 `d617170232db85c3fe0547aca2ea4ce0fc4b48f2fb3b6a5ff5772a663ab8558a`).

- **Veredicto:** GO CLOSED (Ingeniero Rust, 2026-10-02).
- **E0332** `input is not a workspace member`: `arita build <input>` in a package only accepts workspace members; a non-member yields E0332 (exit 1, empty stdout, without mutating the tree). A nonexistent entry remains an I/O error (PM-6), not E0332.
- **Oracles** (k = 7, N = 851): PM-1 `core10-pkg-member-bin-ok` · PM-2 `neg-core10-pkg-member-other` · PM-3 `core10-pkg-member-dotdot` (includes the simple relative path) · PM-4 `neg-core10-pkg-member-other-record` · PM-5 `neg-core10-pkg-member-other-edge` · PM-6 `neg-core10-pkg-member-missing-input` · PM-7 `core10-pkg-member-lib`.
- **Evidencia** (exclusive Lex run, 2026-10-02 19:26–20:55 CEST): `cargo fmt --all -- --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` 0; `cargo build --release -p arita-cli` 0; `cargo test` 498 passed / 0 failed; `arita measure` **851/851 accepted** with no skips; miri-workspace 0 failures.
- **Veyra Proof** (quick): #1 `20261002T185553Z` **REJECTED** (VT006 ×3 in `tests/pkg_member.rs`, remediated in test code only, no waiver); #2 `20261002T193930Z` **ACCEPTED**.
- **Measurement artifact:** `DOC/reviews/MEASURE_ADR287_PKG_MEMBER_EXCLUSIVE_20261002.json` (sha256 `3a031a66…`, md5 `9afbc616…`). Freeze: `DOC/reviews/MEASURE-ADR287-PKG-MEMBER-FREEZE-20261002.sha`.
- **Shas al cierre:** `main.rs` `ed0820b4…`, `package.rs` `5fee2beb…`, `measure.rs` `10443913…`, `tests/pkg_member.rs` `9172a51e…`, `tests/measure_stdout.rs` `8bc7ba81…`.
- **B-measure-stdout closed here:** `arita measure` stdout is pure JSON (test `tests/measure_stdout.rs`).
- **Consecuencia:** historic core04 non-members (`lib-api/bin/main-record.arita`, `ref-pkg-lib/bin/edge.arita`) and the 3 non-members of the new fixture now get E0332 instead of silently building another member’s bin.
- **Known limitation:** the verification script’s `contract` comparison exercises nothing; effective coverage is PM-1..PM-7.
- **Backlog:** B-287-1/2/3 (P3) stay open (§7).
- **Registro de E0332:** there is no dedicated E-code catalog; canonical definitions are this ADR, `package.rs`, `measure.rs` and `tests/pkg_member.rs`.
- **Next in order:** ADR-288 → ADR-289 → ADR-290 (IndexMut).
- Note: ADR sha after this close: recorded by the Engineer addendum.
