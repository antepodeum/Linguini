#!/usr/bin/env bash
set -euo pipefail

warn_limit="${LINGUINI_WARN_SOURCE_FILE_LINES:-400}"
fail_limit="${LINGUINI_MAX_SOURCE_FILE_LINES:-500}"
failed=0

while IFS= read -r -d '' file; do
  line_count="$(wc -l < "$file")"

  if (( line_count > fail_limit )); then
    printf 'error: %s has %s lines, above limit %s\n' "$file" "$line_count" "$fail_limit" >&2
    failed=1
  elif (( line_count > warn_limit )); then
    printf 'warning: %s has %s lines, above warning %s\n' "$file" "$line_count" "$warn_limit" >&2
  fi
done < <(
  find crates scripts tests .github \
    -type f \
    \( -name '*.rs' -o -name '*.sh' -o -name '*.yml' -o -name '*.yaml' \) \
    ! -path '*/target/*' \
    ! -path '*/vendor/*' \
    ! -path '*/generated/*' \
    ! -path '*/snapshots/*' \
    -print0
)

exit "$failed"
