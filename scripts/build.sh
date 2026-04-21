#!/usr/bin/env bash
# Build both backend and frontend

set -e

echo "Building backend..."
cd backend
nix develop .. -c cargo build --release

echo "Building frontend..."
cd ../frontend
nix develop .. -c dx build

echo "Build complete!"
