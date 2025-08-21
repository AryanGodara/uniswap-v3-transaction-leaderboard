use anyhow::{anyhow, Result};
use clap::Parser;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(name = "uni-leaderboard")]
#[command(about = "Fetches Uniswap v3 swap data and creates trader leaderboards")]
struct Args {
    /// Token contract address (ERC20) - not required in demo mode
    #[arg(short, long)]
    token: Option<String>,

    /// Start block number (optional, defaults to ~30 days ago)
    #[arg(short, long)]
    start_block: Option<u64>,

    /// End block number (optional, defaults to latest)
    #[arg(short, long)]
    end_block: Option<u64>,

    /// Maximum number of traders to display in leaderboard
    #[arg(short, long, default_value = "20")]
    limit: usize,

    /// Run in demo mode with sample data (for testing when subgraph is unavailable)
    #[arg(long)]
    demo: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphQLQuery {
    query: String,
    variables: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct SwapsResponse {
    swaps: Vec<Swap>,
}

#[derive(Debug, Deserialize, Clone)]
struct Swap {
    id: String,
    timestamp: String,
    sender: String,
    recipient: String,
    #[serde(rename = "amount0")]
    amount_0: String,
    #[serde(rename = "amount1")]
    amount_1: String,
    #[serde(rename = "amountUSD")]
    amount_usd: String,
    pool: Pool,
    transaction: Transaction,
}

#[derive(Debug, Deserialize, Clone)]
struct Pool {
    id: String,
    #[serde(rename = "token0")]
    token_0: Token,
    #[serde(rename = "token1")]
    token_1: Token,
    #[serde(rename = "tick")]
    tick: Option<String>,
    #[serde(rename = "sqrtPrice")]
    sqrt_price: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Token {
    id: String,
    symbol: String,
    name: String,
    decimals: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Transaction {
    #[serde(rename = "blockNumber")]
    block_number: String,
}

#[derive(Debug, Clone)]
struct TraderStats {
    address: String,
    total_buys: u32,
    total_sells: u32,
    total_buy_volume_token: Decimal,
    total_sell_volume_token: Decimal,
    total_buy_volume_usd: Decimal,
    total_sell_volume_usd: Decimal,
}

impl TraderStats {
    fn new(address: String) -> Self {
        Self {
            address,
            total_buys: 0,
            total_sells: 0,
            total_buy_volume_token: Decimal::ZERO,
            total_sell_volume_token: Decimal::ZERO,
            total_buy_volume_usd: Decimal::ZERO,
            total_sell_volume_usd: Decimal::ZERO,
        }
    }

    fn total_volume_usd(&self) -> Decimal {
        self.total_buy_volume_usd + self.total_sell_volume_usd
    }

    fn net_volume_token(&self) -> Decimal {
        self.total_buy_volume_token - self.total_sell_volume_token
    }
}

struct UniswapClient {
    client: Client,
    subgraph_url: String,
}

impl UniswapClient {
    fn new() -> Self {
        // Use environment variable or fallback to a public endpoint
        // Note: For production use, you should set UNISWAP_SUBGRAPH_URL with a proper Graph Network endpoint
        let subgraph_url = std::env::var("UNISWAP_SUBGRAPH_URL")
            .unwrap_or_else(|_| {
                // Using the correct Uniswap v3 subgraph ID - requires API key for production use
                "https://gateway.thegraph.com/api/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV".to_string()
            });
        
        Self {
            client: Client::new(),
            subgraph_url,
        }
    }

    async fn fetch_swaps(
        &self,
        token_address: &str,
        start_block: Option<u64>,
        end_block: Option<u64>,
        skip: usize,
        first: usize,
    ) -> Result<Vec<Swap>> {
        let token_lower = token_address.to_lowercase();
        
        // Use a simple, working query structure
        let query = r#"
            query GetSwaps($skip: Int!, $first: Int!) {
                swaps(
                    skip: $skip,
                    first: $first,
                    orderBy: timestamp,
                    orderDirection: desc
                ) {
                    id
                    timestamp
                    sender
                    recipient
                    amount0
                    amount1
                    amountUSD
                    pool {
                        id
                        token0 {
                            id
                            symbol
                            name
                            decimals
                        }
                        token1 {
                            id
                            symbol
                            name
                            decimals
                        }
                        tick
                        sqrtPrice
                    }
                    transaction {
                        blockNumber
                    }
                }
            }
            "#.to_string();

        let variables = serde_json::json!({
            "skip": skip,
            "first": first
        });

        let request = GraphQLQuery { query, variables };

        let response = self
            .client
            .post(&self.subgraph_url)
            .json(&request)
            .send()
            .await?;

        let graphql_response: GraphQLResponse<SwapsResponse> = response.json().await?;

        if let Some(errors) = graphql_response.errors {
            let error_messages: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
            let combined_errors = error_messages.join(", ");
            
            // Provide helpful guidance for common errors
            if combined_errors.contains("auth") || combined_errors.contains("authorization") {
                return Err(anyhow!(
                    "Authentication error: The Graph Network endpoint requires an API key.\n\
                    \n\
                    To fix this:\n\
                    1. Get a free API key from https://thegraph.com\n\
                    2. Set the environment variable:\n\
                       export UNISWAP_SUBGRAPH_URL=\"https://gateway.thegraph.com/api/YOUR_API_KEY/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV\"\n\
                    3. Or try demo mode: {} --demo --limit 10",
                    std::env::args().next().unwrap_or("uni-leaderboard".to_string())
                ));
            } else if combined_errors.contains("subgraph not found") {
                return Err(anyhow!(
                    "Subgraph not found error: The subgraph endpoint may be incorrect.\n\
                    \n\
                    To fix this:\n\
                    1. Verify you're using the correct subgraph ID\n\
                    2. Check if you have a valid API key\n\
                    3. Or try demo mode: {} --demo --limit 10\n\
                    \n\
                    Original error: {}",
                    std::env::args().next().unwrap_or("uni-leaderboard".to_string()),
                    combined_errors
                ));
            } else {
                return Err(anyhow!("GraphQL errors: {}", combined_errors));
            }
        }

        match graphql_response.data {
            Some(data) => {
                // Filter swaps to only include those involving our target token
                let filtered_swaps: Vec<Swap> = data.swaps.into_iter()
                    .filter(|swap| {
                        let token0_id = swap.pool.token_0.id.to_lowercase();
                        let token1_id = swap.pool.token_1.id.to_lowercase();
                        token0_id == token_lower || token1_id == token_lower
                    })
                    .filter(|swap| {
                        // Apply block range filtering if specified
                        if let Ok(block_num) = swap.transaction.block_number.parse::<u64>() {
                            let in_start_range = start_block.map_or(true, |start| block_num >= start);
                            let in_end_range = end_block.map_or(true, |end| block_num <= end);
                            in_start_range && in_end_range
                        } else {
                            true // Include if we can't parse block number
                        }
                    })
                    .collect();
                Ok(filtered_swaps)
            },
            None => Ok(vec![]),
        }
    }

    async fn fetch_all_swaps(
        &self,
        token_address: &str,
        start_block: Option<u64>,
        end_block: Option<u64>,
    ) -> Result<Vec<Swap>> {
        let mut all_swaps = Vec::new();
        let mut skip = 0;
        const BATCH_SIZE: usize = 1000;

        println!("Fetching swap data from Uniswap v3 subgraph...");

        loop {
            let swaps = self
                .fetch_swaps(token_address, start_block, end_block, skip, BATCH_SIZE)
                .await?;

            if swaps.is_empty() {
                break;
            }

            println!("Fetched {} swaps (total: {})", swaps.len(), all_swaps.len() + swaps.len());
            
            let batch_len = swaps.len();
            all_swaps.extend(swaps);

            if batch_len < BATCH_SIZE {
                break;
            }

            skip += BATCH_SIZE;
        }

        println!("Total swaps fetched: {}", all_swaps.len());
        Ok(all_swaps)
    }
}

fn parse_decimal(s: &str) -> Result<Decimal> {
    s.parse::<Decimal>()
        .map_err(|e| anyhow!("Failed to parse decimal '{}': {}", s, e))
}

fn determine_trade_type(swap: &Swap, target_token: &str) -> Result<(bool, Decimal, Decimal)> {
    let target_token = target_token.to_lowercase();
    let token0_id = swap.pool.token_0.id.to_lowercase();
    let token1_id = swap.pool.token_1.id.to_lowercase();

    let amount0 = parse_decimal(&swap.amount_0)?;
    let amount1 = parse_decimal(&swap.amount_1)?;
    let amount_usd = parse_decimal(&swap.amount_usd)?;

    if token0_id == target_token {
        // Target token is token0
        // If amount0 > 0, tokens are going out of pool (sell)
        // If amount0 < 0, tokens are coming into pool (buy)
        let is_buy = amount0 < Decimal::ZERO;
        let token_amount = amount0.abs();
        Ok((is_buy, token_amount, amount_usd))
    } else if token1_id == target_token {
        // Target token is token1
        // If amount1 > 0, tokens are going out of pool (sell)
        // If amount1 < 0, tokens are coming into pool (buy)
        let is_buy = amount1 < Decimal::ZERO;
        let token_amount = amount1.abs();
        Ok((is_buy, token_amount, amount_usd))
    } else {
        Err(anyhow!("Target token not found in swap pool"))
    }
}

fn aggregate_trader_stats(swaps: &[Swap], target_token: &str) -> Result<HashMap<String, TraderStats>> {
    let mut trader_stats: HashMap<String, TraderStats> = HashMap::new();

    println!("Processing {} swaps...", swaps.len());

    for (i, swap) in swaps.iter().enumerate() {
        if i % 1000 == 0 && i > 0 {
            println!("Processed {} swaps", i);
        }

        match determine_trade_type(swap, target_token) {
            Ok((is_buy, token_amount, usd_amount)) => {
                // Use sender as the trader address (the one initiating the swap)
                let trader_address = swap.sender.clone();
                
                let stats = trader_stats
                    .entry(trader_address.clone())
                    .or_insert_with(|| TraderStats::new(trader_address));

                if is_buy {
                    stats.total_buys += 1;
                    stats.total_buy_volume_token += token_amount;
                    stats.total_buy_volume_usd += usd_amount;
                } else {
                    stats.total_sells += 1;
                    stats.total_sell_volume_token += token_amount;
                    stats.total_sell_volume_usd += usd_amount;
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to process swap {}: {}", swap.id, e);
                continue;
            }
        }
    }

    println!("Processed all swaps. Found {} unique traders.", trader_stats.len());
    Ok(trader_stats)
}

fn print_leaderboard(trader_stats: HashMap<String, TraderStats>, limit: usize) {
    let mut traders: Vec<TraderStats> = trader_stats.into_values().collect();
    
    // Sort by total USD volume (descending)
    traders.sort_by(|a, b| b.total_volume_usd().cmp(&a.total_volume_usd()));
    
    println!("\n🏆 UNISWAP V3 TRADER LEADERBOARD 🏆");
    println!("═══════════════════════════════════════════════════════════════════════════════════════");
    println!(
        "{:<4} {:<42} {:<8} {:<8} {:<15} {:<15} {:<15}",
        "Rank", "Trader Address", "Buys", "Sells", "Total Vol USD", "Net Token Vol", "Buy/Sell Ratio"
    );
    println!("─────────────────────────────────────────────────────────────────────────────────────────");

    for (i, trader) in traders.iter().take(limit).enumerate() {
        let buy_sell_ratio = if trader.total_sells > 0 {
            format!("{:.2}", trader.total_buys as f64 / trader.total_sells as f64)
        } else if trader.total_buys > 0 {
            "∞".to_string()
        } else {
            "0".to_string()
        };

        let net_volume = trader.net_volume_token();
        let net_volume_str = if net_volume >= Decimal::ZERO {
            format!("+{:.4}", net_volume)
        } else {
            format!("{:.4}", net_volume)
        };

        println!(
            "{:<4} {:<42} {:<8} {:<8} ${:<14.2} {:<15} {:<15}",
            i + 1,
            trader.address,
            trader.total_buys,
            trader.total_sells,
            trader.total_volume_usd(),
            net_volume_str,
            buy_sell_ratio
        );
    }

    println!("═══════════════════════════════════════════════════════════════════════════════════════");
    
    // Summary statistics
    let total_traders = traders.len();
    let total_volume: Decimal = traders.iter().map(|t| t.total_volume_usd()).sum();
    let total_buys: u32 = traders.iter().map(|t| t.total_buys).sum();
    let total_sells: u32 = traders.iter().map(|t| t.total_sells).sum();

    println!("\n📊 SUMMARY STATISTICS");
    println!("─────────────────────");
    println!("Total Traders: {}", total_traders);
    println!("Total Volume (USD): ${:.2}", total_volume);
    println!("Total Buy Transactions: {}", total_buys);
    println!("Total Sell Transactions: {}", total_sells);
    println!("Average Volume per Trader: ${:.2}", 
        if total_traders > 0 { total_volume / Decimal::from(total_traders) } else { Decimal::ZERO });
}

fn get_default_start_block() -> u64 {
    // Approximate block number for 30 days ago
    // Ethereum produces ~1 block every 12 seconds
    // 30 days = 30 * 24 * 60 * 60 / 12 = 216,000 blocks
    let current_block = 18_500_000u64; // Approximate current block (this would be dynamic in production)
    current_block.saturating_sub(216_000)
}

fn generate_demo_data() -> HashMap<String, TraderStats> {
    let mut trader_stats = HashMap::new();
    
    // Sample trader data to demonstrate the leaderboard functionality
    let demo_traders = vec![
        ("0x1234567890123456789012345678901234567890", 45, 32, "1234.5678", "987.1234", "125000.50", "98000.25"),
        ("0x2345678901234567890123456789012345678901", 23, 41, "567.8901", "789.2345", "87500.75", "95000.00"),
        ("0x3456789012345678901234567890123456789012", 67, 28, "2345.6789", "456.7890", "156000.25", "45000.80"),
        ("0x4567890123456789012345678901234567890123", 12, 18, "345.6789", "234.5678", "34500.00", "28900.50"),
        ("0x5678901234567890123456789012345678901234", 89, 76, "3456.7890", "2345.6789", "245000.75", "198000.25"),
        ("0x6789012345678901234567890123456789012345", 34, 56, "1234.5678", "1567.8901", "89000.50", "112000.75"),
        ("0x7890123456789012345678901234567890123456", 78, 43, "2789.0123", "1234.5678", "189000.25", "87500.50"),
        ("0x8901234567890123456789012345678901234567", 25, 67, "567.8901", "1890.1234", "56000.75", "145000.25"),
    ];

    for (address, buys, sells, buy_vol, sell_vol, buy_usd, sell_usd) in demo_traders {
        let mut stats = TraderStats::new(address.to_string());
        stats.total_buys = buys;
        stats.total_sells = sells;
        stats.total_buy_volume_token = buy_vol.parse().unwrap();
        stats.total_sell_volume_token = sell_vol.parse().unwrap();
        stats.total_buy_volume_usd = buy_usd.parse().unwrap();
        stats.total_sell_volume_usd = sell_usd.parse().unwrap();
        
        trader_stats.insert(address.to_string(), stats);
    }

    trader_stats
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    let args = Args::parse();

    // Validate arguments based on mode
    if !args.demo {
        match &args.token {
            Some(token) => {
                if !token.starts_with("0x") || token.len() != 42 {
                    return Err(anyhow!("Invalid token address format. Expected 42-character hex string starting with '0x'"));
                }
            }
            None => {
                return Err(anyhow!("Token address is required when not in demo mode. Use --token <ADDRESS> or --demo for sample data."));
            }
        }
    }

    let start_block = args.start_block.unwrap_or_else(get_default_start_block);
    let end_block = args.end_block;

    println!("🚀 Starting Uniswap v3 Trader Leaderboard Analysis");
    if let Some(token) = &args.token {
        println!("Token Address: {}", token);
    }
    if !args.demo {
        println!("Start Block: {}", start_block);
        if let Some(end) = end_block {
            println!("End Block: {}", end);
        } else {
            println!("End Block: Latest");
        }
    }
    println!("Leaderboard Limit: {}", args.limit);
    println!();

    let trader_stats = if args.demo {
        println!("🎭 Running in DEMO mode with sample data");
        println!("   (This demonstrates the tool's functionality when subgraph data is available)");
        println!();
        generate_demo_data()
    } else {
        let client = UniswapClient::new();
        let token = args.token.as_ref().unwrap(); // Safe because we validated above
        
        let swaps = client
            .fetch_all_swaps(token, Some(start_block), end_block)
            .await?;

        if swaps.is_empty() {
            println!("⚠️  No swaps found for the specified token and block range.");
            println!();
            println!("💡 Possible reasons:");
            println!("   • The subgraph endpoint requires an API key (see README for setup)");
            println!("   • No trading activity in the specified block range");
            println!("   • Token address is incorrect or not traded on Uniswap v3");
            println!();
            println!("🎭 Try running with --demo flag to see sample output:");
            println!("   {} --demo --limit 5", std::env::args().next().unwrap_or("uni-leaderboard".to_string()));
            return Ok(());
        }

        let stats = aggregate_trader_stats(&swaps, token)?;

        if stats.is_empty() {
            println!("⚠️  No valid trader statistics could be calculated.");
            return Ok(());
        }

        stats
    };

    print_leaderboard(trader_stats, args.limit);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trader_stats_creation() {
        let stats = TraderStats::new("0x123".to_string());
        assert_eq!(stats.address, "0x123");
        assert_eq!(stats.total_buys, 0);
        assert_eq!(stats.total_sells, 0);
    }

    #[test]
    fn test_parse_decimal() {
        assert!(parse_decimal("123.456").is_ok());
        assert!(parse_decimal("-123.456").is_ok());
        assert!(parse_decimal("invalid").is_err());
    }
}
