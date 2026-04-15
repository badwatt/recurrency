#!/usr/bin/env bash
set -e

# ── Recurrency Development Environment ──────────────────────
# Usage: ./dev.sh [command]
#
# Commands:
#   db        Start PostgreSQL
#   backend   Start backend (cargo run)
#   dx        Start Dioxus frontend (dx serve --web) + Tailwind watch
#   all       Start db + backend + frontend + Tailwind watch
#   stop      Stop all services
#   test      Run all tests
#   css       Build Tailwind CSS once
#   help      Show this help

CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
DB_CMD="docker compose -f docker-compose.dev.yml"

# Load .env
if [ -z "$DATABASE_URL" ]; then
  if [ -f "$PROJECT_ROOT/.env" ]; then
    source "$PROJECT_ROOT/.env"
  elif [ -f "$PROJECT_ROOT/.env.example" ]; then
    source "$PROJECT_ROOT/.env.example"
  else
    echo -e "${RED}No .env file found. Copy .env.example to .env and edit.${NC}"
    exit 1
  fi
fi

cmd_db() {
  echo -e "${CYAN}Starting PostgreSQL...${NC}"
  $DB_CMD up -d
  echo -e "${GREEN}PostgreSQL running on localhost:5432${NC}"
}

cmd_backend() {
  echo -e "${CYAN}Starting backend...${NC}"
  cd "$PROJECT_ROOT/backend"
  export APP_HOST="${APP_HOST:-localhost}"
  export APP_PORT="${APP_PORT:-8080}"
  export JWT_SECRET="${JWT_SECRET:-dev-secret}"
  export ENABLE_DEMO="${ENABLE_DEMO:-true}"
  cargo run
}

cmd_dx() {
  echo -e "${CYAN}Starting Dioxus frontend...${NC}"
  cd "$PROJECT_ROOT/frontend"
  export RECURRENCY_API_URL="${RECURRENCY_API_URL:-http://localhost:8080}"
  # Build CSS first, then watch alongside dx serve
  npx tailwindcss -i assets/tailwind.src.css -o assets/tailwind.css --minify 2>/dev/null && cp assets/tailwind.css public/tailwind.css || true
  npx tailwindcss -i assets/tailwind.src.css -o public/tailwind.css --watch &
  TW_PID=$!
  trap "kill $TW_PID 2>/dev/null" EXIT
  nix-shell -p openssl pkg-config llvmPackages_19.lld --run "dx serve --platform web --port 1420"
  kill $TW_PID 2>/dev/null
}

cmd_all() {
  cmd_db
  echo ""
  echo -e "${GREEN}Starting backend and frontend in parallel...${NC}"
  echo -e "${RED}Press Ctrl+C to stop.${NC}"
  cd "$PROJECT_ROOT/frontend"
  npx tailwindcss -i assets/tailwind.src.css -o assets/tailwind.css --minify 2>/dev/null && cp assets/tailwind.css public/tailwind.css || true
  npx tailwindcss -i assets/tailwind.src.css -o public/tailwind.css --watch &
  TW_PID=$!
  cmd_backend &
  BACKEND_PID=$!
  sleep 3
  cd "$PROJECT_ROOT/frontend"
  export RECURRENCY_API_URL="${RECURRENCY_API_URL:-http://localhost:8080}"
  nix-shell -p openssl pkg-config llvmPackages_19.lld --run "dx serve --platform web --port 1420" &
  DX_PID=$!
  trap "kill $TW_PID $BACKEND_PID $DX_PID 2>/dev/null" EXIT
  wait $BACKEND_PID $DX_PID
  kill $TW_PID 2>/dev/null
}

cmd_stop() {
  echo -e "${CYAN}Stopping services...${NC}"
  $DB_CMD down 2>/dev/null || true
  echo -e "${GREEN}All services stopped.${NC}"
}

cmd_test() {
  echo -e "${CYAN}Running backend tests...${NC}"
  cd "$PROJECT_ROOT/backend"
  cargo test 2>&1 || true

  echo ""
  echo -e "${CYAN}Running Dioxus frontend tests...${NC}"
  cd "$PROJECT_ROOT/frontend"
  nix-shell -p openssl pkg-config llvmPackages_19.lld --run "cargo test 2>&1" || cargo test 2>&1 || true
}

cmd_help() {
  cat <<EOF
Recurrency Development Environment

Usage: ./dev.sh [command]

Commands:
  db        Start PostgreSQL
  backend   Start Rust backend (port 8080)
  dx        Start Dioxus frontend (port 1420)
  all       Start db + backend + frontend
  stop      Stop all services
  test      Run all tests
  css       Build Tailwind CSS once
  help      Show this help

Environment:
  Set via .env file (copy .env.example)

Examples:
  ./dev.sh db      # Start just the database
  ./dev.sh all     # Start everything
  ./dev.sh test    # Run all tests
  ./dev.sh css     # Build Tailwind CSS once
EOF
}

cmd_css() {
  echo -e "${CYAN}Building Tailwind CSS...${NC}"
  cd "$PROJECT_ROOT/frontend"
  npx tailwindcss -i assets/tailwind.src.css -o assets/tailwind.css --minify
  cp assets/tailwind.css public/tailwind.css
  echo -e "${GREEN}CSS built.${NC}"
}

case "${1:-help}" in
db) cmd_db ;;
backend) cmd_backend ;;
dx) cmd_dx ;;
all) cmd_all ;;
stop) cmd_stop ;;
test) cmd_test ;;
css) cmd_css ;;
help) cmd_help ;;
*)
  echo -e "${RED}Unknown command: $1${NC}"
  cmd_help
  exit 1
  ;;
esac