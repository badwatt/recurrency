#!/usr/bin/env bash
# Format both backend and frontend

echo "Formatting backend..."
cd backend
nix develop .. -c cargo fmt

echo "Formatting frontend..."
cd ../frontend
nix develop .. -c cargo fmt

echo "Format complete!"
