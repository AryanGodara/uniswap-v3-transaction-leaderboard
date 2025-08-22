use axum::{
    http::StatusCode,
    response::Json,
};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::client::UniswapClient;
use crate::types::{LeaderboardRequest, LeaderboardResponse, TraderStats, TraderStatsAPI, SummaryStats};
use crate::utils::{aggregate_trader_stats, generate_demo_data, get_default_start_block};

pub async fn health_check() -> &'static str {
    "Uniswap V3 Leaderboard API is running!"
}

pub async fn leaderboard_handler(
    Json(payload): Json<LeaderboardRequest>,
) -> Result<Json<LeaderboardResponse>, StatusCode> {
    println!("Received leaderboard request: {:?}", payload);

    let trader_stats = if payload.demo.unwrap_or(false) {
        println!("Running in demo mode");
        generate_demo_data()
    } else {
        match &payload.token_address {
            Some(token) => {
                if !token.starts_with("0x") || token.len() != 42 {
                    eprintln!("Invalid token address format: {}", token);
                    return Err(StatusCode::BAD_REQUEST);
                }

                // Additional validation for hex characters
                if !token[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                    eprintln!("Invalid token address (non-hex characters): {}", token);
                    return Err(StatusCode::BAD_REQUEST);
                }

                let network = payload.network.as_deref().unwrap_or("ethereum");
                let client = match UniswapClient::new(network) {
                    Ok(client) => client,
                    Err(e) => {
                        eprintln!("Failed to create client for network {}: {}", network, e);
                        return Err(StatusCode::BAD_REQUEST);
                    }
                };
                let start_block = payload.start_block.unwrap_or_else(get_default_start_block);

                println!("Fetching swaps for token: {}", token);
                match client
                    .fetch_all_swaps(token, Some(start_block), payload.end_block)
                    .await
                {
                    Ok(swaps) => {
                        if swaps.is_empty() {
                            println!("No swaps found for token");
                            HashMap::new()
                        } else {
                            match aggregate_trader_stats(&swaps, token) {
                                Ok(stats) => stats,
                                Err(e) => {
                                    eprintln!("Error aggregating stats: {}", e);
                                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching swaps for token {}: {}", token, e);

                        // Check for specific error types to provide better responses
                        let error_msg = e.to_string();
                        if error_msg.contains("HTML error page")
                            || error_msg.contains("Failed to parse API response")
                        {
                            eprintln!(
                                "Token {} appears to not exist or have no Uniswap V3 pools",
                                token
                            );
                            // Return empty data instead of error for better UX
                            HashMap::new()
                        } else {
                            return Err(StatusCode::INTERNAL_SERVER_ERROR);
                        }
                    }
                }
            }
            None => {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    };

    // Convert to API format
    let mut traders: Vec<TraderStatsAPI> = trader_stats
        .into_values()
        .map(|stats| {
            let total_volume = stats.total_buy_volume_usd + stats.total_sell_volume_usd;
            let net_volume = stats.total_buy_volume_token - stats.total_sell_volume_token;
            let buy_sell_ratio = if stats.total_sells > 0 {
                stats.total_buys as f64 / stats.total_sells as f64
            } else if stats.total_buys > 0 {
                stats.total_buys as f64
            } else {
                0.0
            };

            TraderStatsAPI {
                address: stats.address,
                total_buys: stats.total_buys,
                total_sells: stats.total_sells,
                total_buy_volume_token: stats.total_buy_volume_token.to_string(),
                total_sell_volume_token: stats.total_sell_volume_token.to_string(),
                total_buy_volume_usd: stats.total_buy_volume_usd.to_string(),
                total_sell_volume_usd: stats.total_sell_volume_usd.to_string(),
                total_volume_usd: total_volume.to_string(),
                net_volume_token: if net_volume >= Decimal::ZERO {
                    format!("+{}", net_volume)
                } else {
                    net_volume.to_string()
                },
                buy_sell_ratio,
            }
        })
        .collect();

    // Sort by total volume
    traders.sort_by(|a, b| {
        let a_vol: f64 = a.total_volume_usd.parse().unwrap_or(0.0);
        let b_vol: f64 = b.total_volume_usd.parse().unwrap_or(0.0);
        b_vol
            .partial_cmp(&a_vol)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply limit
    let limit = payload.limit.unwrap_or(20);
    traders.truncate(limit);

    // Calculate summary
    let total_traders = traders.len();
    let total_volume: f64 = traders
        .iter()
        .map(|t| t.total_volume_usd.parse().unwrap_or(0.0))
        .sum();
    let total_buys: u32 = traders.iter().map(|t| t.total_buys).sum();
    let total_sells: u32 = traders.iter().map(|t| t.total_sells).sum();

    let response = LeaderboardResponse {
        traders,
        summary: SummaryStats {
            total_traders,
            total_volume_usd: format!("{:.2}", total_volume),
            total_buy_transactions: total_buys,
            total_sell_transactions: total_sells,
            average_volume_per_trader: if total_traders > 0 {
                format!("{:.2}", total_volume / total_traders as f64)
            } else {
                "0.00".to_string()
            },
        },
    };

    Ok(Json(response))
}
