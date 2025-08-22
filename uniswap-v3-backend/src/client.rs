use anyhow::{Result, anyhow};
use reqwest::Client;
use std::env;

use crate::config::{Config, NetworkConfig};
use crate::types::{GraphQLQuery, GraphQLResponse, Swap, SwapsResponse};

pub struct UniswapClient {
    client: Client,
    subgraph_url: String,
    network: String,
    config: Config,
}

impl UniswapClient {
    pub fn new(network: &str) -> Result<Self> {
        let network_config = NetworkConfig::get(network)?;
        let config = Config::from_env()?;

        let subgraph_url = env::var("UNISWAP_SUBGRAPH_URL").unwrap_or_else(|_| {
            format!(
                "https://gateway.thegraph.com/api/{}/subgraphs/id/{}",
                config.graph_api_key, network_config.subgraph_id
            )
        });

        println!("Using {} network ({})", network_config.name, network);

        Ok(Self {
            client: Client::new(),
            subgraph_url,
            network: network.to_string(),
            config,
        })
    }

    pub async fn fetch_swaps(
        &self,
        token_address: &str,
        start_block: Option<u64>,
        end_block: Option<u64>,
        skip: usize,
        first: usize,
    ) -> Result<Vec<Swap>> {
        // Validate token address format
        let token_lower = token_address.to_lowercase();
        if !token_lower.starts_with("0x") || token_lower.len() != 42 {
            return Err(anyhow!(
                "Invalid token address format. Expected 42-character hex string starting with '0x'"
            ));
        }

        // Check if it's a valid hex string
        if !token_lower[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(anyhow!(
                "Invalid token address. Contains non-hexadecimal characters."
            ));
        }

        // Query with token filtering to get swaps for specific token
        let query = format!(
            r#"
            query GetSwaps($skip: Int!, $first: Int!) {{
                swaps(
                    skip: $skip,
                    first: $first,
                    orderBy: timestamp,
                    orderDirection: desc,
                    where: {{
                        or: [
                            {{ pool_: {{ token0: "{}" }} }},
                            {{ pool_: {{ token1: "{}" }} }}
                        ]
                    }}
                ) {{
                    id
                    timestamp
                    sender
                    recipient
                    amount0
                    amount1
                    amountUSD
                    pool {{
                        id
                        token0 {{
                            id
                            symbol
                            name
                            decimals
                        }}
                        token1 {{
                            id
                            symbol
                            name
                            decimals
                        }}
                        tick
                        sqrtPrice
                    }}
                    transaction {{
                        blockNumber
                    }}
                }}
            }}
            "#,
            token_lower, token_lower
        );

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

        // Check if response is successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("HTTP error {}: {}", status, error_text));
        }

        // Get response text first to debug parsing issues
        let response_text = response.text().await?;

        // Try to parse as JSON, with better error handling
        let graphql_response: GraphQLResponse<SwapsResponse> = match serde_json::from_str(
            &response_text,
        ) {
            Ok(parsed) => parsed,
            Err(parse_err) => {
                eprintln!("Failed to parse response as JSON: {}", parse_err);
                eprintln!(
                    "Response body (first 500 chars): {}",
                    if response_text.len() > 500 {
                        &response_text[..500]
                    } else {
                        &response_text
                    }
                );

                // Check if it's an HTML error page
                if response_text.trim_start().starts_with("<!DOCTYPE html>")
                    || response_text.trim_start().starts_with("<html")
                {
                    return Err(anyhow!(
                        "Received HTML error page instead of JSON. The token address '{}' might not exist or have any pools on Uniswap V3.",
                        token_address
                    ));
                }

                return Err(anyhow!(
                    "Failed to parse API response for token '{}': {}",
                    token_address,
                    parse_err
                ));
            }
        };

        if let Some(errors) = graphql_response.errors {
            let error_messages: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
            let combined_errors = error_messages.join(", ");

            // Provide helpful guidance for common errors
            if combined_errors.contains("auth") || combined_errors.contains("authorization") {
                return Err(anyhow!(
                    "Authentication error: The Graph Network endpoint authentication failed.\n\
                    \n\
                    The API key may have expired or have rate limits.\n\
                    Try demo mode instead: {} --demo --limit 10",
                    std::env::args()
                        .next()
                        .unwrap_or("uni-leaderboard".to_string())
                ));
            } else if combined_errors.contains("subgraph not found") {
                return Err(anyhow!(
                    "Subgraph not found error: The subgraph endpoint may be incorrect.\n\
                    \n\
                    The subgraph ID may be outdated.\n\
                    Try demo mode instead: {} --demo --limit 10\n\
                    \n\
                    Original error: {}",
                    std::env::args()
                        .next()
                        .unwrap_or("uni-leaderboard".to_string()),
                    combined_errors
                ));
            } else {
                return Err(anyhow!("GraphQL errors: {}", combined_errors));
            }
        }

        match graphql_response.data {
            Some(data) => {
                // Apply block range filtering if specified
                let filtered_swaps: Vec<Swap> = data
                    .swaps
                    .into_iter()
                    .filter(|swap| {
                        if let Ok(block_num) = swap.transaction.block_number.parse::<u64>() {
                            let in_start_range =
                                start_block.map_or(true, |start| block_num >= start);
                            let in_end_range = end_block.map_or(true, |end| block_num <= end);
                            in_start_range && in_end_range
                        } else {
                            true // Include if we can't parse block number
                        }
                    })
                    .collect();
                Ok(filtered_swaps)
            }
            None => Ok(vec![]),
        }
    }

    pub async fn fetch_all_swaps(
        &self,
        token_address: &str,
        _start_block: Option<u64>, // Ignored - we'll get latest swaps
        _end_block: Option<u64>,   // Ignored - we'll get latest swaps
    ) -> Result<Vec<Swap>> {
        let mut all_swaps = Vec::new();
        let mut skip = 0;

        println!("Fetching latest swap data from Uniswap v3 subgraph...");
        println!("Network: {}", self.network);
        println!("Looking for token: {}", token_address);
        println!("Target: {} latest swaps", self.config.target_swaps);

        loop {
            let swaps = self
                .fetch_swaps(token_address, None, None, skip, self.config.batch_size)
                .await?;

            if swaps.is_empty() {
                if all_swaps.is_empty() {
                    println!(
                        "No swaps found for token {}. This could mean:",
                        token_address
                    );
                    println!("  • Token has no recent trading activity");
                    println!(
                        "  • Token address is incorrect or doesn't exist on {}",
                        self.network
                    );
                    println!("  • Token is not traded on Uniswap v3 on {}", self.network);
                    println!(
                        "  • Try switching networks (use --network arbitrum/ethereum/polygon)"
                    );
                    println!("  • Try with a more active token");
                }
                break;
            }

            println!(
                "Fetched {} swaps (total: {})",
                swaps.len(),
                all_swaps.len() + swaps.len()
            );

            let batch_len = swaps.len();
            all_swaps.extend(swaps);

            // Stop if we hit our target or got less than a full batch
            if all_swaps.len() >= self.config.target_swaps || batch_len < self.config.batch_size {
                break;
            }

            skip += self.config.batch_size;
        }

        println!(
            "Total swaps fetched: {} (latest swaps from {} network)",
            all_swaps.len(),
            self.network
        );
        Ok(all_swaps)
    }
}
