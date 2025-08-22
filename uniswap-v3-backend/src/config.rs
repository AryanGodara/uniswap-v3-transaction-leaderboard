use anyhow::{Result, anyhow};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub graph_api_key: String,
    pub default_port: u16,
    pub default_limit: usize,
    pub target_swaps: usize,
    pub batch_size: usize,
    pub allowed_origins: Vec<String>,
    pub server_host: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            graph_api_key: env::var("GRAPH_API_KEY")
                .unwrap_or_else(|_| "e945e8b23d8af7b0f249e0a260e6768d".to_string()),
            default_port: env::var("DEFAULT_PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .map_err(|_| anyhow!("Invalid DEFAULT_PORT value"))?,
            default_limit: env::var("DEFAULT_LIMIT")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .map_err(|_| anyhow!("Invalid DEFAULT_LIMIT value"))?,
            target_swaps: env::var("TARGET_SWAPS")
                .unwrap_or_else(|_| "2000".to_string())
                .parse()
                .map_err(|_| anyhow!("Invalid TARGET_SWAPS value"))?,
            batch_size: env::var("BATCH_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .map_err(|_| anyhow!("Invalid BATCH_SIZE value"))?,
            allowed_origins: env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
        })
    }
}

#[derive(Debug)]
pub struct NetworkConfig {
    pub subgraph_id: &'static str,
    pub default_start_block_offset: u64,
    pub name: &'static str,
}

impl NetworkConfig {
    pub fn get(network: &str) -> Result<Self> {
        match network.to_lowercase().as_str() {
            "ethereum" | "mainnet" => Ok(Self {
                subgraph_id: "5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV",
                default_start_block_offset: 216_000, // ~30 days for Ethereum
                name: "Ethereum",
            }),
            "arbitrum" => Ok(Self {
                subgraph_id: "FbCGRftH4a3yZugY7TnbYgPJVEv2LvMT6oF1fxPe9aJM",
                default_start_block_offset: 2_160_000, // ~30 days for Arbitrum
                name: "Arbitrum One",
            }),
            "polygon" => Ok(Self {
                subgraph_id: "3hCPRGf4z88VC5rsBKU5AA9FBBq5nF3jbKJG7VZCbhjm",
                default_start_block_offset: 1_296_000, // ~30 days for Polygon
                name: "Polygon",
            }),
            "optimism" => Ok(Self {
                subgraph_id: "Cghf4LfVqPiFw6fp6Y5X5Ubc8UpmUhSfJL82zwiBFLaj",
                default_start_block_offset: 432_000, // ~30 days for Optimism
                name: "Optimism",
            }),
            "base" => Ok(Self {
                subgraph_id: "43Hwfi3dJSoGpyas9VkK2E9DiKpweh7jijkRBhWGwHJK",
                default_start_block_offset: 432_000, // ~30 days for Base
                name: "Base",
            }),
            _ => Err(anyhow!(
                "Unsupported network: {}. Supported networks: ethereum, arbitrum, polygon, optimism, base",
                network
            )),
        }
    }
}
