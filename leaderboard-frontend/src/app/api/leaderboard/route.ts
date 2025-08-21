import { NextRequest, NextResponse } from 'next/server';
import { LeaderboardParams, LeaderboardData, TraderStats } from '@/types';

// Hardcoded API key for The Graph Network
const GRAPH_API_KEY = 'e945e8b23d8af7b0f249e0a260e6768d';
const UNISWAP_SUBGRAPH_URL = `https://gateway.thegraph.com/api/${GRAPH_API_KEY}/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV`;

interface SwapData {
  id: string;
  timestamp: string;
  sender: string;
  recipient: string;
  amount0: string;
  amount1: string;
  amountUSD: string;
  pool: {
    id: string;
    token0: {
      id: string;
      symbol: string;
      name: string;
      decimals: string;
    };
    token1: {
      id: string;
      symbol: string;
      name: string;
      decimals: string;
    };
  };
  transaction: {
    blockNumber: string;
  };
}

async function fetchLeaderboardFromGraph(params: LeaderboardParams): Promise<LeaderboardData> {
  if (!params.token_address) {
    throw new Error('Token address is required');
  }

  const query = `
    query GetSwaps($first: Int!, $skip: Int!) {
      swaps(
        first: $first,
        skip: $skip,
        orderBy: timestamp,
        orderDirection: desc,
        where: {
          or: [
            { pool_: { token0: "${params.token_address.toLowerCase()}" } },
            { pool_: { token1: "${params.token_address.toLowerCase()}" } }
          ]
        }
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
        }
        transaction {
          blockNumber
        }
      }
    }
  `;

  const allSwaps: SwapData[] = [];
  let skip = 0;
  const batchSize = 1000;

  // Fetch swaps in batches
  while (allSwaps.length < 10000) { // Limit to prevent timeout
    const response = await fetch(UNISWAP_SUBGRAPH_URL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        query,
        variables: { first: batchSize, skip }
      }),
    });

    if (!response.ok) {
      throw new Error(`Graph API error: ${response.status}`);
    }

    const result = await response.json();
    
    if (result.errors) {
      throw new Error(`GraphQL errors: ${result.errors.map((e: { message: string }) => e.message).join(', ')}`);
    }

    const swaps = result.data?.swaps || [];
    
    if (swaps.length === 0) {
      break;
    }

    allSwaps.push(...swaps);
    skip += batchSize;

    if (swaps.length < batchSize) {
      break; // Last batch
    }
  }

  // Process swaps and aggregate trader stats
  const traderStatsMap = new Map<string, {
    address: string;
    total_buys: number;
    total_sells: number;
    total_buy_volume_token: number;
    total_sell_volume_token: number;
    total_buy_volume_usd: number;
    total_sell_volume_usd: number;
  }>();

  for (const swap of allSwaps) {
    const tokenAddress = params.token_address!.toLowerCase();
    const amount0 = parseFloat(swap.amount0);
    const amount1 = parseFloat(swap.amount1);
    const amountUSD = parseFloat(swap.amountUSD);
    
    let isBuy = false;
    let tokenAmount = 0;
    
    if (swap.pool.token0.id.toLowerCase() === tokenAddress) {
      // Target token is token0
      isBuy = amount0 < 0; // Negative means tokens going into pool (buy)
      tokenAmount = Math.abs(amount0);
    } else if (swap.pool.token1.id.toLowerCase() === tokenAddress) {
      // Target token is token1
      isBuy = amount1 < 0; // Negative means tokens going into pool (buy)
      tokenAmount = Math.abs(amount1);
    } else {
      continue; // Skip if target token not found
    }

    const trader = swap.sender;
    
    if (!traderStatsMap.has(trader)) {
      traderStatsMap.set(trader, {
        address: trader,
        total_buys: 0,
        total_sells: 0,
        total_buy_volume_token: 0,
        total_sell_volume_token: 0,
        total_buy_volume_usd: 0,
        total_sell_volume_usd: 0,
      });
    }

    const stats = traderStatsMap.get(trader)!;
    
    if (isBuy) {
      stats.total_buys++;
      stats.total_buy_volume_token += tokenAmount;
      stats.total_buy_volume_usd += amountUSD;
    } else {
      stats.total_sells++;
      stats.total_sell_volume_token += tokenAmount;
      stats.total_sell_volume_usd += amountUSD;
    }
  }

  // Convert to array and sort by total volume
  const traders: TraderStats[] = Array.from(traderStatsMap.values())
    .map(stats => ({
      address: stats.address,
      total_buys: stats.total_buys,
      total_sells: stats.total_sells,
      total_buy_volume_token: stats.total_buy_volume_token.toFixed(4),
      total_sell_volume_token: stats.total_sell_volume_token.toFixed(4),
      total_buy_volume_usd: stats.total_buy_volume_usd.toFixed(2),
      total_sell_volume_usd: stats.total_sell_volume_usd.toFixed(2),
      total_volume_usd: (stats.total_buy_volume_usd + stats.total_sell_volume_usd).toFixed(2),
      net_volume_token: (stats.total_buy_volume_token - stats.total_sell_volume_token >= 0 ? '+' : '') + 
                       (stats.total_buy_volume_token - stats.total_sell_volume_token).toFixed(4),
      buy_sell_ratio: stats.total_sells > 0 ? stats.total_buys / stats.total_sells : stats.total_buys,
    }))
    .sort((a, b) => parseFloat(b.total_volume_usd) - parseFloat(a.total_volume_usd))
    .slice(0, params.limit || 20);

  // Calculate summary
  const totalTraders = traders.length;
  const totalVolumeUsd = traders.reduce((sum, t) => sum + parseFloat(t.total_volume_usd), 0);
  const totalBuys = traders.reduce((sum, t) => sum + t.total_buys, 0);
  const totalSells = traders.reduce((sum, t) => sum + t.total_sells, 0);

  return {
    traders,
    summary: {
      total_traders: totalTraders,
      total_volume_usd: totalVolumeUsd.toFixed(2),
      total_buy_transactions: totalBuys,
      total_sell_transactions: totalSells,
      average_volume_per_trader: totalTraders > 0 ? (totalVolumeUsd / totalTraders).toFixed(2) : "0.00",
    },
  };
}

export async function POST(request: NextRequest) {
  try {
    const params: LeaderboardParams = await request.json();

    // For demo mode, return mock data
    if (params.demo) {
      const demoData: LeaderboardData = {
        traders: [
          {
            address: "0x5678901234567890123456789012345678901234",
            total_buys: 89,
            total_sells: 76,
            total_buy_volume_token: "3456.7890",
            total_sell_volume_token: "2345.6789",
            total_buy_volume_usd: "245000.75",
            total_sell_volume_usd: "198000.25",
            total_volume_usd: "443001.00",
            net_volume_token: "+1111.1101",
            buy_sell_ratio: 1.17
          },
          {
            address: "0x7890123456789012345678901234567890123456",
            total_buys: 78,
            total_sells: 43,
            total_buy_volume_token: "2789.0123",
            total_sell_volume_token: "1234.5678",
            total_buy_volume_usd: "189000.25",
            total_sell_volume_usd: "87500.50",
            total_volume_usd: "276500.75",
            net_volume_token: "+1554.4445",
            buy_sell_ratio: 1.81
          },
          {
            address: "0x1234567890123456789012345678901234567890",
            total_buys: 45,
            total_sells: 32,
            total_buy_volume_token: "1234.5678",
            total_sell_volume_token: "987.1234",
            total_buy_volume_usd: "125000.50",
            total_sell_volume_usd: "98000.25",
            total_volume_usd: "223000.75",
            net_volume_token: "+247.4444",
            buy_sell_ratio: 1.41
          },
          {
            address: "0x6789012345678901234567890123456789012345",
            total_buys: 34,
            total_sells: 56,
            total_buy_volume_token: "1234.5678",
            total_sell_volume_token: "1567.8901",
            total_buy_volume_usd: "89000.50",
            total_sell_volume_usd: "112000.75",
            total_volume_usd: "201001.25",
            net_volume_token: "-333.3223",
            buy_sell_ratio: 0.61
          }
        ],
        summary: {
          total_traders: 4,
          total_volume_usd: "1143504.75",
          total_buy_transactions: 246,
          total_sell_transactions: 207,
          average_volume_per_trader: "285876.19"
        }
      };

      return NextResponse.json(demoData);
    }

    // For real data, call the Rust backend
    const rustBackendUrl = process.env.RUST_BACKEND_URL || 'http://localhost:3001';
    
    try {
      const response = await fetch(`${rustBackendUrl}/api/leaderboard`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(params),
        signal: AbortSignal.timeout(90000), // 90 second timeout for large datasets
      });

      if (!response.ok) {
        throw new Error(`Backend error: ${response.status}`);
      }

      const data = await response.json();
      return NextResponse.json(data);
    } catch (backendError) {
      console.error('Failed to connect to Rust backend:', backendError);
      
      // Fallback to direct Graph API if backend is down
      console.log('Falling back to direct Graph API...');
      try {
        const leaderboardData = await fetchLeaderboardFromGraph(params);
        return NextResponse.json(leaderboardData);
      } catch (graphError) {
        console.error('Both backend and direct API failed:', graphError);
        
        // Return an error response with helpful message
        return NextResponse.json(
          {
            error: 'Backend connection failed',
            message: `Could not connect to the Rust backend at ${rustBackendUrl}. Direct API fallback also failed.`,
            suggestion: 'Try using demo mode instead or ensure the backend server is running.',
            backendUrl: rustBackendUrl,
            timestamp: new Date().toISOString()
          },
          { status: 503 }
        );
      }
    }
  } catch (error) {
    console.error('API error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
