"use client";

import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Search, TrendingUp, TrendingDown, Users, DollarSign, Activity } from 'lucide-react';
import type { LeaderboardData, LeaderboardParams } from '@/types';

// Import Firebase modules
import { initializeApp } from "firebase/app";
import { getFirestore, doc, getDoc, setDoc } from "firebase/firestore";

// Your web app's Firebase configuration
const firebaseConfig = {
  apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
  authDomain: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
  projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
  storageBucket: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET,
  messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID,
  appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID
};

// Initialize Firebase and Firestore
const app = initializeApp(firebaseConfig);
const db = getFirestore(app);


const NETWORK_TOKENS: Record<string, Array<{address: string, symbol: string, name: string}>> = {
  ethereum: [
    { address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", symbol: "USDC", name: "USD Coin" },
    { address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", symbol: "WETH", name: "Wrapped Ethereum" },
    { address: "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984", symbol: "UNI", name: "Uniswap Token" },
    { address: "0x7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9", symbol: "AAVE", name: "Aave Token" },
    { address: "0x6B175474E89094C44Da98b954EedeAC495271d0F", symbol: "DAI", name: "Dai Stablecoin" },
    { address: "0xdAC17F958D2ee523a2206206994597C13D831ec7", symbol: "USDT", name: "Tether USD" }
  ],
  arbitrum: [
    { address: "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8", symbol: "USDC.e", name: "USD Coin (Arbitrum)" },
    { address: "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1", symbol: "WETH", name: "Wrapped Ethereum" },
    { address: "0xFa7F8980b0f1E64A2062791cc3b0871572f1F7f0", symbol: "UNI", name: "Uniswap Token" },
    { address: "0x912CE59144191C1204E64559FE8253a0e49E6548", symbol: "ARB", name: "Arbitrum Token" },
    { address: "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1", symbol: "DAI", name: "Dai Stablecoin" },
    { address: "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9", symbol: "USDT", name: "Tether USD" }
  ],
  polygon: [
    { address: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174", symbol: "USDC", name: "USD Coin" },
    { address: "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619", symbol: "WETH", name: "Wrapped Ethereum" },
    { address: "0xb33EaAd8d922B1083446DC23f610c2567fB5180f", symbol: "UNI", name: "Uniswap Token" },
    { address: "0xD6DF932A45C0f255f85145f286eA0b292B21C90B", symbol: "AAVE", name: "Aave Token" },
    { address: "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063", symbol: "DAI", name: "Dai Stablecoin" },
    { address: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F", symbol: "USDT", name: "Tether USD" }
  ],
  optimism: [
    { address: "0x7F5c764cBc14f9669B88837ca1490cCa17c31607", symbol: "USDC", name: "USD Coin" },
    { address: "0x4200000000000000000000000000000000000006", symbol: "WETH", name: "Wrapped Ethereum" },
    { address: "0x6fd9d7AD17242c41f7131d257212c54A0e816691", symbol: "UNI", name: "Uniswap Token" },
    { address: "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1", symbol: "DAI", name: "Dai Stablecoin" },
    { address: "0x94b008aA00579c1307B0EF2c499aD98a8ce58e58", symbol: "USDT", name: "Tether USD" },
    { address: "0x4200000000000000000000000000000000000042", symbol: "OP", name: "Optimism Token" }
  ],
  base: [
    { address: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", symbol: "USDC", name: "USD Coin" },
    { address: "0x4200000000000000000000000000000000000006", symbol: "WETH", name: "Wrapped Ethereum" },
    { address: "0xd9aAEc86B65D86f6A7B5B1b0c42FFA531710b6CA", symbol: "USDbC", name: "USD Base Coin" },
    { address: "0x50c5725949A6F0c72E6C4a641F24049A917DB0Cb", symbol: "DAI", name: "Dai Stablecoin" },
    { address: "0x2Ae3F1Ec7F1F5012CFEab0185bfc7aa3cf0DEc22", symbol: "cbETH", name: "Coinbase Wrapped Staked ETH" }
  ]
};

const NETWORKS = [
  { value: 'ethereum', label: 'Ethereum'},
  { value: 'arbitrum', label: 'Arbitrum'},
  { value: 'polygon', label: 'Polygon'},
  { value: 'optimism', label: 'Optimism'},
  { value: 'base', label: 'Base'}
];

export default function Leaderboard() {
  const [data, setData] = useState<LeaderboardData>({ 
    traders: [], 
    summary: { 
      total_traders: 0, 
      total_volume_usd: "0", 
      total_buy_transactions: 0, 
      total_sell_transactions: 0, 
      average_volume_per_trader: "0" 
    } 
  });
  const [loading, setLoading] = useState(false);
  const [params, setParams] = useState<LeaderboardParams>({
    demo: false,
    limit: 20,
    network: 'ethereum'
  });

  const formatAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const formatNumber = (num: string | number) => {
    const value = typeof num === 'string' ? Number.parseFloat(num) : num;
    if (value >= 1000000) {
      return `$${(value / 1000000).toFixed(2)}M`;
    }
    if (value >= 1000) {
      return `$${(value / 1000).toFixed(2)}K`;
    }
    return `$${value.toFixed(2)}`;
  };

  const fetchLeaderboard = async () => {
    // Validate token address format before making request
    if (params.token_address && !params.demo) {
      const tokenAddr = params.token_address.trim();
      if (!tokenAddr.startsWith('0x') || tokenAddr.length !== 42) {
        console.error('Invalid token address format');
        setData({ traders: [], summary: { total_traders: 0, total_volume_usd: "0", total_buy_transactions: 0, total_sell_transactions: 0, average_volume_per_trader: "0" } });
        setLoading(false);
        return;
      }
      
      if (!/^0x[a-fA-F0-9]{40}$/.test(tokenAddr)) {
        console.error('Invalid token address: contains non-hexadecimal characters');
        setData({ traders: [], summary: { total_traders: 0, total_volume_usd: "0", total_buy_transactions: 0, total_sell_transactions: 0, average_volume_per_trader: "0" } });
        setLoading(false);
        return;
      }
    }

    setLoading(true);

    // Create a unique ID for the firestore document based on network and token address
    const docId = `${params.network}_${params.token_address}`;
    const docRef = doc(db, "leaderboards", docId);

    try {
      // Check Firestore for cached data first
      const docSnap = await getDoc(docRef);

      if (docSnap.exists()) {
        console.log("Document data:", docSnap.data());
        setData(docSnap.data() as LeaderboardData);
      } else {
        // If no data in Firestore, fetch from API
        console.log("No such document in cache, fetching from API");
        const response = await fetch('/api/leaderboard', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(params),
        });

        if (!response.ok) {
          const errorText = await response.text();
          console.error('API Error:', response.status, errorText);
          throw new Error(`Failed to fetch leaderboard: ${response.status}`);
        }

        const leaderboardData = await response.json();
        
        if (leaderboardData.traders && leaderboardData.traders.length === 0) {
          console.warn('No traders found for this token - might have low trading volume or token might not exist on Uniswap V3');
        }
        
        // Save the new data to Firestore
        await setDoc(docRef, leaderboardData);
        console.log("Cached new data to Firestore");

        setData(leaderboardData);
      }
    } catch (error) {
      console.error('Failed to fetch leaderboard:', error);
      setData({ traders: [], summary: { total_traders: 0, total_volume_usd: "0", total_buy_transactions: 0, total_sell_transactions: 0, average_volume_per_trader: "0" } });
    } finally {
      setLoading(false);
    }
  };

  const handleTokenSelect = (tokenAddress: string) => {
    setParams({ ...params, token_address: tokenAddress, demo: false });
  };

  const handleNetworkChange = (network: string) => {
    setParams({ ...params, network, token_address: '', demo: false });
    setData({ 
      traders: [], 
      summary: { 
        total_traders: 0, 
        total_volume_usd: "0", 
        total_buy_transactions: 0, 
        total_sell_transactions: 0, 
        average_volume_per_trader: "0" 
      } 
    });
  };

  const getCurrentNetworkTokens = () => {
    return NETWORK_TOKENS[params.network || 'ethereum'] || NETWORK_TOKENS.ethereum;
  };

  const getCurrentNetworkInfo = () => {
    return NETWORKS.find(n => n.value === (params.network || 'ethereum')) || NETWORKS[0];
  };

  return (
    <div className="min-h-screen bg-background p-4 space-y-6">
      {/* Header */}
      <div className="text-center space-y-4">
        <h1 className="text-4xl font-bold text-foreground">
          UNISWAP V3 TRADER LEADERBOARD
        </h1>
        <div className="flex items-center justify-center gap-2">
          <p className="text-lg text-muted-foreground">
            Discover the top traders on 
          </p>
          <Badge variant="default" className="text-sm font-bold retro-shadow">
            {getCurrentNetworkInfo().label}
          </Badge>
        </div>
      </div>

      {/* Controls */}
      <Card className="bg-card retro-card-hover">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Search size={20} />
            Search & Filters
          </CardTitle>
          <CardDescription>
            Select a network above, then enter a token address or choose from popular tokens (works best with tokens that have recent trading activity)
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Network Selector */}
          <div className="space-y-2">
            <h4 className="text-sm font-medium">Network:</h4>
            <div className="flex flex-wrap gap-2">
              {NETWORKS.map((network) => (
                <Button
                  key={network.value}
                  variant={params.network === network.value ? "default" : "outline"}
                  size="sm"
                  onClick={() => handleNetworkChange(network.value)}
                  className={`text-xs retro-btn-shake ${params.network === network.value ? 'retro-shadow' : ''}`}
                >
                {network.label}
                </Button>
              ))}
            </div>
          </div>

          <div className="flex gap-2">
            <Input
              placeholder={`Token contract address on ${getCurrentNetworkInfo().label} (0x...)`}
              value={params.token_address || ''}
              onChange={(e) => setParams({ ...params, token_address: e.target.value, demo: false })}
              className="flex-1"
            />
            <Button 
              onClick={fetchLeaderboard} 
              disabled={loading}
              className="retro-btn-bounce font-bold"
              size="default"
            >
              {loading ? 'Loading...' : 'Analyze'}
            </Button>
          </div>
          
          <div className="space-y-2">
            <h4 className="text-sm font-medium">High-Activity Tokens on {getCurrentNetworkInfo().label}:</h4>
            <div className="flex flex-wrap gap-2">
              {getCurrentNetworkTokens().map((token) => (
                <Button
                  key={token.address}
                  variant="outline"
                  size="sm"
                  onClick={() => handleTokenSelect(token.address)}
                  className="text-xs retro-btn-shake hover:bg-yellow-100"
                  title={`${token.name} - ${token.address}`}
                >
                  {token.symbol}
                </Button>
              ))}
            </div>
          </div>


        </CardContent>
      </Card>

      {/* Summary Stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
        <Card className="retro-interactive">
          <CardContent className="p-6">
            <div className="flex items-center space-x-2">
              <Users className="h-4 w-4 text-muted-foreground" />
              <div>
                <p className="text-2xl font-bold">{data.summary.total_traders}</p>
                <p className="text-xs text-muted-foreground">Total Traders</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="retro-interactive">
          <CardContent className="p-6">
            <div className="flex items-center space-x-2">
              <DollarSign className="h-4 w-4 text-muted-foreground" />
              <div>
                <p className="text-2xl font-bold">{formatNumber(data.summary.total_volume_usd)}</p>
                <p className="text-xs text-muted-foreground">Total Volume</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="retro-interactive">
          <CardContent className="p-6">
            <div className="flex items-center space-x-2">
              <TrendingUp className="h-4 w-4 text-green-600" />
              <div>
                <p className="text-2xl font-bold">{data.summary.total_buy_transactions}</p>
                <p className="text-xs text-muted-foreground">Buy Transactions</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="retro-interactive">
          <CardContent className="p-6">
            <div className="flex items-center space-x-2">
              <TrendingDown className="h-4 w-4 text-red-600" />
              <div>
                <p className="text-2xl font-bold">{data.summary.total_sell_transactions}</p>
                <p className="text-xs text-muted-foreground">Sell Transactions</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="retro-interactive">
          <CardContent className="p-6">
            <div className="flex items-center space-x-2">
              <Activity className="h-4 w-4 text-muted-foreground" />
              <div>
                <p className="text-2xl font-bold">{formatNumber(data.summary.average_volume_per_trader)}</p>
                <p className="text-xs text-muted-foreground">Avg per Trader</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Leaderboard Table */}
      <Card className="retro-card-hover">
        <CardHeader>
          <CardTitle>Trader Rankings</CardTitle>
          <CardDescription>
            Top traders ranked by total trading volume (USD)
          </CardDescription>
        </CardHeader>
        <CardContent>
          {data.traders.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-lg text-muted-foreground mb-4">
                {data.summary.total_traders === 0 && params.token_address ? 
                  "No trading data found for this token" : 
                  "Enter a token address above to analyze trader activity"
                }
              </p>
              <p className="text-sm text-muted-foreground">
                {data.summary.total_traders === 0 && params.token_address ? 
                  `This token might have low trading volume on ${getCurrentNetworkInfo().label}. Try switching networks or using the recommended tokens above` : 
                  `Select ${getCurrentNetworkInfo().label} network and try the recommended tokens above for best results`
                }
              </p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Rank</TableHead>
                  <TableHead>Trader Address</TableHead>
                  <TableHead>Buys</TableHead>
                  <TableHead>Sells</TableHead>
                  <TableHead>Total Volume (USD)</TableHead>
                  <TableHead>Net Token Volume</TableHead>
                  <TableHead>Buy/Sell Ratio</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.traders.map((trader, index) => (
                  <TableRow key={trader.address}>
                    <TableCell>
                      <Badge variant={index < 3 ? "default" : "secondary"}>
                        #{index + 1}
                      </Badge>
                    </TableCell>
                    <TableCell className="font-mono">{formatAddress(trader.address)}</TableCell>
                    <TableCell>
                      <Badge variant="secondary" className="bg-green-100 text-green-800">
                        {trader.total_buys}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <Badge variant="secondary" className="bg-red-100 text-red-800">
                        {trader.total_sells}
                      </Badge>
                    </TableCell>
                    <TableCell className="font-bold">{formatNumber(trader.total_volume_usd)}</TableCell>
                    <TableCell>
                      <span className={trader.net_volume_token.startsWith('+') ? 'text-green-600' : 'text-red-600'}>
                        {trader.net_volume_token}
                      </span>
                    </TableCell>
                    <TableCell>
                      <Badge variant={trader.buy_sell_ratio > 1 ? "default" : "outline"}>
                        {trader.buy_sell_ratio.toFixed(2)}
                      </Badge>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Footer */}
      <div className="text-center text-sm text-muted-foreground">
        <p>Powered by Uniswap v3 Subgraph • Built with RetroUI</p>
      </div>
    </div>
  );
}
