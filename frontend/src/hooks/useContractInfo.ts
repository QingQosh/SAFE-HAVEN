// ============================================================
//  Hook: load contract-level info (admin, paused, constants)
// ============================================================

import { useCallback, useEffect, useState, useRef } from 'react'
import { getAdmin, isPaused, getConstants, getStorageVersion, getDepositorCount, getFeeRecipient, getPendingAdmin } from '../lib/stellar'

interface ContractInfo {
  admin: string | null
  pendingAdmin: string | null
  paused: boolean
  maxDeposit: bigint
  maxLockSecs: number
  version: number | null
  depositorCount: number
  feeRecipient: string | null
  loading: boolean
  refresh: () => void
}

export function useContractInfo(): ContractInfo {
  const [admin,          setAdmin]          = useState<string | null>(null)
  const [pendingAdmin,   setPendingAdmin]   = useState<string | null>(null)
  const [paused,         setPaused]         = useState(false)
  const [maxDeposit,     setMaxDeposit]     = useState<bigint>(1_000_000_000_000_000n)
  const [maxLockSecs,    setMaxLockSecs]    = useState(157_788_000)
  const [version,        setVersion]        = useState<number | null>(null)
  const [depositorCount, setDepositorCount] = useState(0)
  const [feeRecipient,   setFeeRecipient]   = useState<string | null>(null)
  const [loading,        setLoading]        = useState(true)
  const prevPausedRef = useRef<boolean>(false)

  const refresh = useCallback(async () => {
    setLoading(true)
    try {
      const [adminVal, pendingAdminVal, pausedVal, constants, storageVersion, count, fee] = await Promise.all([
        getAdmin(),
        getPendingAdmin(),
        isPaused(),
        getConstants(),
        getStorageVersion(),
        getDepositorCount(),
        getFeeRecipient(),
      ])
      setAdmin(adminVal)
      setPendingAdmin(pendingAdminVal)
      setPaused(pausedVal)
      setVersion(storageVersion)
      if (constants) {
        setMaxDeposit(constants.maxDeposit)
        setMaxLockSecs(constants.maxLockSecs)
      }
      setDepositorCount(count)
      setFeeRecipient(fee)
    } catch (e) {
      console.error('Failed to load contract info:', e)
    } finally {
      setLoading(false)
    }
  }, [])

  // Initial fetch on mount
  useEffect(() => { void refresh() }, [refresh])

  // Set up 30-second polling interval for pause state
  useEffect(() => {
    const intervalId = setInterval(() => {
      void refresh()
    }, 30_000) // 30 seconds

    return () => clearInterval(intervalId)
  }, [refresh])

  return {
    admin, pendingAdmin, paused, maxDeposit, maxLockSecs, version, depositorCount, feeRecipient, loading, refresh,
  }
}
