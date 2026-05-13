#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

mode="${1:---check}"

snapshot_tests=(
  linguini-analyzer
  linguini-codegen-ts
  linguini-ir
  linguini-syntax
)

run_snapshot_tests() {
  local args=()
  local crate

  for crate in "${snapshot_tests[@]}"; do
    args+=("-p" "$crate")
  done

  cargo test "${args[@]}"
}

case "$mode" in
  --check)
    run_snapshot_tests
    ;;
  review)
    run_snapshot_tests
    if ! cargo insta --version >/dev/null 2>&1; then
      printf 'error: cargo-insta is required for interactive snapshot review\n' >&2
      printf 'hint: install with `cargo install cargo-insta`\n' >&2
      exit 1
    fi
    cargo insta review
    ;;
  *)
    printf 'usage: %s [--check|review]\n' "$0" >&2
    exit 2
    ;;
esac
