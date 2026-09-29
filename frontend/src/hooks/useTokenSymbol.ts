// ============================================================
//  Hook: fetch and cache a SAC token's symbol
// ============================================================

import { useEffect, useState } from 'react'
import { getTokenSymbol } from '../lib/stellar'
import { CONFIG } from '../config'

interface UseTokenSymbolResult {
  /** The resolved symbol (e.g. "USDC"), or null while loading / on error */
  symbol: string | null
  /** True while the RPC call is in-flight */
  loading: boolean
}

/**
 * Returns the `symbol()` of a SAC token contract.
 *
 * - Returns `"XLM"` immediately for the native token without an RPC call.
 * - Delegates to `getTokenSymbol()` which caches results by address, so
 *   repeated calls for the same token across the component tree are free.
 * - Returns `null` (with `loading: false`) when `tokenAddress` is falsy.
 */
export function useTokenSymbol(tokenAddress: string | null | undefined): UseTokenSymbolResult {
  const [symbol, setSymbol]   = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (!tokenAddress) {
      setSymbol(null)
      setLoading(false)
      return
    }

    // Native token: no RPC call needed
    if (tokenAddress === CONFIG.NATIVE_TOKEN) {
      setSymbol('XLM')
      setLoading(false)
      return
    }

    let cancelled = false
    setLoading(true)
    setSymbol(null)

    getTokenSymbol(tokenAddress).then((result) => {
      if (cancelled) return
      setSymbol(result)
      setLoading(false)
    })

    return () => { cancelled = true }
  }, [tokenAddress])

  return { symbol, loading }
}
