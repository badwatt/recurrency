#!/usr/bin/env bash
#
# Recurrency - Subscription & Expense Tracker
# Usage: ./re <command>
#

set -euo pipefail

# ── Mode (dev/prod) ────────────────────────────────────────

MODE="dev"
for arg in "$@"; do
  case $arg in
    --prod) MODE="prod" ;;
  esac
done

# Demo mode: enabled by default in dev, disabled in prod
if [ "$MODE" = "prod" ]; then
  export ENABLE_DEMO="false"
else
  export ENABLE_DEMO="true"
fi

# Colors
CB='\033[0;34m'
CG='\033[0;32m'
CY='\033[0;33m'
CR='\033[0;31m'
CN='\033[0m'

# Paths
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_BACKEND="$PROJECT_DIR/.backend.pid"
PID_FRONTEND="$PROJECT_DIR/.frontend.pid"
LOG_BACKEND="$PROJECT_DIR/backend.log"
LOG_FRONTEND="$PROJECT_DIR/frontend.log"
LOG_FILE="/dev/null"

# Helpers
info()  { printf "${CB}%s${CN}\n" "$*"; }
ok()    { printf "${CG}✅ %s${CN}\n" "$*"; }
warn()  { printf "${CY}⚠️  %s${CN}\n" "$*"; }
err()   { printf "${CR}❌ %s${CN}\n" "$*"; }

# ── Services ──────────────────────────────────────────────

start_db() {
  info "🐘 Starting PostgreSQL..."
  docker compose -f docker-compose.dev.yml up -d > /dev/null 2>&1
  ok "PostgreSQL running on port 5432"
}

stop_db() {
  info "🐘 Stopping PostgreSQL..."
  docker compose -f docker-compose.dev.yml down > /dev/null 2>&1
  ok "PostgreSQL stopped"
}

start_backend() {
  info "🦀 Starting Backend..."
  cd "$PROJECT_DIR/backend" && cargo run > "$LOG_BACKEND" 2>&1 &
  echo $! > "$PID_BACKEND"
  sleep 3
  ok "Backend started"
}

stop_backend() {
  info "🦀 Stopping Backend..."
  if [ -f "$PID_BACKEND" ]; then
    kill -TERM "$(cat "$PID_BACKEND")" 2>/dev/null || true
    rm -f "$PID_BACKEND"
  fi
  pkill -TERM -f "recurrency-api" 2>/dev/null || true
  pkill -9 -f "recurrency-api" 2>/dev/null || true
  ok "Backend stopped"
}

start_frontend() {
  info "⚛️  Starting Frontend..."
  cd "$PROJECT_DIR/frontend" && bun run dev > "$LOG_FRONTEND" 2>&1 &
  echo $! > "$PID_FRONTEND"
  sleep 3
  ok "Frontend started"
}

stop_frontend() {
  info "⚛️  Stopping Frontend..."
  if [ -f "$PID_FRONTEND" ]; then
    kill -TERM "$(cat "$PID_FRONTEND")" 2>/dev/null || true
    rm -f "$PID_FRONTEND"
  fi
  pkill -TERM -f "vite" 2>/dev/null || true
  pkill -9 -f "vite" 2>/dev/null || true
  ok "Frontend stopped"
}

is_running() { pgrep -f "$1" > /dev/null 2>&1; }

show_status() {
  info "📊 Recurrency Status:"
  echo

  if is_running "recurrency-api"; then
    ok "Backend running (PID: $(pgrep -f "recurrency-api" | head -1))"
  else
    err "Backend stopped"
  fi

  if is_running "vite"; then
    ok "Frontend running (PID: $(pgrep -f "vite" | head -1))"
  else
    err "Frontend stopped"
  fi

  if docker ps --filter name=recurrency-db --format "{{.Names}}" 2>/dev/null | grep -q recurrency-db; then
    ok "PostgreSQL running"
  else
    err "PostgreSQL stopped"
  fi
}

# ── Commands ──────────────────────────────────────────────

cmd_up() {
  info "🚀 Starting Recurrency..."
  start_db
  sleep 2
  start_backend
  start_frontend
  echo
  ok "All services started!"
  printf "${CY}Backend:  http://localhost:8080${CN}\n"
  printf "${CY}Frontend: http://localhost:1420${CN}\n"
  printf "${CY}Demo mode: %s${CN}\n" "$ENABLE_DEMO"
}

cmd_down() {
  info "🛑 Stopping services..."
  stop_frontend || true
  stop_backend || true
  stop_db || true
  ok "All services stopped"
}

cmd_restart() {
  cmd_down
  cmd_up
}

cmd_status() {
  show_status
}

cmd_logs() {
  tail -f "$LOG_BACKEND" 2>/dev/null || warn "No backend log found"
}

cmd_setup() {
  info "🔧 Setting up Recurrency..."
  cd "$PROJECT_DIR/frontend" && bun install
  cd "$PROJECT_DIR/backend" && cargo build
  ok "Setup complete!"
}

cmd_test() {
  cd "$PROJECT_DIR/frontend" && bun test
  cd "$PROJECT_DIR/backend" && cargo test
}

cmd_typecheck() {
  cd "$PROJECT_DIR/frontend" && bun run typecheck
}

cmd_build() {
  info "🔨 Building Recurrency..."
  cd "$PROJECT_DIR/frontend" && bun run build
  cd "$PROJECT_DIR/backend" && cargo build --release
  ok "Build complete!"
}

cmd_clean() {
  info "🧹 Cleaning..."
  rm -f "$PID_BACKEND" "$PID_FRONTEND" "$LOG_BACKEND" "$LOG_FRONTEND"
  cd "$PROJECT_DIR/frontend" && rm -rf dist node_modules/.vite
  cd "$PROJECT_DIR/backend" && cargo clean
  ok "Clean complete"
}

cmd_reset() {
  warn "This will DELETE ALL data in the database!"
  read -rp "Are you sure? [y/N] " confirm
  if [[ "$confirm" =~ ^[Yy]$ ]]; then
    info "Resetting database..."
    docker compose -f docker-compose.dev.yml down -v > /dev/null 2>&1
    docker compose -f docker-compose.dev.yml up -d > /dev/null 2>&1

    # Wait for PostgreSQL to be ready
    sleep 3
    for i in {1..10}; do
      if docker compose -f docker-compose.dev.yml exec -T db pg_isready -U recurrency > /dev/null 2>&1; then
        break
      fi
      sleep 1
    done

    # Recreate demo user via backend API
    info "Creating demo user..."
    curl -s http://localhost:8080/api/demo/init > /dev/null 2>&1 || true

    ok "Database reset complete"
  else
    echo "Cancelled."
  fi
}

cmd_help() {
  printf "\n${CB}Recurrency - Commands${CN}\n\n"
  printf "  ${CG}%-12s${CN} %s\n" "up" "Start all services"
  printf "  ${CG}%-12s${CN} %s\n" "up --prod" "Start in production mode"
  printf "  ${CG}%-12s${CN} %s\n" "down" "Stop all services"
  printf "  ${CG}%-12s${CN} %s\n" "restart" "Restart everything"
  printf "  ${CG}%-12s${CN} %s\n" "status" "Show service status"
  printf "  ${CG}%-12s${CN} %s\n" "logs" "Show backend logs"
  printf "  ${CG}%-12s${CN} %s\n" "setup" "Install deps & build"
  printf "  ${CG}%-12s${CN} %s\n" "test" "Run all tests"
  printf "  ${CG}%-12s${CN} %s\n" "typecheck" "TypeScript check"
  printf "  ${CG}%-12s${CN} %s\n" "build" "Production build"
  printf "  ${CG}%-12s${CN} %s\n" "clean" "Remove artifacts"
  printf "  ${CG}%-12s${CN} %s\n" "reset" "Reset database"
  echo
  printf "${CY}Demo mode: %s${CN}\n" "$ENABLE_DEMO"
  echo
}

# ── Dispatch ──────────────────────────────────────────────

case "${1:-up}" in
  up)        cmd_up ;;
  down)      cmd_down ;;
  restart)   cmd_restart ;;
  status)    cmd_status ;;
  logs)      cmd_logs ;;
  setup)     cmd_setup ;;
  test)      cmd_test ;;
  typecheck) cmd_typecheck ;;
  build)     cmd_build ;;
  clean)     cmd_clean ;;
  reset)     cmd_reset ;;
  *)         cmd_help ;;
esac
