#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

failed=0

fail() {
  printf 'error: %s\n' "$1" >&2
  failed=1
}

require_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    fail "missing required file: $path"
  fi
}

require_dir() {
  local path="$1"
  if [[ ! -d "$path" ]]; then
    fail "missing required directory: $path"
  fi
}

require_text() {
  local path="$1"
  local pattern="$2"
  local label="$3"
  if ! grep -Eq "$pattern" "$path"; then
    fail "$path does not contain required $label"
  fi
}

required_crates=(
  linguini-cli
  linguini-config
  linguini-syntax
  linguini-schema
  linguini-locale
  linguini-analyzer
  linguini-cldr
  linguini-ir
  linguini-codegen-ts
  linguini-codegen-js
  linguini-codegen-rust
  linguini-format
  linguini-lsp
  linguini-package
  linguini-test-support
)

for crate in "${required_crates[@]}"; do
  require_dir "crates/$crate"
  require_file "crates/$crate/Cargo.toml"
  require_text "Cargo.toml" "\"crates/$crate\"" "workspace member $crate"
done

required_stack=(
  clap
  serde
  toml
  serde_json
  chumsky
  ariadne
  thiserror
  camino
  ignore
  globwalk
  indexmap
  tower-lsp-server
  lsp-types
  tokio
  insta
  assert_cmd
  predicates
  tempfile
  proptest
  tracing
  tracing-subscriber
)

for dependency in "${required_stack[@]}"; do
  require_text "Cargo.toml" "\"$dependency\"" "planned dependency $dependency"
done

require_text "crates/linguini-cli/Cargo.toml" '^clap = ' "actual clap dependency for linguini-cli"
require_text "crates/linguini-cli/src/lib.rs" 'derive\(Debug, Parser\)' "clap Parser derive for CLI arguments"
require_text "crates/linguini-cli/src/lib.rs" 'derive\(Debug, Subcommand\)' "clap Subcommand derive for CLI commands"

require_file ".codex"
require_file "CONTRIBUTING.md"
require_text "CONTRIBUTING.md" "Bug fixes require a focused regression test" "regression-test rule"
for heading in \
  "Completed slice:" \
  "Implementation decisions:" \
  "Tests last run:" \
  "Known deferred work:" \
  "Next recommended task:"
do
  require_text ".codex" "^$heading$" "handoff heading $heading"
done

require_file "scripts/check-file-size.sh"
require_file "scripts/check-unit-test-structure.sh"
require_file "scripts/check-syntax-fixtures.sh"
require_file "scripts/review-snapshots.sh"
require_file "scripts/validate-generated-js.sh"
require_file "scripts/validate-generated-rust.sh"
require_file "scripts/coverage.sh"
require_text ".github/workflows/ci.yml" "cargo fmt --all --check" "format check"
require_text ".github/workflows/ci.yml" "./scripts/check-file-size.sh" "file-size check"
require_text ".github/workflows/ci.yml" "./scripts/check-unit-test-structure.sh" "unit test structure check"
require_text ".github/workflows/ci.yml" "./scripts/check-syntax-fixtures.sh" "syntax fixture check"
require_text ".github/workflows/ci.yml" "./scripts/review-snapshots.sh --check" "snapshot workflow check"
require_text ".github/workflows/ci.yml" "bash scripts/validate-generated-ts.sh" "generated TypeScript validation"
require_text ".github/workflows/ci.yml" "./scripts/validate-generated-js.sh" "generated JavaScript validation"
require_text ".github/workflows/ci.yml" "./scripts/validate-generated-rust.sh" "generated Rust validation"
require_text "CONTRIBUTING.md" "./scripts/coverage.sh" "coverage command"
require_text ".github/workflows/ci.yml" "cargo clippy --workspace --all-targets -- -D warnings" "clippy check"
require_text ".github/workflows/ci.yml" "cargo test --workspace" "workspace tests"
require_text ".github/workflows/ci.yml" "./scripts/check-spec-gates.sh" "spec gate check"

awk '
  /^## 0\. / { active = 1 }
  !active { next }
  /^- \[x\] / {
    if (item != "" && (!note || !evidence)) {
      printf "error: completed checklist item missing note/evidence near line %d: %s\n", line, item > "/dev/stderr"
      failed = 1
    }
    item = $0
    line = NR
    note = 0
    evidence = 0
    next
  }
  item != "" && /^  - Note: completed on [0-9]{4}-[0-9]{2}-[0-9]{2}\./ { note = 1 }
  item != "" && /^  - Evidence: .+/ { evidence = 1 }
  END {
    if (item != "" && (!note || !evidence)) {
      printf "error: completed checklist item missing note/evidence near line %d: %s\n", line, item > "/dev/stderr"
      failed = 1
    }
    exit failed
  }
' docs/checklist.md || failed=1

awk '
  /^## 0\. / { active = 1 }
  !active { next }
  /^- \[x\] / {
    if (item != "" && block ~ /(simplified substitute|simplify, skip, or omit|skip specified behavior|omit specified behavior|fragment-only syntax fixture)/) {
      printf "error: completed checklist item appears to claim simplified/omitted behavior near line %d: %s\n", line, item > "/dev/stderr"
      failed = 1
    }
    item = $0
    line = NR
    block = ""
    next
  }
  /^- \[[ x]\] / {
    if (item != "" && block ~ /(simplified substitute|simplify, skip, or omit|skip specified behavior|omit specified behavior|fragment-only syntax fixture)/) {
      printf "error: completed checklist item appears to claim simplified/omitted behavior near line %d: %s\n", line, item > "/dev/stderr"
      failed = 1
    }
    item = ""
    block = ""
    next
  }
  item != "" { block = block $0 "\n" }
  END {
    if (item != "" && block ~ /(simplified substitute|simplify, skip, or omit|skip specified behavior|omit specified behavior|fragment-only syntax fixture)/) {
      printf "error: completed checklist item appears to claim simplified/omitted behavior near line %d: %s\n", line, item > "/dev/stderr"
      failed = 1
    }
    exit failed
  }
' docs/checklist.md || failed=1

if find crates -path '*/src/main.rs' -type f -exec wc -l {} + | awk '$2 != "total" && $1 > 80 { exit 1 }'; then
  :
else
  fail "main.rs files must stay thin; move business logic to library modules"
fi

exit "$failed"
