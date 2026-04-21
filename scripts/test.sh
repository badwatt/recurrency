#!/usr/bin/env bash
# Test both backend and frontend

set -e

echo "Testing backend..."
cd backend
nix develop .. -c cargo test --lib

echo ""
echo "Testing frontend..."
cd ../frontend
nix develop .. -c cargo test --lib

echo ""
echo "All tests passed!"
