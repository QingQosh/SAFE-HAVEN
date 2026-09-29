// ============================================================
//  SubmitTimeoutBanner
//
//  Displayed while a deposit submission is in-flight.
//  Shows a countdown, escalates to a warning when <30 s remain,
//  and switches to a "timed out" error state with a retry button.
//
//  Props:
//    secondsRemaining  — from useSubmitTimeout, null when inactive
//    isWarning         — true once warning threshold is crossed
//    timedOut          — true after the submission has timed out
//    onRetry           — called when user clicks "Try again"
//    onDismiss         — called when user clicks "Dismiss" (optional)
// ============================================================

interface SubmitTimeoutBannerProps {
  /** Seconds remaining (null = timer not running, 0 = deadline hit). */
  secondsRemaining: number | null
  /** True once the warning threshold (30 s) has been crossed. */
  isWarning: boolean
  /** True after the timeout has fired and the submission failed. */
  timedOut: boolean
  /** Re-submit with the same form data. */
  onRetry: () => void
  /** Hide the banner without retrying (optional). */
  onDismiss?: () => void
}

/** Format seconds as m:ss (e.g. "1:05") or bare seconds (e.g. "45 s"). */
function formatCountdown(seconds: number): string {
  if (seconds >= 60) {
    const m = Math.floor(seconds / 60)
    const s = seconds % 60
    return `${m}:${String(s).padStart(2, '0')}`
  }
  return `${seconds} s`
}

export function SubmitTimeoutBanner({
  secondsRemaining,
  isWarning,
  timedOut,
  onRetry,
  onDismiss,
}: SubmitTimeoutBannerProps) {
  // ── Timed-out state ──────────────────────────────────────────
  if (timedOut) {
    return (
      <div
        role="alert"
        aria-live="assertive"
        className="rounded-xl border border-red-700/50 bg-red-950/40 px-4 py-3 text-sm"
      >
        <div className="flex items-start gap-3">
          <span className="mt-0.5 text-red-400 text-base leading-none select-none" aria-hidden>⏱</span>
          <div className="flex-1 min-w-0">
            <p className="font-medium text-red-300">Submission timed out</p>
            <p className="mt-0.5 text-red-400/80">
              The network didn't respond in time. Your wallet wasn't charged — you can try again safely.
            </p>
            <div className="mt-3 flex items-center gap-3">
              <button
                type="button"
                onClick={onRetry}
                className="inline-flex items-center gap-1.5 rounded-lg bg-red-600 hover:bg-red-500 active:bg-red-700 px-3 py-1.5 text-xs font-semibold text-white transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-400"
              >
                <svg viewBox="0 0 20 20" fill="currentColor" className="w-3.5 h-3.5" aria-hidden>
                  <path fillRule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clipRule="evenodd" />
                </svg>
                Try again
              </button>
              {onDismiss && (
                <button
                  type="button"
                  onClick={onDismiss}
                  className="text-xs text-red-400/70 hover:text-red-300 transition-colors"
                >
                  Dismiss
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    )
  }

  // ── Not active ───────────────────────────────────────────────
  if (secondsRemaining === null) return null

  // ── Warning state (< 30 s remaining) ────────────────────────
  if (isWarning) {
    return (
      <div
        role="status"
        aria-live="polite"
        aria-label={`Submission warning: ${secondsRemaining} seconds remaining`}
        className="rounded-xl border border-orange-700/50 bg-orange-950/40 px-4 py-3 text-sm"
      >
        <div className="flex items-center gap-3">
          <span
            className="text-orange-400 text-base leading-none select-none animate-pulse"
            aria-hidden
          >
            ⚠️
          </span>
          <div className="flex-1 min-w-0">
            <p className="font-medium text-orange-300">
              Submission taking longer than expected
            </p>
            <p className="mt-0.5 text-orange-400/80">
              Will time out in{' '}
              <span className="tabular-nums font-semibold text-orange-300">
                {formatCountdown(secondsRemaining)}
              </span>
              . Keep this tab open.
            </p>
          </div>
          <CountdownRing
            seconds={secondsRemaining}
            maxSeconds={30}
            colorClass="text-orange-400"
          />
        </div>
      </div>
    )
  }

  // ── Normal in-progress state ─────────────────────────────────
  return (
    <div
      role="status"
      aria-live="polite"
      aria-label={`Submitting deposit, ${secondsRemaining} seconds remaining`}
      className="rounded-xl border border-slate-700/60 bg-slate-800/40 px-4 py-3 text-sm"
    >
      <div className="flex items-center gap-3">
        <span className="text-slate-400 text-base leading-none select-none" aria-hidden>🔒</span>
        <div className="flex-1 min-w-0">
          <p className="font-medium text-slate-300">Submitting deposit…</p>
          <p className="mt-0.5 text-slate-500">
            Timeout in{' '}
            <span className="tabular-nums font-medium text-slate-400">
              {formatCountdown(secondsRemaining)}
            </span>
          </p>
        </div>
        <CountdownRing
          seconds={secondsRemaining}
          maxSeconds={120}
          colorClass="text-stellar-400"
        />
      </div>
    </div>
  )
}

// ── Sub-component: SVG ring countdown ───────────────────────────

interface CountdownRingProps {
  seconds: number
  maxSeconds: number
  colorClass: string
}

function CountdownRing({ seconds, maxSeconds, colorClass }: CountdownRingProps) {
  const radius = 14
  const circumference = 2 * Math.PI * radius
  const progress = Math.max(0, Math.min(1, seconds / maxSeconds))
  const strokeDashoffset = circumference * (1 - progress)

  return (
    <svg
      width="40"
      height="40"
      viewBox="0 0 40 40"
      className={`shrink-0 -rotate-90 ${colorClass}`}
      aria-hidden
    >
      {/* Track */}
      <circle
        cx="20"
        cy="20"
        r={radius}
        fill="none"
        stroke="currentColor"
        strokeWidth="2.5"
        className="opacity-15"
      />
      {/* Progress arc */}
      <circle
        cx="20"
        cy="20"
        r={radius}
        fill="none"
        stroke="currentColor"
        strokeWidth="2.5"
        strokeDasharray={circumference}
        strokeDashoffset={strokeDashoffset}
        strokeLinecap="round"
        style={{ transition: 'stroke-dashoffset 0.9s linear' }}
      />
      {/* Seconds label — re-rotate to be readable */}
      <text
        x="20"
        y="20"
        textAnchor="middle"
        dominantBaseline="central"
        fontSize="10"
        fontWeight="600"
        className="rotate-90 origin-center fill-current"
        style={{ transform: 'rotate(90deg)', transformOrigin: '20px 20px' }}
      >
        {seconds}
      </text>
    </svg>
  )
}
