#!/bin/bash

# Example script for using uni-leaderboard
# This script demonstrates various ways to use the tool

echo "🚀 Uni-Leaderboard Example Usage"
echo "=================================="

# Build the project first
echo "Building the project..."
cargo build --release

echo ""
echo "Example 1: Demo mode (works without API key)"
echo "Command: ./target/release/uni-leaderboard --demo --limit 10"
echo ""

# Example 1: Demo mode
./target/release/uni-leaderboard --demo --limit 10

echo ""
echo "Example 2: Real data mode (requires API key setup)"
echo "Command: ./target/release/uni-leaderboard --token 0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 --limit 5"
echo "Note: This will show how to set up API key if not configured"
echo ""

# Example 2: Real data mode (will show API key setup instructions)
./target/release/uni-leaderboard \
  --token 0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 \
  --limit 5

echo ""
echo "Example 3: Show help"
echo "Command: ./target/release/uni-leaderboard --help"
echo ""

# Example 3: Show help
./target/release/uni-leaderboard --help

echo ""
echo "✅ Examples completed!"
echo ""
echo "💡 Tips:"
echo "  - Use real token addresses from popular tokens like USDC, WETH, DAI"
echo "  - Adjust block ranges based on when the token was active"
echo "  - Use environment variable UNISWAP_SUBGRAPH_URL to specify custom endpoints"
echo "  - Start with smaller block ranges for faster testing"
