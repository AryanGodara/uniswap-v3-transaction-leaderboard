# Uni-Leaderboard

A Rust command-line tool that fetches Uniswap v3 swap transactions for a specific token and aggregates trader statistics to create leaderboards.

## Features

- Fetch swap data from Uniswap v3 subgraph
- Identify buy/sell transactions for any ERC20 token
- Track volume in both token units and USD equivalent
- Aggregate per-address statistics (buys, sells, volumes)
- Support for custom block ranges
- Pagination support for large datasets

## Installation

Build the project:
    ```bash
        cargo build --release
    ```

## Usage

### Basic Usage

#### Demo Mode (Recommended for Testing)

See how the tool works with sample data:
    ```bash
        cargo run -- --demo --limit 10
    ```

#### Real Data Mode

Analyze the last 30 days of trading for a token (requires API key - see setup below):
    ```bash
        cargo run -- --token 0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48
    ```

### Advanced Usage

#### Specify custom block range

    ```bash
        cargo run -- --token 0xA0b86a33E6441d83E3F5E5B3c4E5F6f8E7A8B9C0 --start-block 18000000 --end-block 18100000
    ```

#### Limit the number of traders shown

    ```bash
        cargo run -- --token 0xA0b86a33E6441d83E3F5E5B3c4E5F6f8E7A8B9C0 --limit 10
    ```

### Using with Alternative Subgraph Endpoints

The tool uses a public Graph Network endpoint by default. For production use with higher rate limits, you can specify a custom endpoint with your API key:

    ```bash
        # Using Graph Network with API key (recommended for production)
        UNISWAP_SUBGRAPH_URL="https://gateway.thegraph.com/api/YOUR_API_KEY/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV" cargo run -- --token 0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48

        # Using alternative public endpoint
        UNISWAP_SUBGRAPH_URL="https://your-custom-endpoint.com" cargo run -- --token 0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48
    ```

**Important**: The hosted service has been deprecated. You need a Graph Network API key for real data:

1. Visit [thegraph.com](https://thegraph.com) and create an account
2. Generate an API key in your dashboard
3. Set the environment variable with your API key:

   ```bash
   export UNISWAP_SUBGRAPH_URL="https://gateway.thegraph.com/api/YOUR_API_KEY/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV"
   ```

For testing without an API key, use the `--demo` flag to see sample output.

**Current Status**: The application is fully functional. Real data mode requires a Graph Network API key (free to obtain), while demo mode works immediately without any setup.

### Popular Token Addresses for Testing

- **USDC**: `0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48`
- **WETH**: `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2`
- **DAI**: `0x6B175474E89094C44Da98b954EedeAC495271d0F`
- **USDT**: `0xdAC17F958D2ee523a2206206994597C13D831ec7`

### Command Line Options

- `--token, -t`: Token contract address (required for real data mode)
- `--start-block, -s`: Start block number (optional, defaults to ~30 days ago)
- `--end-block, -e`: End block number (optional, defaults to latest)
- `--limit, -l`: Maximum number of traders to display (default: 20)
- `--demo`: Run in demo mode with sample data (no API key required)

## Example Output

```
🚀 Starting Uniswap v3 Trader Leaderboard Analysis
Token Address: 0xA0b86a33E6441d83E3F5E5B3c4E5F6f8E7A8B9C0
Start Block: 18284000
End Block: Latest
Leaderboard Limit: 20

Fetching swap data from Uniswap v3 subgraph...
Fetched 1000 swaps (total: 1000)
Total swaps fetched: 1000
Processing 1000 swaps...
Processed all swaps. Found 234 unique traders.

🏆 UNISWAP V3 TRADER LEADERBOARD 🏆
═══════════════════════════════════════════════════════════════════════════════════════
Rank Trader Address                           Buys     Sells    Total Vol USD   Net Token Vol   Buy/Sell Ratio
─────────────────────────────────────────────────────────────────────────────────────────
1    0x1234567890123456789012345678901234567890 45       32       $1,234,567.89   +1,234.5678     1.41
2    0x2345678901234567890123456789012345678901 23       41       $987,654.32     -567.8901       0.56
...

📊 SUMMARY STATISTICS
─────────────────────
Total Traders: 234
Total Volume (USD): $12,345,678.90
Total Buy Transactions: 1,234
Total Sell Transactions: 1,456
Average Volume per Trader: $52,779.83
```

## How It Works

1. **Data Fetching**: Queries the Uniswap v3 subgraph for swap transactions involving the specified token
2. **Trade Classification**: Determines whether each swap is a buy or sell based on token amounts
3. **Aggregation**: Groups trades by sender address and calculates statistics
4. **Leaderboard**: Sorts traders by total USD volume and displays formatted results

## Technical Details

- **Subgraph**: Uses The Graph's hosted Uniswap v3 subgraph
- **Pagination**: Automatically handles large datasets with 1000-swap batches
- **Buy/Sell Logic**: Based on token amount signs (negative = tokens entering pool = buy)
- **USD Conversion**: Uses price data from the subgraph's calculated USD amounts

## Dependencies

- `tokio`: Async runtime
- `reqwest`: HTTP client for GraphQL queries
- `serde`: JSON serialization/deserialization
- `clap`: Command-line argument parsing
- `anyhow`: Error handling
- `chrono`: Date/time handling
- `rust_decimal`: Precise decimal arithmetic
- `ethers`: Ethereum utilities

## Error Handling

The tool includes comprehensive error handling for:

- Invalid token addresses
- Network connectivity issues
- GraphQL query errors
- Data parsing errors
- Mathematical operations

## Limitations

- Relies on The Graph's hosted service availability
- USD values are based on subgraph calculations
- Historical data availability depends on subgraph indexing
- Rate limits may apply for very large datasets

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is open source and available under the MIT License.
