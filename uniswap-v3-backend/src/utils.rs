use anyhow::{Result, anyhow};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::types::{Swap, TraderStats};

pub fn parse_decimal(s: &str) -> Result<Decimal> {
    s.parse::<Decimal>()
        .map_err(|e| anyhow!("Failed to parse decimal '{}': {}", s, e))
}

pub fn determine_trade_type(swap: &Swap, target_token: &str) -> Result<(bool, Decimal, Decimal)> {
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

pub fn aggregate_trader_stats(
    swaps: &[Swap],
    target_token: &str,
) -> Result<HashMap<String, TraderStats>> {
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

    println!(
        "Processed all swaps. Found {} unique traders.",
        trader_stats.len()
    );
    Ok(trader_stats)
}

pub fn print_leaderboard(trader_stats: HashMap<String, TraderStats>, limit: usize) {
    let mut traders: Vec<TraderStats> = trader_stats.into_values().collect();

    // Sort by total USD volume (descending)
    traders.sort_by(|a, b| b.total_volume_usd().cmp(&a.total_volume_usd()));

    println!("\n🏆 UNISWAP V3 TRADER LEADERBOARD 🏆");
    println!(
        "═══════════════════════════════════════════════════════════════════════════════════════"
    );
    println!(
        "{:<4} {:<42} {:<8} {:<8} {:<15} {:<15} {:<15}",
        "Rank",
        "Trader Address",
        "Buys",
        "Sells",
        "Total Vol USD",
        "Net Token Vol",
        "Buy/Sell Ratio"
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────"
    );

    for (i, trader) in traders.iter().take(limit).enumerate() {
        let buy_sell_ratio = if trader.total_sells > 0 {
            format!(
                "{:.2}",
                trader.total_buys as f64 / trader.total_sells as f64
            )
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

    println!(
        "═══════════════════════════════════════════════════════════════════════════════════════"
    );

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
    println!(
        "Average Volume per Trader: ${:.2}",
        if total_traders > 0 {
            total_volume / Decimal::from(total_traders)
        } else {
            Decimal::ZERO
        }
    );
}

pub fn get_default_start_block() -> u64 {
    // Approximate block number for 30 days ago
    // Ethereum produces ~1 block every 12 seconds
    // 30 days = 30 * 24 * 60 * 60 / 12 = 216,000 blocks
    let current_block = 18_500_000u64; // Approximate current block (this would be dynamic in production)
    current_block.saturating_sub(216_000)
}

pub fn generate_demo_data() -> HashMap<String, TraderStats> {
    let mut trader_stats = HashMap::new();

    // Sample trader data to demonstrate the leaderboard functionality
    let demo_traders = vec![
        (
            "0x1234567890123456789012345678901234567890",
            45,
            32,
            "1234.5678",
            "987.1234",
            "125000.50",
            "98000.25",
        ),
        (
            "0x2345678901234567890123456789012345678901",
            23,
            41,
            "567.8901",
            "789.2345",
            "87500.75",
            "95000.00",
        ),
        (
            "0x3456789012345678901234567890123456789012",
            67,
            28,
            "2345.6789",
            "456.7890",
            "156000.25",
            "45000.80",
        ),
        (
            "0x4567890123456789012345678901234567890123",
            12,
            18,
            "345.6789",
            "234.5678",
            "34500.00",
            "28900.50",
        ),
        (
            "0x5678901234567890123456789012345678901234",
            89,
            76,
            "3456.7890",
            "2345.6789",
            "245000.75",
            "198000.25",
        ),
        (
            "0x6789012345678901234567890123456789012345",
            34,
            56,
            "1234.5678",
            "1567.8901",
            "89000.50",
            "112000.75",
        ),
        (
            "0x7890123456789012345678901234567890123456",
            78,
            43,
            "2789.0123",
            "1234.5678",
            "189000.25",
            "87500.50",
        ),
        (
            "0x8901234567890123456789012345678901234567",
            25,
            67,
            "567.8901",
            "1890.1234",
            "56000.75",
            "145000.25",
        ),
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
