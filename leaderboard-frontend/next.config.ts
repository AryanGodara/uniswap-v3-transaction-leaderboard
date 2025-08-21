import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  env: {
    // Hardcoded API key for The Graph Network
    NEXT_PUBLIC_GRAPH_API_KEY: 'e945e8b23d8af7b0f249e0a260e6768d',
    NEXT_PUBLIC_UNISWAP_SUBGRAPH_URL: 'https://gateway.thegraph.com/api/e945e8b23d8af7b0f249e0a260e6768d/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV',
    RUST_BACKEND_URL: 'http://localhost:3001'
  }
};

export default nextConfig;
