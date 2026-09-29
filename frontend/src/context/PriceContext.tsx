/**
 * Price Context — Global price state management
 * Provides cached prices and refresh status to all components
 */

import { createContext, useCallback, useEffect, useRef, useState } from 'react'
import { fetchTokenPrices, type PriceCache, type TokenPrice } from '../lib/priceService'

export interface PriceContextType {
  // Cached prices: tokenId → { usd, lastUpdated }
  prices: PriceCache
  // Last time any price was refreshed
  lastRefresh: number
  // Is currently fetching prices
  isLoading: boolean
  // Error from last fetch attempt (cleared on success)
  error: string | null
  // Get price for a token, or null if not cached/failed
  getPrice: (tokenId: string) => TokenPrice | null
  // Manually trigger a price refresh
  refresh: (tokenIds?: string[]) => Promise<void>
}

export const PriceContext = createContext<PriceContextType | null>(null)

interface PriceProviderProps {
  children: React.ReactNode
  refreshIntervalMs?: number // Default 60_000 (1 minute)
  tokenIds?: string[] // Tokens to watch/refresh
}

export function PriceProvider({
  children,
  refreshIntervalMs = 60_000,
  tokenIds = ['native'],
}: PriceProviderProps) {
  const [prices, setPrices] = useState<PriceCache>({})
  const [lastRefresh, setLastRefresh] = useState(0)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)

  const refresh = useCallback(async (tokensToFetch = tokenIds) => {
    if (!tokensToFetch.length) return

    setIsLoading(true)
    setError(null)

    try {
      const newPrices = await fetchTokenPrices(tokensToFetch)

      // Only update prices that were successfully fetched
      // Keep old prices for failed fetches (graceful degradation)
      setPrices((prev) => ({
        ...prev,
        ...(newPrices as PriceCache),
      }))

      setLastRefresh(Date.now())
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch prices')
    } finally {
      setIsLoading(false)
    }
  }, [tokenIds])

  // Auto-refresh on interval
  useEffect(() => {
    // Initial fetch
    void refresh()

    // Set up interval
    intervalRef.current = setInterval(() => {
      void refresh()
    }, refreshIntervalMs)

    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current)
    }
  }, [refresh, refreshIntervalMs])

  const getPrice = useCallback((tokenId: string): TokenPrice | null => {
    return prices[tokenId] ?? null
  }, [prices])

  const value: PriceContextType = {
    prices,
    lastRefresh,
    isLoading,
    error,
    getPrice,
    refresh,
  }

  return <PriceContext.Provider value={value}>{children}</PriceContext.Provider>
}
