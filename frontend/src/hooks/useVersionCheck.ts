// ============================================================
//  Hook: useVersionCheck
//
//  Fetches the live contract version and storage schema version
//  on mount, compares them against the compatibility matrix, and
//  returns a VersionCheckResult plus a loading flag.
//
//  Refreshes automatically every 5 minutes so long-running sessions
//  notice contract upgrades without a page reload.
// ============================================================

import { useCallback, useEffect, useRef, useState } from 'react'
import { getContractVersion, getStorageVersion } from '../lib/stellar'
import {
  checkVersionCompatibility,
  type VersionCheckResult,
} from '../lib/versionCompat'

/** How often (ms) to re-check the contract version. */
const REFRESH_INTERVAL_MS = 5 * 60 * 1_000 // 5 minutes

export interface UseVersionCheckResult {
  /** The compatibility check result, or null while the first fetch is in flight. */
  result: VersionCheckResult | null
  /** True while the version fetch is in progress. */
  loading: boolean
}

/**
 * Fetches contract version and storage schema version from the live contract,
 * then runs checkVersionCompatibility() to produce a VersionCheckResult.
 *
 * Gracefully degrades: if either RPC call fails, the hook still resolves with
 * a 'warn' severity rather than throwing. This keeps the UI functional even
 * when the contract is temporarily unreachable.
 */
export function useVersionCheck(): UseVersionCheckResult {
  const [result, setResult] = useState<VersionCheckResult | null>(null)
  const [loading, setLoading] = useState(true)

  // Ref to cancel stale in-flight fetches when the component unmounts or
  // when a new fetch is triggered before the previous one completes.
  const abortRef = useRef<AbortController | null>(null)

  const check = useCallback(async () => {
    // Cancel any previously in-flight fetch.
    abortRef.current?.abort()
    const ctrl = new AbortController()
    abortRef.current = ctrl

    setLoading(true)
    try {
      // Fetch both values concurrently. Neither call requires a wallet.
      const [contractVersion, storageVersion] = await Promise.all([
        getContractVersion().catch((err) => {
          console.warn('[useVersionCheck] getContractVersion() failed:', err)
          return null
        }),
        getStorageVersion().catch((err) => {
          console.warn('[useVersionCheck] getStorageVersion() failed:', err)
          return null
        }),
      ])

      // Abort guard: ignore result if this fetch was cancelled.
      if (ctrl.signal.aborted) return

      const checked = checkVersionCompatibility(contractVersion, storageVersion)
      setResult(checked)
    } catch (e) {
      // Unexpected error — degrade gracefully rather than crashing.
      if (ctrl.signal.aborted) return
      console.error('[useVersionCheck] Unexpected error during version check:', e)
      setResult(checkVersionCompatibility(null, null))
    } finally {
      if (!ctrl.signal.aborted) {
        setLoading(false)
      }
    }
  }, [])

  // Initial fetch on mount.
  useEffect(() => {
    void check()
  }, [check])

  // Periodic refresh every 5 minutes.
  useEffect(() => {
    const intervalId = setInterval(() => {
      void check()
    }, REFRESH_INTERVAL_MS)

    return () => {
      clearInterval(intervalId)
      abortRef.current?.abort()
    }
  }, [check])

  return { result, loading }
}
