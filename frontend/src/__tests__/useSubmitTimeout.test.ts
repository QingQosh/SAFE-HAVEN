// ================================================================
//  useSubmitTimeout — unit tests
//
//  Tests cover:
//    • idle state on mount
//    • countdown starts and decrements via fake timers
//    • warning flag fires at the right threshold
//    • TimeoutError is thrown when deadline fires
//    • cancel() cleans up state without throwing
//    • onTimeout / onWarning monitoring callbacks fire exactly once
//    • retry: re-calling start() after a timeout resets state cleanly
//    • unmount clears timers (no act() warning)
// ================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useSubmitTimeout, TimeoutError } from '../hooks/useSubmitTimeout'

// ── helpers ─────────────────────────────────────────────────────

/** Advance fake timers by `ms` milliseconds inside act(). */
async function tick(ms: number) {
  await act(async () => {
    vi.advanceTimersByTime(ms)
  })
}

/** Flush all microtasks (Promise resolutions). */
async function flush() {
  await act(async () => {
    await Promise.resolve()
  })
}

// ── setup ────────────────────────────────────────────────────────

beforeEach(() => {
  vi.useFakeTimers()
})

afterEach(() => {
  vi.useRealTimers()
  vi.clearAllMocks()
})

// ── idle state ───────────────────────────────────────────────────

describe('useSubmitTimeout — initial state', () => {
  it('starts idle with null secondsRemaining', () => {
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 10_000, warningMs: 3_000 }),
    )
    expect(result.current.secondsRemaining).toBeNull()
    expect(result.current.isWarning).toBe(false)
    expect(result.current.isActive).toBe(false)
  })
})

// ── countdown ────────────────────────────────────────────────────

describe('useSubmitTimeout — countdown', () => {
  it('sets secondsRemaining to ceiling(timeoutMs/1000) on start', async () => {
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 10_000, warningMs: 3_000 }),
    )
    act(() => { void result.current.start() })
    expect(result.current.secondsRemaining).toBe(10)
    expect(result.current.isActive).toBe(true)
  })

  it('decrements secondsRemaining every second', async () => {
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 10_000, warningMs: 3_000 }),
    )
    act(() => { void result.current.start() })

    await tick(1_000)
    expect(result.current.secondsRemaining).toBe(9)

    await tick(1_000)
    expect(result.current.secondsRemaining).toBe(8)

    await tick(3_000)
    expect(result.current.secondsRemaining).toBe(5)
  })

  it('isActive is true while timer is running', async () => {
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 10_000, warningMs: 3_000 }),
    )
    act(() => { void result.current.start() })
    expect(result.current.isActive).toBe(true)
  })
})

// ── warning threshold ─────────────────────────────────────────────

describe('useSubmitTimeout — warning', () => {
  it('sets isWarning when warning threshold is crossed', async () => {
    const onWarning = vi.fn()
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 10_000, warningMs: 3_000, onWarning }),
    )
    act(() => { void result.current.start() })

    // Not yet warning at 6 s elapsed (4 s remaining — still above 3 s threshold)
    await tick(6_000)
    expect(result.current.isWarning).toBe(false)

    // Cross warning threshold at exactly 7 s elapsed (3 s remaining)
    await tick(1_000)
    expect(result.current.isWarning).toBe(true)
  })

  it('calls onWarning callback exactly once', async () => {
    const onWarning = vi.fn()
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 10_000, warningMs: 3_000, onWarning }),
    )
    act(() => { void result.current.start() })

    await tick(8_000)
    expect(onWarning).toHaveBeenCalledTimes(1)

    // Advancing further should NOT call it again
    await tick(1_000)
    expect(onWarning).toHaveBeenCalledTimes(1)
  })

  it('fires onWarning immediately if warningMs >= timeoutMs', async () => {
    const onWarning = vi.fn()
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 5_000, warningMs: 6_000, onWarning }),
    )
    act(() => { void result.current.start() })
    // Warning should fire immediately (warningMs > timeoutMs → warn right away)
    expect(result.current.isWarning).toBe(true)
    expect(onWarning).toHaveBeenCalledTimes(1)
  })
})

// ── timeout / deadline ────────────────────────────────────────────

describe('useSubmitTimeout — timeout deadline', () => {
  it('rejects the start() promise with TimeoutError when deadline fires', async () => {
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 5_000, warningMs: 2_000 }),
    )

    let timeoutError: unknown = null
    act(() => {
      result.current.start().catch((e) => { timeoutError = e })
    })

    await tick(5_000)
    await flush()

    expect(timeoutError).toBeInstanceOf(TimeoutError)
    expect((timeoutError as TimeoutError).name).toBe('TimeoutError')
  })

  it('calls onTimeout callback with elapsed ms when deadline fires', async () => {
    const onTimeout = vi.fn()
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 5_000, warningMs: 2_000, onTimeout }),
    )
    act(() => { result.current.start().catch(() => {}) })

    await tick(5_000)
    await flush()

    expect(onTimeout).toHaveBeenCalledTimes(1)
    // Elapsed should be >= 5000 (fake timers are deterministic)
    const [elapsed] = onTimeout.mock.calls[0] as [number]
    expect(elapsed).toBeGreaterThanOrEqual(5_000)
  })

  it('resets to idle state after timeout fires', async () => {
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 5_000, warningMs: 2_000 }),
    )
    act(() => { result.current.start().catch(() => {}) })

    await tick(5_000)
    await flush()

    expect(result.current.secondsRemaining).toBeNull()
    expect(result.current.isActive).toBe(false)
    expect(result.current.isWarning).toBe(false)
  })
})

// ── cancel ────────────────────────────────────────────────────────

describe('useSubmitTimeout — cancel', () => {
  it('resets to idle state when cancel() is called', async () => {
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 10_000, warningMs: 3_000 }),
    )
    act(() => { void result.current.start() })

    await tick(2_000)
    act(() => { result.current.cancel() })

    expect(result.current.secondsRemaining).toBeNull()
    expect(result.current.isActive).toBe(false)
    expect(result.current.isWarning).toBe(false)
  })

  it('does not call onTimeout after cancel()', async () => {
    const onTimeout = vi.fn()
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 5_000, warningMs: 2_000, onTimeout }),
    )
    act(() => { result.current.start().catch(() => {}) })
    await tick(2_000)
    act(() => { result.current.cancel() })

    // Advance past the original deadline — callback should NOT fire
    await tick(5_000)
    await flush()
    expect(onTimeout).not.toHaveBeenCalled()
  })

  it('is safe to call cancel() when not active', () => {
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 5_000, warningMs: 2_000 }),
    )
    // Should not throw
    expect(() => act(() => { result.current.cancel() })).not.toThrow()
    expect(result.current.isActive).toBe(false)
  })
})

// ── retry after timeout ───────────────────────────────────────────

describe('useSubmitTimeout — retry after timeout', () => {
  it('resets countdown when start() is called again after a timeout', async () => {
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 5_000, warningMs: 2_000 }),
    )

    // First run — let it time out
    act(() => { result.current.start().catch(() => {}) })
    await tick(5_000)
    await flush()

    expect(result.current.secondsRemaining).toBeNull()

    // Second run (retry)
    act(() => { void result.current.start() })
    expect(result.current.secondsRemaining).toBe(5)
    expect(result.current.isActive).toBe(true)
    expect(result.current.isWarning).toBe(false)
  })

  it('cancels previous timer when start() is called while active', async () => {
    const onTimeout = vi.fn()
    const { result } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 5_000, warningMs: 2_000, onTimeout }),
    )

    // Start first run
    act(() => { result.current.start().catch(() => {}) })
    await tick(2_000)

    // Start a second run while first is still active — supersedes it
    act(() => { result.current.start().catch(() => {}) })
    expect(result.current.secondsRemaining).toBe(5)

    // Let run #2 reach its deadline
    await tick(5_000)
    await flush()

    // onTimeout fires exactly once (for run #2)
    expect(onTimeout).toHaveBeenCalledTimes(1)

    // Clean up any residual timer state
    act(() => { result.current.cancel() })
  })
})

// ── unmount cleanup ───────────────────────────────────────────────

describe('useSubmitTimeout — unmount', () => {
  it('clears timers on unmount without errors', async () => {
    const onTimeout = vi.fn()
    const { result, unmount } = renderHook(() =>
      useSubmitTimeout({ timeoutMs: 10_000, warningMs: 3_000, onTimeout }),
    )
    act(() => { result.current.start().catch(() => {}) })

    // Unmount while timer is running
    await act(async () => { unmount() })

    // Advance past deadline — callback should NOT fire after unmount
    await tick(15_000)
    await flush()

    expect(onTimeout).not.toHaveBeenCalled()
  })
})

// ── TimeoutError class ────────────────────────────────────────────

describe('TimeoutError', () => {
  it('is an instance of Error', () => {
    const e = new TimeoutError()
    expect(e).toBeInstanceOf(Error)
  })

  it('has name TimeoutError', () => {
    expect(new TimeoutError().name).toBe('TimeoutError')
  })

  it('uses default message', () => {
    expect(new TimeoutError().message).toBe('Deposit submission timed out')
  })

  it('accepts a custom message', () => {
    expect(new TimeoutError('custom').message).toBe('custom')
  })
})
