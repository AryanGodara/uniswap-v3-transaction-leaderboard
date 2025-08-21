# Uniswap V3 Transaction Leaderboard

Full-stack application for analyzing Uniswap V3 trading data and displaying trader leaderboards.

## Quick Start

```bash
# Install dependencies
make install

# Start both backend (port 3001) and frontend (port 3000)
make start
```

## Available Commands

```bash
make help           # Show all available commands
make install        # Install all dependencies
make build          # Build both projects
make lint           # Run linting
make start          # Start both services
make start-backend  # Start only backend (port 3001)
make start-frontend # Start only frontend (port 3000)
make stop           # Stop all running services
make clean          # Clean build artifacts
```

## Project Structure

```bash
├── backend/        # Rust backend server
├── frontend/       # Next.js frontend
└── Makefile        # Build automation
```

## Usage

1. **Development**: `make start` - starts backend then frontend
2. **Individual services**: Use `make start-backend` or `make start-frontend`
3. **Production build**: `make build`
4. **Code quality**: `make lint`

The backend runs on port 3001 and serves API endpoints. The frontend runs on port 3000 and connects to the backend automatically.
