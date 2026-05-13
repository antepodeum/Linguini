#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

case "${1:-}" in
  --help|-h)
    cat <<'TXT'
usage: ./scripts/coverage.sh

Measures workspace coverage with cargo-llvm-cov and writes:
  target/coverage/lcov.info
  target/coverage/html/

Install tool:
  cargo install cargo-llvm-cov
TXT
    exit 0
    ;;
esac

if ! cargo llvm-cov --version >/dev/null 2>&1; then
  printf 'error: cargo-llvm-cov is required for coverage measurement\n' >&2
  printf 'hint: install with `cargo install cargo-llvm-cov`\n' >&2
  exit 1
fi

mkdir -p target/coverage

cargo llvm-cov clean --workspace
cargo llvm-cov \
  --workspace \
  --all-targets \
  --ignore-filename-regex '(/target/|/tests/fixtures/golden/snapshots/|/vendor/|/generated/)' \
  --lcov \
  --output-path target/coverage/lcov.info

cargo llvm-cov report \
  --html \
  --output-dir target/coverage/html
