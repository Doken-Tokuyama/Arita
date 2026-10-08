#!/usr/bin/env bash
# ARITA local-first CI gate: fmt (if available) → clippy → test → measure.
# Measure exit codes passthrough: 0=accepted, 1=rejected, 2=inconclusive/usage.
# skip ≠ PASS — missing rustfmt is WARN/inconclusive, never claimed as fmt PASS.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "======== ARITA CI ========"
echo "root: ${ROOT}"
echo

echo "-------- fmt --------"
if rustfmt --version >/dev/null 2>&1; then
  cargo fmt --check
  echo "fmt: check ok"
else
  echo "WARN: rustfmt not available; fmt check skipped (inconclusive; skip ≠ PASS)"
fi
echo

echo "-------- clippy --------"
cargo clippy \
  -p arita-syntax \
  -p arita-codegen \
  -p arita-hir \
  -p arita-logic \
  -p arita-cli \
  -- -D warnings
echo "clippy: ok"
echo

echo "-------- test --------"
cargo test --workspace -- --test-threads=1
echo "test: ok"
echo

echo "-------- measure --------"
set +e
cargo run -q -p arita-cli -- measure
measure_ec=$?
set -e
echo "measure exit: ${measure_ec} (0=accepted, 1=rejected, 2=inconclusive)"
exit "${measure_ec}"
