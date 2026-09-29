// ============================================================
//  Hook: load all deposits for the connected wallet
// ============================================================

import { useCallback, useEffect, useRef, useState } from 'react'
import { getDepositIds, getDepositBatch, getLedgerTime, getVault, getTimeRemaining } from '../lib/stellar'
import type { Deposit } from '../types'

// How often to re-anchor all countdowns against chain time (ms).
const RESYNC_INTERVAL_MS = 30_000

interface UseDepositsResult {
  deposits: Deposit[]
  loading: boolean
  error: string | null
  refresh: () => void
  pollRemoveDeposit: (depositId: number, maxAttempts?: number) => Promise<void>
}

export function useDeposits(depositorAddress: string | null): UseDepositsResult {
  const [deposits, setDeposits] = useState<Deposit[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const abortRef = useRef<AbortController | null>(null)

  // Track which deposit IDs currently have an in-flight chain verification so
  // we never fire two concurrent getTimeRemaining() calls for the same deposit.
  const verifyingRef = useRef<Set<number>>(new Set())

  const refresh = useCallback(async () => {
    if (!depositorAddress) {
      setDeposits([])
      return
    }

    // Cancel any in-flight fetch
    abortRef.current?.abort()
    const ctrl = new AbortController()
    abortRef.current = ctrl

    setLoading(true)
    setError(null)

    try {
      const ids = await getDepositIds(depositorAddress)
      if (ctrl.signal.aborted) return

      // Get current ledger time for computing timeRemaining
      const now = await getLedgerTime()
      if (ctrl.signal.aborted) return

      // Fetch all vault entries in batches (max 25 per call)
      const batchSize = 25
      const allDeposits: Deposit[] = []

      for (let i = 0; i < ids.length; i += batchSize) {
        const batch = ids.slice(i, i + batchSize)
        const results = await getDepositBatch(depositorAddress, batch)
        if (ctrl.signal.aborted) return

        for (let j = 0; j < results.length; j++) {
          const { entry } = results[j]
          if (entry) {
            const timeRemaining = Math.max(0, entry.unlockTime - now)
            allDeposits.push({
              ...entry,
              depositId: batch[j],
              timeRemaining,
              // Data came straight from a fresh getLedgerTime() call — already
              // chain-authoritative, so no re-verification needed.
              unlockVerified: true,
            })
          }
        }
      }

      if (ctrl.signal.aborted) return
      setDeposits(allDeposits)
    } catch (e) {
      if (ctrl.signal.aborted) return
      setError(e instanceof Error ? e.message : 'Failed to load deposits')
    } finally {
      if (!ctrl.signal.aborted) setLoading(false)
    }
  }, [depositorAddress])

  /**
   * Re-anchor all countdown timers against the current chain time.
   * Called every RESYNC_INTERVAL_MS. Uses a single getLedgerTime() RPC call
   * regardless of how many deposits are loaded.
   *
   * Any deposit whose recomputed timeRemaining is 0 but whose previous value
   * was > 0 gets `unlockVerified: false` so that a per-deposit chain query
   * is triggered by the effect below.
   */
  const resync = useCallback(async () => {
    if (!depositorAddress) return

    try {
      const now = await getLedgerTime()

      setDeposits((prev) =>
        prev.map((d) => {
          const recomputed = Math.max(0, d.unlockTime - now)
          const justUnlocked = recomputed === 0 && (d.timeRemaining ?? 1) > 0
          return {
            ...d,
            timeRemaining: recomputed,
            // If this resync is what pushed the deposit to 0, require a chain
            // confirmation before showing the Withdraw button.
            unlockVerified: justUnlocked ? false : d.unlockVerified,
          }
        }),
      )
    } catch (e) {
      // Resync is best-effort; a failed call is not fatal.
      console.warn('useDeposits resync failed:', e)
    }
  }, [depositorAddress])

  /**
   * Confirm a single deposit's unlock status against the chain.
   * Sets unlockVerified: true once getTimeRemaining() returns 0,
   * or corrects timeRemaining if the chain says it's still locked.
   */
  const verifyUnlock = useCallback(async (depositId: number) => {
    if (!depositorAddress) return
    if (verifyingRef.current.has(depositId)) return
    verifyingRef.current.add(depositId)

    try {
      const remaining = await getTimeRemaining(depositorAddress, depositId)
      setDeposits((prev) =>
        prev.map((d) => {
          if (d.depositId !== depositId) return d
          if (remaining === 0) {
            // Chain confirms unlocked — show the Withdraw button.
            return { ...d, timeRemaining: 0, unlockVerified: true }
          }
          // Chain says still locked — correct the local countdown and keep
          // the Withdraw button hidden (unlockVerified stays false until the
          // next ticker-to-0 transition).
          return { ...d, timeRemaining: remaining, unlockVerified: false }
        }),
      )
    } catch (e) {
      console.warn(`verifyUnlock(${depositId}) failed:`, e)
      // Leave unlockVerified: false; the next resync or ticker event will retry.
    } finally {
      verifyingRef.current.delete(depositId)
    }
  }, [depositorAddress])

  /**
   * Poll a single deposit until it is removed (getVault returns null),
   * then remove it from local state immediately.
   * Polls with exponential backoff: 500ms, 1s, 2s, 4s, 8s.
   */
  const pollRemoveDeposit = useCallback(async (depositId: number, maxAttempts = 5) => {
    if (!depositorAddress) return

    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      // Exponential backoff: 500ms * 2^attempt
      const delayMs = 500 * Math.pow(2, attempt)
      await new Promise((r) => setTimeout(r, delayMs))

      try {
        const vault = await getVault(depositorAddress, depositId)
        if (vault === null) {
          // Deposit removed on-chain, remove from local state immediately
          setDeposits((prev) => prev.filter((d) => d.depositId !== depositId))
          return
        }
      } catch (e) {
        // Retry on network error
        console.error(`pollRemoveDeposit(${depositId}) attempt ${attempt + 1} failed:`, e)
      }
    }

    // If polling failed, fall back to full refresh
    console.warn(`pollRemoveDeposit(${depositId}) exhausted attempts, falling back to full refresh`)
    await refresh()
  }, [depositorAddress, refresh])

  // Auto-refresh on address change
  useEffect(() => {
    void refresh()
    return () => abortRef.current?.abort()
  }, [refresh])

  // 30-second resync: re-anchor all countdowns to chain time.
  useEffect(() => {
    const id = setInterval(() => { void resync() }, RESYNC_INTERVAL_MS)
    return () => clearInterval(id)
  }, [resync])

  // 1-second ticker: smooth UI countdown.
  // When a deposit ticks down to exactly 0, mark it as unverified so the
  // per-deposit chain confirmation below is triggered.
  useEffect(() => {
    const id = setInterval(() => {
      setDeposits((prev) =>
        prev.map((d) => {
          if (d.timeRemaining === null || d.timeRemaining === 0) return d
          const next = d.timeRemaining - 1
          if (next === 0) {
            // Just hit 0 locally — require chain confirmation before enabling
            // the Withdraw button.
            return { ...d, timeRemaining: 0, unlockVerified: false }
          }
          return { ...d, timeRemaining: next }
        }),
      )
    }, 1000)
    return () => clearInterval(id)
  }, [])

  // Chain-verification effect: fires verifyUnlock() for every deposit that has
  // timeRemaining === 0 but unlockVerified === false.
  useEffect(() => {
    for (const d of deposits) {
      if (d.timeRemaining === 0 && !d.unlockVerified) {
        void verifyUnlock(d.depositId)
      }
    }
  }, [deposits, verifyUnlock])

  return { deposits, loading, error, refresh, pollRemoveDeposit }
}
