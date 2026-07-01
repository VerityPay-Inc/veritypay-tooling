#!/usr/bin/env bash
# Readiness gate for veritypay-tooling.
# Run from the repository root or any directory; resolves paths relative to this script.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> cargo fmt --check"
cargo fmt --check

echo "==> cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> cargo test --workspace"
cargo test --workspace

SPEC="${ROOT}/../veritypay-spec"
if [[ -d "$SPEC" ]]; then
    echo "==> cargo run -p vp-cli -- validate --spec ${SPEC}"
    cargo run -p vp-cli -- validate --spec "$SPEC"

    echo "==> cargo run -p vp-cli -- validate --spec ${SPEC} --format json"
    cargo run -p vp-cli -- validate --spec "$SPEC" --format json
else
    echo "==> skipping spec validation: ${SPEC} not found"
    echo "    clone veritypay-spec alongside this repository to run end-to-end validation"
fi

echo "readiness gate passed"
