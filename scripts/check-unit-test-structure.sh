#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

failed=0

while IFS= read -r crate_manifest; do
  crate_dir="$(dirname "$crate_manifest")"
  crate_name="$(basename "$crate_dir")"

  if ! grep -R -Eq '#\[cfg\(test\)\]|#\[test\]' "$crate_dir/src"; then
    printf 'error: %s has no unit test structure under src/\n' "$crate_name" >&2
    failed=1
  fi
done < <(find crates -mindepth 2 -maxdepth 2 -name Cargo.toml | sort)

exit "$failed"
