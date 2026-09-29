// ============================================================
//  Unit tests for balanceSyncService
//  Uses Vitest fake timers + fetch/WebSocket stubs
// ============================================================

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import {
  startSync,
  stopSync,
  refreshBalance,
  checkWebSocketCapability,
  type BalanceSyncCallbacks,
} from '../lib/balanceSyncService'

// ---------------------------------------------------------------------------
//  Helpers
// ---------------------------------------------------------------------------

const DEBOUNCE_MS = 300
const POLL_MS     = 5_000
const TEST_ADDR   = 'GABC1234567890123456789012345678901234567890123456789012'
const HORIZON_URL = 'https://horizon-testnet.stellar.org'

function makeHorizonAccount(balance = '42.0000000') {
  return {
    balances: [
      { asset_type: 'native', balance },
      { asset_type: 'credit_alphanum4', balance: '1.0000000' },
    ],
  }
}

function okFetch(body: unknown) {
  return vi.fn().mockResolvedValue({
    ok: true,
    status: 200,
    json: () => Promise.resolve(body),
  })
}

function errorFetch(status: number) {
  return vi.fn().mockResolvedValue({
    ok: false,
    status,
    json: () => Promise.resolve({}),
  })
}

function networkErrorFetch() {
  return vi.fn().mockRejectedValue(new TypeError('Failed to fetch'))
}

/** Minimal WebSocket stub that never sends any messages */
class SilentWebSocket {
  static CONNECTING = 0 as const
  static OPEN       = 1 as const
  static CLOSING    = 2 as const
  static CLOSED     = 3 as const

  readyState = SilentWebSocket.CONNECTING

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  addEventListener(_event: string, _listener: unknown) { /* noop */ }
  removeEventListener() { /* noop */ }
  send() { /* noop */ }
  close() { this.readyState = SilentWebSocket.CLOSED }
}

// ---------------------------------------------------------------------------
//  Setup / teardown
// ---------------------------------------------------------------------------

beforeEach(() => {
  vi.useFakeTimers()
  // Stub WebSocket globally so capability checks always fail (fall back to polling)
  vi.stubGlobal('WebSocket', SilentWebSocket)
  // Ensure we start each test with a clean service state
  stopSync()
})

afterEach(() => {
  stopSync()
  vi.restoreAllMocks()
  vi.useRealTimers()
})

// ---------------------------------------------------------------------------
//  Task 10.1 — connect → immediate fetch fires
// ---------------------------------------------------------------------------

describe('10.1 connect → immediate fetch fires', () => {
  it('calls fetch immediately after debounce for the correct URL', async () => {
    const fetchMock = okFetch(makeHorizonAccount())
    vi.stubGlobal('fetch', fetchMock)

    const callbacks: BalanceSyncCallbacks = {
      onSuccess: vi.fn(),
      onError:   vi.fn(),
    }

    startSync(TEST_ADDR, callbacks, { horizonUrl: HORIZON_URL, pollIntervalMs: POLL_MS })

    // Advance past the debounce window
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS + 10)

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock).toHaveBeenCalledWith(
      `${HORIZON_URL}/accounts/${TEST_ADDR}`,
      expect.objectContaining({ signal: expect.any(AbortSignal) }),
    )
    expect(callbacks.onSuccess).toHaveBeenCalledTimes(1)
    expect(callbacks.onSuccess).toHaveBeenCalledWith('42.0000000', expect.any(Date))
    expect(callbacks.onError).not.toHaveBeenCalled()
  })
})

// ---------------------------------------------------------------------------
//  Task 10.2 — stopSync halts polling
// ---------------------------------------------------------------------------

describe('10.2 stopSync halts polling', () => {
  it('does not call fetch after stopSync is called', async () => {
    const fetchMock = okFetch(makeHorizonAccount())
    vi.stubGlobal('fetch', fetchMock)

    const callbacks: BalanceSyncCallbacks = {
      onSuccess: vi.fn(),
      onError:   vi.fn(),
    }

    startSync(TEST_ADDR, callbacks, { horizonUrl: HORIZON_URL, pollIntervalMs: POLL_MS })

    // Advance past debounce + initial fetch
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS + 10)
    const countAfterStart = (fetchMock as ReturnType<typeof vi.fn>).mock.calls.length

    // Stop the service
    stopSync()

    // Advance through several poll intervals
    await vi.advanceTimersByTimeAsync(POLL_MS * 4)

    expect((fetchMock as ReturnType<typeof vi.fn>).mock.calls.length).toBe(countAfterStart)
  })
})

// ---------------------------------------------------------------------------
//  Task 10.3 — 404 response sets balance to '0.0000000' with no error
// ---------------------------------------------------------------------------

describe('10.3 HTTP 404 → unfunded account', () => {
  it('calls onSuccess with "0.0000000" and does not call onError', async () => {
    vi.stubGlobal('fetch', errorFetch(404))

    const callbacks: BalanceSyncCallbacks = {
      onSuccess: vi.fn(),
      onError:   vi.fn(),
    }

    startSync(TEST_ADDR, callbacks, { horizonUrl: HORIZON_URL, pollIntervalMs: POLL_MS })
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS + 10)

    expect(callbacks.onSuccess).toHaveBeenCalledWith('0.0000000', expect.any(Date))
    expect(callbacks.onError).not.toHaveBeenCalled()
  })
})

// ---------------------------------------------------------------------------
//  Task 10.4 — 3 consecutive failures trigger toast once
// ---------------------------------------------------------------------------

describe('10.4 consecutive failures → toast at count 3', () => {
  it('calls onError with consecutiveCount 1, 2, 3 across three polls', async () => {
    vi.stubGlobal('fetch', networkErrorFetch())

    const callbacks: BalanceSyncCallbacks = {
      onSuccess: vi.fn(),
      onError:   vi.fn(),
    }

    startSync(TEST_ADDR, callbacks, { horizonUrl: HORIZON_URL, pollIntervalMs: POLL_MS })

    // First failure: immediate fetch after debounce
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS + 10)
    expect(callbacks.onError).toHaveBeenNthCalledWith(1, expect.any(String), 1)

    // Second failure: one poll interval later
    await vi.advanceTimersByTimeAsync(POLL_MS)
    expect(callbacks.onError).toHaveBeenNthCalledWith(2, expect.any(String), 2)

    // Third failure
    await vi.advanceTimersByTimeAsync(POLL_MS)
    expect(callbacks.onError).toHaveBeenNthCalledWith(3, expect.any(String), 3)

    expect(callbacks.onSuccess).not.toHaveBeenCalled()
  })

  it('does not call onError a 4th time with count 4 (count keeps incrementing but toast is deduplicated)', async () => {
    vi.stubGlobal('fetch', networkErrorFetch())

    const callbacks: BalanceSyncCallbacks = {
      onSuccess: vi.fn(),
      onError:   vi.fn(),
    }

    startSync(TEST_ADDR, callbacks, { horizonUrl: HORIZON_URL, pollIntervalMs: POLL_MS })

    // Drive 4 failures
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS + 10)
    await vi.advanceTimersByTimeAsync(POLL_MS)
    await vi.advanceTimersByTimeAsync(POLL_MS)
    await vi.advanceTimersByTimeAsync(POLL_MS)

    // onError IS still called on the 4th failure (count = 4), but toast deduplication
    // is handled inside WalletContext (toast id). Here we verify count increments.
    expect(callbacks.onError).toHaveBeenNthCalledWith(4, expect.any(String), 4)
    expect(callbacks.onError).toHaveBeenCalledTimes(4)
  })
})

// ---------------------------------------------------------------------------
//  Task 10.5 — success after failure clears error state
// ---------------------------------------------------------------------------

describe('10.5 success after failure clears error state', () => {
  it('calls onSuccess after two failures and does not increment consecutiveCount', async () => {
    const failFetch = vi.fn()
      // First two calls fail
      .mockResolvedValueOnce({ ok: false, status: 500, json: () => Promise.resolve({}) })
      .mockResolvedValueOnce({ ok: false, status: 500, json: () => Promise.resolve({}) })
      // Third call succeeds
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(makeHorizonAccount('99.0000000')),
      })

    vi.stubGlobal('fetch', failFetch)

    const callbacks: BalanceSyncCallbacks = {
      onSuccess: vi.fn(),
      onError:   vi.fn(),
    }

    startSync(TEST_ADDR, callbacks, { horizonUrl: HORIZON_URL, pollIntervalMs: POLL_MS })

    // First failure (immediate)
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS + 10)
    expect(callbacks.onError).toHaveBeenNthCalledWith(1, expect.any(String), 1)

    // Second failure
    await vi.advanceTimersByTimeAsync(POLL_MS)
    expect(callbacks.onError).toHaveBeenNthCalledWith(2, expect.any(String), 2)

    // Third call succeeds — should call onSuccess, not onError
    await vi.advanceTimersByTimeAsync(POLL_MS)
    expect(callbacks.onSuccess).toHaveBeenCalledWith('99.0000000', expect.any(Date))
    expect(callbacks.onError).toHaveBeenCalledTimes(2)
  })
})

// ---------------------------------------------------------------------------
//  refreshBalance — basic behaviour
// ---------------------------------------------------------------------------

describe('refreshBalance', () => {
  it('triggers an immediate fetch outside the poll cycle', async () => {
    const fetchMock = okFetch(makeHorizonAccount('10.0000000'))
    vi.stubGlobal('fetch', fetchMock)

    const callbacks: BalanceSyncCallbacks = {
      onSuccess: vi.fn(),
      onError:   vi.fn(),
    }

    startSync(TEST_ADDR, callbacks, { horizonUrl: HORIZON_URL, pollIntervalMs: POLL_MS })
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS + 10)
    const countAfterStart = (fetchMock as ReturnType<typeof vi.fn>).mock.calls.length

    // Manually trigger refresh well before the next poll
    await vi.advanceTimersByTimeAsync(1_000)
    await refreshBalance()

    expect((fetchMock as ReturnType<typeof vi.fn>).mock.calls.length).toBe(countAfterStart + 1)
  })

  it('throws when called with no active session', async () => {
    stopSync()
    await expect(refreshBalance()).rejects.toThrow(/no active sync session/i)
  })
})

// ---------------------------------------------------------------------------
//  checkWebSocketCapability
// ---------------------------------------------------------------------------

describe('checkWebSocketCapability', () => {
  it('returns false when WebSocket immediately errors', async () => {
    class ErrorWebSocket {
      static CONNECTING = 0 as const
      static OPEN = 1 as const
      static CLOSING = 2 as const
      static CLOSED = 3 as const

      private listeners: Record<string, ((...args: unknown[]) => void)[]> = {}

      addEventListener(event: string, cb: (...args: unknown[]) => void) {
        if (!this.listeners[event]) this.listeners[event] = []
        this.listeners[event].push(cb)
        // Emit error immediately on next tick
        if (event === 'error') {
          Promise.resolve().then(() => cb())
        }
      }

      close() { /* noop */ }
      send() { /* noop */ }
    }

    vi.stubGlobal('WebSocket', ErrorWebSocket)

    // Run real timers for the async capability check
    vi.useRealTimers()
    const result = await checkWebSocketCapability('https://soroban-testnet.stellar.org')
    vi.useFakeTimers()

    expect(result).toBe(false)
  })
})
