#!/usr/bin/env bash
# Sync VERSION file to all project files that hardcode the version
set -euo pipefail

VERSION=$(cat VERSION)

if [ -z "$VERSION" ]; then
  echo "ERROR: VERSION file is empty"
  exit 1
fi

echo "Syncing version: $VERSION"

# package.json
sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" frontend/package.json

# tauri.conf.json
sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" frontend/src-tauri/tauri.conf.json

# Cargo.toml (frontend)
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" frontend/src-tauri/Cargo.toml

# Cargo.toml (backend)
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" backend/Cargo.toml

echo "Done. Verify with: grep -r '$VERSION' VERSION frontend/package.json frontend/src-tauri/tauri.conf.json frontend/src-tauri/Cargo.toml backend/Cargo.toml"
