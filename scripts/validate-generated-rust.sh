#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmpdir="$(mktemp -d /tmp/linguini-rust-run-XXXXXX)"
trap 'rm -rf "$tmpdir"' EXIT

cp -R "$repo_root/tests/fixtures/golden/snapshots/rust/." "$tmpdir"
cargo test --manifest-path "$tmpdir/Cargo.toml"
