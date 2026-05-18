#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
config="$repo_root/crates/linguini-cldr/cldr-json.toml"
destination="${LINGUINI_CLDR_VENDOR_DIR:-$repo_root/crates/linguini-cldr/vendor/cldr-json}"

read_config_value() {
  local key="$1"
  sed -nE "s/^[[:space:]]*$key[[:space:]]*=[[:space:]]*\"([^\"]+)\".*/\1/p" "$config" | head -n1
}

repo="$(read_config_value repo)"
ref="$(read_config_value ref)"
commit_prefix="$(read_config_value commit_prefix)"

if [[ -z "$repo" || -z "$ref" || -z "$commit_prefix" ]]; then
  printf 'error: %s must define repo, ref, and commit_prefix\n' "$config" >&2
  exit 1
fi

if [[ -d "$destination/.git" ]]; then
  head="$(git -C "$destination" rev-parse HEAD 2>/dev/null || true)"
  if [[ "$head" == "$commit_prefix"* ]]; then
    printf 'CLDR JSON already present at %s (%s)\n' "$destination" "$head"
    exit 0
  fi
  printf 'Replacing CLDR JSON checkout at %s: got %s, expected prefix %s\n' "$destination" "${head:-unknown}" "$commit_prefix" >&2
  rm -rf "$destination"
elif [[ -e "$destination" ]]; then
  printf 'Replacing non-git CLDR JSON path at %s\n' "$destination" >&2
  rm -rf "$destination"
fi

mkdir -p "$(dirname "$destination")"

git -c init.defaultBranch=main init -q "$destination"
git -C "$destination" remote add origin "$repo"
git -C "$destination" fetch --quiet --depth=1 origin "$ref"
git -C "$destination" checkout --quiet --detach FETCH_HEAD

head="$(git -C "$destination" rev-parse HEAD)"
if [[ "$head" != "$commit_prefix"* ]]; then
  printf 'error: CLDR JSON ref %s resolved to %s, expected prefix %s\n' "$ref" "$head" "$commit_prefix" >&2
  exit 1
fi

if [[ ! -f "$destination/cldr-json/cldr-core/supplemental/plurals.json" ]]; then
  printf 'error: checkout is missing cldr-json/cldr-core/supplemental/plurals.json\n' >&2
  exit 1
fi

printf 'Fetched CLDR JSON %s into %s\n' "$head" "$destination"
