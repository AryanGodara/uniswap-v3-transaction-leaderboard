use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphQLQuery {
    pub query: String,
    pub variables: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SwapsResponse {
    pub swaps: Vec<Swap>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Swap {
    pub id: String,
    pub timestamp: String,
    pub sender: String,
    pub recipient: String,
    #[serde(rename = "amount0")]
    pub amount_0: String,
    #[serde(rename = "amount1")]
    pub amount_1: String,
    #[serde(rename = "amountUSD")]
    pub amount_usd: String,
    pub pool: Pool,
    pub transaction: Transaction,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Pool {
    pub id: String,
    #[serde(rename = "token0")]
    pub token_0: Token,
    #[serde(rename = "token1")]
    pub token_1: Token,
    #[serde(rename = "tick")]
    pub tick: Option<String>,
    #[serde(rename = "sqrtPrice")]
    pub sqrt_price: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Token {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Transaction {
    #[serde(rename = "blockNumber")]
    pub block_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraderStats {
    pub address: String,
    pub total_buys: u32,
    pub total_sells: u32,
    pub total_buy_volume_token: Decimal,
    pub total_sell_volume_token: Decimal,
    pub total_buy_volume_usd: Decimal,
    pub total_sell_volume_usd: Decimal,
}

impl TraderStats {
    pub fn new(address: String) -> Self {
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

    pub fn total_volume_usd(&self) -> Decimal {
        self.total_buy_volume_usd + self.total_sell_volume_usd
    }

    pub fn net_volume_token(&self) -> Decimal {
        self.total_buy_volume_token - self.total_sell_volume_token
    }
}

// API types for HTTP server
#[derive(Debug, Deserialize)]
pub struct LeaderboardRequest {
    pub token_address: Option<String>,
    pub start_block: Option<u64>,
    pub end_block: Option<u64>,
    pub limit: Option<usize>,
    pub demo: Option<bool>,
    pub network: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub traders: Vec<TraderStatsAPI>,
    pub summary: SummaryStats,
}

#[derive(Debug, Serialize)]
pub struct TraderStatsAPI {
    pub address: String,
    pub total_buys: u32,
    pub total_sells: u32,
    pub total_buy_volume_token: String,
    pub total_sell_volume_token: String,
    pub total_buy_volume_usd: String,
    pub total_sell_volume_usd: String,
    pub total_volume_usd: String,
    pub net_volume_token: String,
    pub buy_sell_ratio: f64,
}

#[derive(Debug, Serialize)]
pub struct SummaryStats {
    pub total_traders: usize,
    pub total_volume_usd: String,
    pub total_buy_transactions: u32,
    pub total_sell_transactions: u32,
    pub average_volume_per_trader: String,
}
