# Uniswap V3 Leaderboard - Makefile
# Minimal build and run automation for frontend + backend

.PHONY: help install build lint start-backend start-frontend start clean

# Default target
help:
	@echo "Uniswap V3 Leaderboard - Available commands:"
	@echo ""
	@echo "  make install     - Install all dependencies"
	@echo "  make build       - Build both frontend and backend"
	@echo "  make lint        - Run linting for both projects"
	@echo "  make start       - Start both backend (port 3001) and frontend (port 3000)"
	@echo "  make start-backend - Start only the backend server"
	@echo "  make start-frontend - Start only the frontend"
	@echo "  make clean       - Clean build artifacts"
	@echo ""

# Install dependencies
install: install-backend install-frontend

install-backend:
	@echo "📦 Installing backend dependencies..."
	cd uniswap-v3-backend && cargo fetch

install-frontend:
	@echo "📦 Installing frontend dependencies..."
	cd leaderboard-frontend && npm install

# Build targets
build: build-backend build-frontend

build-backend:
	@echo "🔨 Building backend..."
	cd uniswap-v3-backend && cargo build --release

build-frontend: 
	@echo "🔨 Building frontend..."
	cd leaderboard-frontend && npm run build

# Linting
lint: lint-backend lint-frontend

lint-backend:
	@echo "🔍 Linting backend..."
	cd uniswap-v3-backend && cargo clippy -- -D warnings
	cd uniswap-v3-backend && cargo fmt --check

lint-frontend:
	@echo "🔍 Linting frontend..."
	cd leaderboard-frontend && npm run lint

# Start services
start-backend:
	@echo "🚀 Starting backend server on port 3001..."
	cd uniswap-v3-backend && cargo run --release -- --server --port 3001

start-frontend:
	@echo "🌐 Starting frontend on port 3000..."
	cd leaderboard-frontend && npm run dev

# Start both services (backend first, then frontend)
start:
	@echo "🚀 Starting Uniswap V3 Leaderboard..."
	@echo "📡 Starting backend server on port 3001..."
	cd uniswap-v3-backend && cargo run --release -- --server --port 3001 & \
	echo $$! > backend.pid && \
	sleep 5 && \
	echo "🌐 Starting frontend on port 3000..." && \
	cd leaderboard-frontend && npm run dev

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cd uniswap-v3-backend && cargo clean
	cd leaderboard-frontend && npm run clean 2>/dev/null || rm -rf .next node_modules/.cache
	rm -f backend.pid

# Stop running services
stop:
	@echo "🛑 Stopping services..."
	@if [ -f backend.pid ]; then \
		kill `cat backend.pid` 2>/dev/null || true; \
		rm -f backend.pid; \
	fi
	@pkill -f "cargo run.*--server" 2>/dev/null || true
	@pkill -f "npm run dev" 2>/dev/null || true
