#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

failed=0

fail() {
  printf 'error: %s\n' "$1" >&2
  failed=1
}

require_text() {
  local path="$1"
  local pattern="$2"
  local label="$3"

  if ! grep -Eq "$pattern" "$path"; then
    fail "$path does not cover $label"
  fi
}

schema_fixture="tests/fixtures/golden/schema/shop.lgs"
locale_fixture="tests/fixtures/golden/locale/ru.lgl"
syntax_schema_fixture="tests/fixtures/golden/syntax/all.lgs"
syntax_locale_fixture="tests/fixtures/golden/syntax/all.lgl"

require_text "$schema_fixture" '^enum [A-Z][A-Za-z0-9_]* \{.*\}$' "schema enum declarations"
require_text "$schema_fixture" '^type [A-Z][A-Za-z0-9_]* = [A-Z][A-Za-z0-9_]* @' "schema formatter annotations"
require_text "$schema_fixture" '^/// ' "schema doc comments"
require_text "$schema_fixture" '^[a-z][A-Za-z0-9_]*\([^)]*: [A-Z][A-Za-z0-9_]*' "message signatures"
require_text "$schema_fixture" '^[a-z][A-Za-z0-9_]* \{$' "grouped messages"

require_text "$locale_fixture" '^enum [A-Z][A-Za-z0-9_]* \{.*\}$' "locale enum declarations"
require_text "$locale_fixture" '^impl [A-Z][A-Za-z0-9_]* \{$' "locale impl declarations"
require_text "$locale_fixture" '^form [A-Z][A-Za-z0-9_]*\([A-Z][A-Za-z0-9_]*(, [A-Z][A-Za-z0-9_]*)*\) \{$' "typed dispatch forms"
require_text "$locale_fixture" '^fn [A-Z][A-Za-z0-9_]*\(' "local functions"
require_text "$locale_fixture" '^[[:space:]]+one =>' "plural-shaped branch maps"
require_text "$locale_fixture" '^[[:space:]]+[a-z][A-Za-z0-9_]* \{$' "nested form attributes"
require_text "$locale_fixture" '^[[:space:]]+_ =>' "wildcard fallback branches"
require_text "$locale_fixture" '^/// ' "locale doc comments"
require_text "$locale_fixture" '\{[a-z][A-Za-z0-9_]*(\.[a-z][A-Za-z0-9_]*)*(\([^}]*\))?\}' "placeholders"
require_text "$locale_fixture" '@currency\(' "placeholder formatter annotations"
require_text "$locale_fixture" '^[a-z][A-Za-z0-9_]* \{$' "grouped message implementations"

require_text "$syntax_schema_fixture" '^type [A-Z][A-Za-z0-9_]* = [A-Z][A-Za-z0-9_]* @.*\(' "schema annotation arguments"
require_text "$syntax_locale_fixture" '^override ' "locale override declarations"
require_text "$syntax_locale_fixture" '^fn [A-Z][A-Za-z0-9_]*\([a-z][A-Za-z0-9_]*: String, [A-Z][A-Za-z0-9_]*\)' "named and unnamed function parameters"
require_text "$syntax_locale_fixture" '^\s+formal \{$' "nested dispatch branches"

while IFS= read -r fixture; do
  line_count="$(wc -l < "$fixture")"
  if (( line_count < 8 )); then
    fail "$fixture is too small for a golden syntax fixture"
  fi
done < <(find tests/fixtures/golden -type f \( -name '*.lgs' -o -name '*.lgl' \) | sort)

while IFS= read -r fixture; do
  case "$fixture" in
    tests/fixtures/invalid/*) ;;
    *) fail "invalid diagnostic fixture is outside tests/fixtures/invalid: $fixture" ;;
  esac
done < <(find tests/fixtures/invalid -type f \( -name '*.lgs' -o -name '*.lgl' \) | sort)

exit "$failed"
