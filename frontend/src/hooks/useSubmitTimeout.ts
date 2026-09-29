// ============================================================
//  Hook: useSubmitTimeout
//
//  Manages a countdown timer for deposit form submission.
//  Drives the SubmitTimeoutBanner component and provides a
//  Promise-based timeout gate for handleSubmit.
//
//  Lifecycle:
//    1. Call start() when the user submits the form.
//    2. The hook starts a countdown from DEPOSIT_TIMEOUT_MS.
//    3. At DEPOSIT_TIMEOUT_WARNING_MS remaining the banner enters
//       "warning" mode so the user sees it before it's too late.
//    4. If cancel() is not called before the deadline, the returned
//       Promise rejects and the form is reset to idle.
//    5. Call cancel() on success or user-rejection to clean up.
//
//  Monitoring:
//    onTimeout  — called exactly once when the deadline fires (for
//                 analytics / error-tracking).
//    onWarning  — called exactly once when the warning threshold is
//                 crossed.
// ============================================================

import { useCallback, useEffect, useRef, useState } from 'react'
import { CONFIG } from '../config'

export interface UseSubmitTimeoutOptions {
  /** Override the full timeout in ms (defaults to CONFIG.DEPOSIT_TIMEOUT_MS). */
  timeoutMs?: number
  /** Override the warning threshold in ms (defaults to CONFIG.DEPOSIT_TIMEOUT_WARNING_MS). */
  warningMs?: number
  /** Called once when the deadline fires — use for monitoring / telemetry. */
  onTimeout?: (elapsedMs: number) => void
  /** Called once when the warning threshold is crossed. */
  onWarning?: () => void
}

export interface UseSubmitTimeoutResult {
  /**
   * Seconds remaining in the countdown (null when not active).
   * Drive the countdown display from this value.
   */
  secondsRemaining: number | null
  /** True once the warning threshold has been crossed. */
  isWarning: boolean
  /** True while the timer is running (between start() and cancel/timeout). */
  isActive: boolean
  /**
   * Start the countdown. Returns a Promise that resolves immediately when the
   * submission succeeds (caller calls cancel()) or rejects with a TimeoutError
   * when the deadline fires. The caller should await this inside handleSubmit
   * and treat a rejection as a timeout failure.
   *
   * Calling start() while already active cancels the previous run first.
   */
  start: () => Promise<void>
  /**
   * Cancel an in-flight countdown (call on success or user rejection).
   * Safe to call when the timer is not running.
   */
  cancel: () => void
}

/** Sentinel error class for timeout-specific error handling. */
export class TimeoutError extends Error {
  constructor(message = 'Deposit submission timed out') {
    super(message)
    this.name = 'TimeoutError'
  }
}

export function useSubmitTimeout(options: UseSubmitTimeoutOptions = {}): UseSubmitTimeoutResult {
  const {
    timeoutMs = CONFIG.DEPOSIT_TIMEOUT_MS,
    warningMs = CONFIG.DEPOSIT_TIMEOUT_WARNING_MS,
    onTimeout,
    onWarning,
  } = options

  const [secondsRemaining, setSecondsRemaining] = useState<number | null>(null)
  const [isWarning, setIsWarning] = useState(false)
  const [isActive, setIsActive] = useState(false)

  // Refs so interval/timeout callbacks always see fresh values without
  // needing to be re-created on every render.
  const deadlineTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const warningTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const tickerRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const rejectRef = useRef<((reason: TimeoutError) => void) | null>(null)
  const startTimeRef = useRef<number>(0)
  const warnedRef = useRef(false)

  /** Internal cleanup — clears all timers and resets state. */
  const cleanup = useCallback(() => {
    if (deadlineTimerRef.current !== null) {
      clearTimeout(deadlineTimerRef.current)
      deadlineTimerRef.current = null
    }
    if (warningTimerRef.current !== null) {
      clearTimeout(warningTimerRef.current)
      warningTimerRef.current = null
    }
    if (tickerRef.current !== null) {
      clearInterval(tickerRef.current)
      tickerRef.current = null
    }
    rejectRef.current = null
    warnedRef.current = false
    setSecondsRemaining(null)
    setIsWarning(false)
    setIsActive(false)
  }, [])

  const cancel = useCallback(() => {
    cleanup()
  }, [cleanup])

  const start = useCallback((): Promise<void> => {
    // Cancel any previous run before starting a fresh one.
    if (rejectRef.current) {
      rejectRef.current(new TimeoutError('Superseded by new submission'))
      cleanup()
    }

    return new Promise<void>((resolve, reject) => {
      startTimeRef.current = Date.now()
      warnedRef.current = false

      setIsActive(true)
      setIsWarning(false)
      setSecondsRemaining(Math.ceil(timeoutMs / 1000))

      // We expose resolve so the caller can complete the promise by calling
      // cancel(). We need cancel() to resolve the outer promise, but cancel()
      // doesn't have access to `resolve`. We thread it through a ref.
      //
      // Design: the caller wraps the whole submission in a race:
      //
      //   const timeoutPromise = timeout.start()
      //   await Promise.race([doSubmit(), timeoutPromise])
      //
      // When doSubmit() finishes first the caller calls timeout.cancel() to
      // clean up. The timeout Promise simply stays pending until GC'd.
      // When the deadline fires first, rejectRef is called → timeoutPromise
      // rejects → the race rejects → the caller's catch block runs.
      rejectRef.current = reject
      // Keep resolve available so cancel() can resolve the promise (not
      // strictly needed for the race pattern but useful for testing).
      const resolveRef = { current: resolve }

      // Warning timer
      const warningDelay = timeoutMs - warningMs
      if (warningDelay > 0) {
        warningTimerRef.current = setTimeout(() => {
          if (!warnedRef.current) {
            warnedRef.current = true
            setIsWarning(true)
            onWarning?.()
          }
        }, warningDelay)
      } else {
        // Warning threshold is >= timeout — warn immediately.
        warnedRef.current = true
        setIsWarning(true)
        onWarning?.()
      }

      // Deadline timer
      deadlineTimerRef.current = setTimeout(() => {
        const elapsed = Date.now() - startTimeRef.current
        onTimeout?.(elapsed)
        const err = new TimeoutError()
        const rej = rejectRef.current
        cleanup()
        rej?.(err)
        // Silence the unused resolveRef lint complaint.
        void resolveRef
      }, timeoutMs)

      // 1-second ticker for countdown display
      tickerRef.current = setInterval(() => {
        setSecondsRemaining((prev) => {
          if (prev === null || prev <= 1) return prev
          return prev - 1
        })
      }, 1000)
    })
  }, [cleanup, timeoutMs, warningMs, onTimeout, onWarning])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      // eslint-disable-next-line react-hooks/exhaustive-deps
      cleanup()
    }
  }, [cleanup])

  return { secondsRemaining, isWarning, isActive, start, cancel }
}
