#!/usr/bin/env bash
# Sync VERSION file to all project files that hardcode the version
# Run from project root or via tauri beforeDevCommand/beforeBuildCommand
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# Resolve project root relative to script location
if [[ "$SCRIPT_DIR" == *"/scripts" ]]; then
  ROOT="$SCRIPT_DIR/.."
else
  ROOT="."
fi
ROOT="$(cd "$ROOT" && pwd)"

VERSION=$(cat "$ROOT/VERSION")

if [ -z "$VERSION" ]; then
  echo "ERROR: VERSION file is empty"
  exit 1
fi

echo "Syncing version: $VERSION"

# package.json
sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" "$ROOT/frontend/package.json"

# tauri.conf.json
sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" "$ROOT/frontend/src-tauri/tauri.conf.json"

# Cargo.toml (frontend)
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" "$ROOT/frontend/src-tauri/Cargo.toml"

# Cargo.toml (backend)
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" "$ROOT/backend/Cargo.toml"

echo "Done. Version $VERSION synced."
# List synced files
grep -H "$VERSION" "$ROOT/VERSION" "$ROOT/frontend/package.json" "$ROOT/frontend/src-tauri/tauri.conf.json" "$ROOT/frontend/src-tauri/Cargo.toml" "$ROOT/backend/Cargo.toml" 2>/dev/null || true
