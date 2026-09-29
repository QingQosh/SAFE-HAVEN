import { describe, it, expect, vi, beforeEach, beforeAll } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'

// ================================================================
//  Mock the stellar module — must be before any imports that use it
// ================================================================

const mockGetDepositIds   = vi.fn()
const mockGetDepositBatch = vi.fn()
const mockGetLedgerTime   = vi.fn()
const mockGetVault        = vi.fn()
const mockGetTimeRemaining = vi.fn()

vi.mock('../lib/stellar', () => ({
  getDepositIds:    (...args: unknown[]) => mockGetDepositIds(...args),
  getDepositBatch:  (...args: unknown[]) => mockGetDepositBatch(...args),
  getLedgerTime:    (...args: unknown[]) => mockGetLedgerTime(...args),
  getVault:         (...args: unknown[]) => mockGetVault(...args),
  getTimeRemaining: (...args: unknown[]) => mockGetTimeRemaining(...args),
}))

// Use dynamic import so the mock is applied first
let useDeposits: typeof import('../hooks/useDeposits').useDeposits

beforeAll(async () => {
  const mod = await import('../hooks/useDeposits')
  useDeposits = mod.useDeposits
})

// Helper address
const ADDR  = 'GABCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUV'
const TOKEN = 'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC'

function makeEntry(amount: bigint, unlockTime: number, penaltyBps = 0) {
  return { token: TOKEN, amount, unlockTime, depositor: ADDR, penaltyBps }
}

function makeBatchResult(id: number, entry: ReturnType<typeof makeEntry> | null) {
  return { id, entry }
}

describe('useDeposits', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  // ----------------------------------------------------------------
  //  Basic loading behaviour
  // ----------------------------------------------------------------

  it('returns empty deposits when no address is provided', () => {
    const { result } = renderHook(() => useDeposits(null))
    expect(result.current.deposits).toEqual([])
    expect(result.current.loading).toBe(false)
    expect(result.current.error).toBeNull()
  })

  it('loads deposits for a given address', async () => {
    const now = 1_700_000_000

    mockGetDepositIds.mockResolvedValueOnce([0, 1])
    mockGetLedgerTime.mockResolvedValueOnce(now)
    mockGetDepositBatch.mockResolvedValueOnce([
      makeBatchResult(0, makeEntry(1000n, now + 10_000)),
      makeBatchResult(1, makeEntry(5000n, now + 20_000, 500)),
    ])

    const { result } = renderHook(() => useDeposits(ADDR))

    expect(result.current.loading).toBe(true)

    await waitFor(() => expect(result.current.loading).toBe(false), { timeout: 5000 })

    expect(result.current.deposits).toHaveLength(2)
    expect(result.current.deposits[0].depositId).toBe(0)
    expect(result.current.deposits[0].timeRemaining).toBe(10_000)
    expect(result.current.deposits[1].depositId).toBe(1)
    expect(result.current.deposits[1].timeRemaining).toBe(20_000)
    expect(result.current.error).toBeNull()
  })

  it('handles empty deposit IDs gracefully', async () => {
    mockGetDepositIds.mockResolvedValueOnce([])

    const { result } = renderHook(() => useDeposits(ADDR))

    await waitFor(() => expect(result.current.loading).toBe(false), { timeout: 5000 })

    expect(result.current.deposits).toEqual([])
    expect(result.current.error).toBeNull()
  })

  it('handles errors gracefully', async () => {
    mockGetDepositIds.mockRejectedValueOnce(new Error('Network error'))

    const { result } = renderHook(() => useDeposits(ADDR))

    await waitFor(() => expect(result.current.loading).toBe(false), { timeout: 5000 })

    expect(result.current.error).toBe('Network error')
    expect(result.current.deposits).toEqual([])
  })

  it('filters out null entries from batch', async () => {
    const now = 1_700_000_000

    mockGetDepositIds.mockResolvedValueOnce([0, 1, 2])
    mockGetLedgerTime.mockResolvedValueOnce(now)
    mockGetDepositBatch.mockResolvedValueOnce([
      makeBatchResult(0, null),
      makeBatchResult(1, makeEntry(5000n, now + 5000)),
      makeBatchResult(2, null),
    ])

    const { result } = renderHook(() => useDeposits(ADDR))

    await waitFor(() => expect(result.current.loading).toBe(false), { timeout: 5000 })

    expect(result.current.deposits).toHaveLength(1)
    expect(result.current.deposits[0].depositId).toBe(1)
  })

  // ----------------------------------------------------------------
  //  unlockVerified initial value
  // ----------------------------------------------------------------

  it('sets unlockVerified:true for deposits that are already unlocked on load', async () => {
    const now = 1_700_000_000

    mockGetDepositIds.mockResolvedValueOnce([0])
    mockGetLedgerTime.mockResolvedValueOnce(now)
    mockGetDepositBatch.mockResolvedValueOnce([
      makeBatchResult(0, makeEntry(1000n, now - 100)), // unlock time in the past
    ])

    const { result } = renderHook(() => useDeposits(ADDR))

    await waitFor(() => expect(result.current.loading).toBe(false), { timeout: 5000 })

    const d = result.current.deposits[0]
    expect(d.timeRemaining).toBe(0)
    expect(d.unlockVerified).toBe(true)
  })

  it('sets unlockVerified:true for locked deposits on load (no verification needed yet)', async () => {
    const now = 1_700_000_000

    mockGetDepositIds.mockResolvedValueOnce([0])
    mockGetLedgerTime.mockResolvedValueOnce(now)
    mockGetDepositBatch.mockResolvedValueOnce([
      makeBatchResult(0, makeEntry(1000n, now + 10_000)),
    ])

    const { result } = renderHook(() => useDeposits(ADDR))

    await waitFor(() => expect(result.current.loading).toBe(false), { timeout: 5000 })

    expect(result.current.deposits[0].unlockVerified).toBe(true)
  })

  // ----------------------------------------------------------------
  //  Ticker sets unlockVerified:false on tick-to-0
  // ----------------------------------------------------------------

  it('marks unlockVerified:false when the 1-second ticker reaches 0', async () => {
    const now = 1_700_000_000

    mockGetDepositIds.mockResolvedValueOnce([0])
    mockGetLedgerTime.mockResolvedValueOnce(now)
    mockGetDepositBatch.mockResolvedValueOnce([
      makeBatchResult(0, makeEntry(1000n, now + 2)), // 2 seconds remaining
    ])
    // verifyUnlock() will be called when timeRemaining hits 0 — return still-locked
    // for this test so we can observe the unlockVerified:false state.
    mockGetTimeRemaining.mockResolvedValue(1)

    const { result } = renderHook(() => useDeposits(ADDR))
    await waitFor(() => expect(result.current.loading).toBe(false), { timeout: 5000 })

    // Advance 2 seconds so the ticker ticks down to 0
    await act(async () => {
      vi.advanceTimersByTime(2000)
    })

    const d = result.current.deposits[0]
    expect(d.timeRemaining).toBe(0)
    expect(d.unlockVerified).toBe(false)
  })

  // ----------------------------------------------------------------
  //  verifyUnlock chain re-confirmation
  // ----------------------------------------------------------------

  it('sets unlockVerified:true after chain confirms unlock (getTimeRemaining returns 0)', async () => {
    const now = 1_700_000_000

    mockGetDepositIds.mockResolvedValueOnce([0])
    mockGetLedgerTime.mockResolvedValueOnce(now)
    mockGetDepositBatch.mockResolvedValueOnce([
      makeBatchResult(0, makeEntry(1000n, now + 1)), // 1 second remaining
    ])
    // Chain confirms unlocked when verifyUnlock fires
    mockGetTimeRemaining.mockResolvedValue(0)

    const { result } = renderHook(() => useDeposits(ADDR))
    await waitFor(() => expect(result.current.loading).toBe(false), { timeout: 5000 })

    // Tick down to 0
    await act(async () => {
      vi.advanceTimersByTime(1000)
    })

    // Wait for the async verifyUnlock to resolve
    await waitFor(() => {
      expect(result.current.deposits[0].unlockVerified).toBe(true)
    }, { timeout: 5000 })

    expect(result.current.deposits[0].timeRemaining).toBe(0)
  })

  it('corrects timeRemaining if chain says still locked after ticker hits 0', async () => {
    const now = 1_700_000_000

    mockGetDepositIds.mockResolvedValueOnce([0])
    mockGetLedgerTime.mockResolvedValueOnce(now)
    mockGetDepositBatch.mockResolvedValueOnce([
      makeBatchResult(0, makeEntry(1000n, now + 1)),
    ])
    // Chain says still 30 seconds left (clock drift scenario)
    mockGetTimeRemaining.mockResolvedValue(30)

    const { result } = renderHook(() => useDeposits(ADDR))
    await waitFor(() => expect(result.current.loading).toBe(false), { timeout: 5000 })

    await act(async () => {
      vi.advanceTimersByTime(1000)
    })

    await waitFor(() => {
      // timeRemaining corrected to chain value; withdrawal still blocked
      expect(result.current.deposits[0].timeRemaining).toBe(30)
    }, { timeout: 5000 })

    expect(result.current.deposits[0].unlockVerified).toBe(false)
  })

  // ----------------------------------------------------------------
  //  30-second resync
  // ----------------------------------------------------------------

  it('resyncs all countdowns via getLedgerTime every 30 seconds', async () => {
    const now = 1_700_000_000

    // Initial load
    mockGetDepositIds.mockResolvedValueOnce([0])
    mockGetLedgerTime.mockResolvedValueOnce(now)
    mockGetDepositBatch.mockResolvedValueOnce([
      makeBatchResult(0, makeEntry(1000n, now + 100)),
    ])

    const { result } = renderHook(() => useDeposits(ADDR))
    await waitFor(() => expect(result.current.loading).toBe(false), { timeout: 5000 })

    expect(result.current.deposits[0].timeRemaining).toBe(100)

    // Simulate 30 seconds elapsing; also the chain time jumps by 35s (drift)
    mockGetLedgerTime.mockResolvedValueOnce(now + 35)

    await act(async () => {
      vi.advanceTimersByTime(30_000)
    })

    await waitFor(() => {
      // timeRemaining should be recomputed as unlockTime - newNow = (now+100) - (now+35) = 65
      expect(result.current.deposits[0].timeRemaining).toBe(65)
    }, { timeout: 5000 })
  })

  it('sets unlockVerified:false when resync pushes timeRemaining to 0', async () => {
    const now = 1_700_000_000

    mockGetDepositIds.mockResolvedValueOnce([0])
    mockGetLedgerTime.mockResolvedValueOnce(now)
    mockGetDepositBatch.mockResolvedValueOnce([
      makeBatchResult(0, makeEntry(1000n, now + 100)), // 100s remaining
    ])

    // Resync: chain time has jumped past the unlock point
    mockGetLedgerTime.mockResolvedValueOnce(now + 110)
    // verifyUnlock will be triggered — return confirmed unlock
    mockGetTimeRemaining.mockResolvedValue(0)

    const { result } = renderHook(() => useDeposits(ADDR))
    await waitFor(() => expect(result.current.loading).toBe(false), { timeout: 5000 })

    await act(async () => {
      vi.advanceTimersByTime(30_000)
    })

    // After resync + verifyUnlock resolves, unlockVerified should be true
    await waitFor(() => {
      expect(result.current.deposits[0].timeRemaining).toBe(0)
      expect(result.current.deposits[0].unlockVerified).toBe(true)
    }, { timeout: 5000 })
  })
})
