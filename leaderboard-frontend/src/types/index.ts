export interface TraderStats {
  address: string;
  total_buys: number;
  total_sells: number;
  total_buy_volume_token: string;
  total_sell_volume_token: string;
  total_buy_volume_usd: string;
  total_sell_volume_usd: string;
  total_volume_usd: string;
  net_volume_token: string;
  buy_sell_ratio: number;
}

export interface LeaderboardData {
  traders: TraderStats[];
  summary: {
    total_traders: number;
    total_volume_usd: string;
    total_buy_transactions: number;
    total_sell_transactions: number;
    average_volume_per_trader: string;
  };
}

export interface TokenInfo {
  address: string;
  symbol: string;
  name: string;
  decimals: number;
}

export interface LeaderboardParams {
  token_address?: string;
  start_block?: number;
  end_block?: number;
  limit?: number;
  demo?: boolean;
  network?: string;
}
