#!/bin/bash
# Post-edit formatter hook: runs the appropriate formatter based on file type.
# Called by Claude Code after Edit/Write tool use.
# Receives tool input as JSON on stdin.

set -uo pipefail

FILE=$(jq -r '.tool_input.file_path // empty')

if [[ -z "$FILE" ]]; then
  exit 0
fi

PROJECT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"

case "$FILE" in
  *.rs | */Cargo.toml | Cargo.toml)
    rustfmt +nightly "$FILE" 2>&1 || true
    cargo check --message-format=short --quiet 2>&1 || true
    ;;
  *.ts | *.js | *.html)
    cd "$PROJECT_DIR"
    pnpm exec prettier --write "$FILE" 2>&1 || true
    ;;
esac
