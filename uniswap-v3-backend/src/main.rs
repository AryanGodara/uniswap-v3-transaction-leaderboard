mod client;
mod config;
mod handlers;
mod server;
mod types;
mod utils;

use anyhow::{Result, anyhow};
use clap::Parser;

use crate::client::UniswapClient;
use crate::config::Config;
use crate::server::run_server;
use crate::utils::{
    aggregate_trader_stats, generate_demo_data, get_default_start_block, print_leaderboard,
};

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
    #[arg(short, long)]
    limit: Option<usize>,

    /// Run in demo mode with sample data (for testing when subgraph is unavailable)
    #[arg(long)]
    demo: bool,

    /// Run as HTTP server for frontend integration
    #[arg(long)]
    server: bool,

    /// Server port
    #[arg(long)]
    port: Option<u16>,

    /// Network to query (ethereum, arbitrum, polygon, optimism, base)
    #[arg(long, default_value = "ethereum")]
    network: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let config = Config::from_env()?;
    let args = Args::parse();

    // Use config defaults for optional args
    let limit = args.limit.unwrap_or(config.default_limit);
    let port = args.port.unwrap_or(config.default_port);

    // If server mode, run HTTP server
    if args.server {
        println!("🚀 Starting Uniswap v3 Leaderboard HTTP Server");
        println!("Port: {}", port);
        println!("API key: configured and ready");
        println!();
        return run_server(port).await;
    }

    // Validate arguments based on mode for CLI
    if !args.demo {
        match &args.token {
            Some(token) => {
                if !token.starts_with("0x") || token.len() != 42 {
                    return Err(anyhow!(
                        "Invalid token address format. Expected 42-character hex string starting with '0x'"
                    ));
                }
            }
            None => {
                return Err(anyhow!(
                    "Token address is required when not in demo mode. Use --token <ADDRESS> or --demo for sample data."
                ));
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
    println!("Leaderboard Limit: {}", limit);
    println!();

    let trader_stats = if args.demo {
        println!("🎭 Running in DEMO mode with sample data");
        println!("   (This demonstrates the tool's functionality when subgraph data is available)");
        println!();
        generate_demo_data()
    } else {
        let client = UniswapClient::new(&args.network)?;
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
            println!(
                "   {} --demo --limit 5",
                std::env::args()
                    .next()
                    .unwrap_or("uni-leaderboard".to_string())
            );
            return Ok(());
        }

        let stats = aggregate_trader_stats(&swaps, token)?;

        if stats.is_empty() {
            println!("⚠️  No valid trader statistics could be calculated.");
            return Ok(());
        }

        stats
    };

    print_leaderboard(trader_stats, limit);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        let config = Config::from_env().unwrap();
        assert!(!config.graph_api_key.is_empty());
        assert!(config.default_port > 0);
        assert!(config.default_limit > 0);
    }
}
