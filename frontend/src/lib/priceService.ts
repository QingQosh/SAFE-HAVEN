/**
 * Price Service — CoinGecko API integration
 * Fetches and caches token prices with automatic refresh
 */

export interface TokenPrice {
  usd: number
  lastUpdated: number // Unix timestamp (ms)
}

export interface PriceCache {
  [tokenId: string]: TokenPrice
}

// Map token contract addresses to CoinGecko IDs
const TOKEN_TO_COINGECKO_ID: Record<string, string> = {
  // XLM (Stellar Lumens)
  native: 'stellar',
  // Add more tokens as needed
  // Example: 'USDC contract address': 'usd-coin',
}

const COINGECKO_API_URL = 'https://api.coingecko.com/api/v3'

/**
 * Fetch price for a single token from CoinGecko
 * Returns null if fetch fails or token is unknown
 */
export async function fetchTokenPrice(tokenId: string): Promise<TokenPrice | null> {
  try {
    const coingeckoId = TOKEN_TO_COINGECKO_ID[tokenId]
    if (!coingeckoId) {
      console.warn(`No CoinGecko mapping for token: ${tokenId}`)
      return null
    }

    const response = await fetch(
      `${COINGECKO_API_URL}/simple/price?ids=${coingeckoId}&vs_currencies=usd`,
      { signal: AbortSignal.timeout(5000) }
    )

    if (!response.ok) {
      console.error(`CoinGecko API error: ${response.status} ${response.statusText}`)
      return null
    }

    const data = await response.json()
    const usd = data[coingeckoId]?.usd

    if (typeof usd !== 'number' || usd < 0) {
      console.error(`Invalid price data for ${coingeckoId}:`, data)
      return null
    }

    return {
      usd,
      lastUpdated: Date.now(),
    }
  } catch (error) {
    console.error(`Failed to fetch price for ${tokenId}:`, error)
    return null
  }
}

/**
 * Fetch prices for multiple tokens
 * Returns object with successful fetches; missing entries are failed fetches
 */
export async function fetchTokenPrices(tokenIds: string[]): Promise<Partial<PriceCache>> {
  const uniqueIds = Array.from(new Set(tokenIds))
  const results: Partial<PriceCache> = {}

  // Fetch all in parallel
  const promises = uniqueIds.map(async (id) => {
    const price = await fetchTokenPrice(id)
    if (price) {
      results[id] = price
    }
  })

  await Promise.all(promises)
  return results
}

/**
 * Check if a cached price is stale (older than 1 minute)
 */
export function isPriceStale(price: TokenPrice, maxAge = 60_000): boolean {
  return Date.now() - price.lastUpdated > maxAge
}
