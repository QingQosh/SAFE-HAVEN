// ============================================================
//  VersionWarningBanner
//
//  Displays a version compatibility warning at the top of the app.
//
//  Severity rules:
//    'ok'    → renders nothing
//    'warn'  → yellow dismissible banner
//    'error' → red non-dismissible banner
//
//  Shows the live contract version, expected version, and human-readable
//  warning. If a migration guide is available, it is shown in a
//  collapsible <details> block.
// ============================================================

import { useState } from 'react'
import {
  EXPECTED_CONTRACT_VERSION,
  EXPECTED_STORAGE_VERSION,
  getMigrationGuide,
  type VersionCheckResult,
} from '../lib/versionCompat'

interface VersionWarningBannerProps {
  result: VersionCheckResult
}

export function VersionWarningBanner({ result }: VersionWarningBannerProps) {
  const [dismissed, setDismissed] = useState(false)

  // Nothing to show when compatible.
  if (result.severity === 'ok') return null

  // Warn banners can be dismissed; error banners cannot.
  if (result.severity === 'warn' && dismissed) return null

  // Determine whether there is a migration guide to show.
  const showMigrationGuide =
    result.contractVersion !== null &&
    result.contractVersion !== EXPECTED_CONTRACT_VERSION

  const migrationGuide = showMigrationGuide
    ? getMigrationGuide(EXPECTED_CONTRACT_VERSION, result.contractVersion!)
    : null

  if (result.severity === 'warn') {
    return (
      <WarnBanner
        result={result}
        migrationGuide={migrationGuide}
        onDismiss={() => setDismissed(true)}
      />
    )
  }

  // severity === 'error'
  return (
    <ErrorBanner
      result={result}
      migrationGuide={migrationGuide}
    />
  )
}

// ----------------------------------------------------------------
//  WarnBanner (yellow, dismissible)
// ----------------------------------------------------------------

interface BannerProps {
  result: VersionCheckResult
  migrationGuide: string | null
  onDismiss?: () => void
}

function WarnBanner({ result, migrationGuide, onDismiss }: BannerProps) {
  return (
    <div
      role="alert"
      aria-live="polite"
      className="border-b border-yellow-700/50 bg-yellow-950/40"
    >
      <div className="max-w-6xl mx-auto px-4 py-3">
        <div className="flex items-start gap-3">
          {/* Icon */}
          <span
            className="mt-0.5 shrink-0 text-yellow-400"
            aria-hidden="true"
          >
            <svg
              viewBox="0 0 20 20"
              fill="currentColor"
              className="w-5 h-5"
            >
              <path
                fillRule="evenodd"
                d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 9a1 1 0 100-2 1 1 0 000 2z"
                clipRule="evenodd"
              />
            </svg>
          </span>

          {/* Content */}
          <div className="flex-1 min-w-0 text-sm">
            <p className="font-semibold text-yellow-300">
              Contract version mismatch
            </p>
            <VersionDetails result={result} />
            {result.warning && (
              <p className="mt-1 text-yellow-400/90 text-xs leading-relaxed">
                {result.warning}
              </p>
            )}
            {migrationGuide && (
              <MigrationGuideDetails guide={migrationGuide} colorClass="text-yellow-300" borderClass="border-yellow-700/40" bgClass="bg-yellow-950/60" />
            )}
          </div>

          {/* Dismiss button */}
          {onDismiss && (
            <button
              type="button"
              onClick={onDismiss}
              aria-label="Dismiss version warning"
              className="shrink-0 rounded p-1 text-yellow-400/70 hover:text-yellow-300 hover:bg-yellow-900/40 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-yellow-400"
            >
              <svg viewBox="0 0 20 20" fill="currentColor" className="w-4 h-4" aria-hidden="true">
                <path d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" />
              </svg>
            </button>
          )}
        </div>
      </div>
    </div>
  )
}

// ----------------------------------------------------------------
//  ErrorBanner (red, non-dismissible)
// ----------------------------------------------------------------

function ErrorBanner({ result, migrationGuide }: BannerProps) {
  return (
    <div
      role="alert"
      aria-live="assertive"
      className="border-b border-red-700/50 bg-red-950/40"
    >
      <div className="max-w-6xl mx-auto px-4 py-3">
        <div className="flex items-start gap-3">
          {/* Icon */}
          <span
            className="mt-0.5 shrink-0 text-red-400"
            aria-hidden="true"
          >
            <svg
              viewBox="0 0 20 20"
              fill="currentColor"
              className="w-5 h-5"
            >
              <path
                fillRule="evenodd"
                d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-5a.75.75 0 01.75.75v4.5a.75.75 0 01-1.5 0v-4.5A.75.75 0 0110 5zm0 10a1 1 0 100-2 1 1 0 000 2z"
                clipRule="evenodd"
              />
            </svg>
          </span>

          {/* Content */}
          <div className="flex-1 min-w-0 text-sm">
            <p className="font-semibold text-red-300">
              Storage schema incompatibility — action required
            </p>
            <VersionDetails result={result} />
            {result.warning && (
              <p className="mt-1 text-red-400/90 text-xs leading-relaxed">
                {result.warning}
              </p>
            )}
            {migrationGuide && (
              <MigrationGuideDetails guide={migrationGuide} colorClass="text-red-300" borderClass="border-red-700/40" bgClass="bg-red-950/60" />
            )}
          </div>

          {/* No dismiss button — error banners are sticky */}
        </div>
      </div>
    </div>
  )
}

// ----------------------------------------------------------------
//  Shared sub-components
// ----------------------------------------------------------------

function VersionDetails({ result }: { result: VersionCheckResult }) {
  return (
    <div className="mt-1 flex flex-wrap gap-x-4 gap-y-0.5 text-xs font-mono text-slate-400">
      <span>
        contract:{' '}
        <span className="text-slate-200">
          {result.contractVersion ?? 'unknown'}
        </span>
      </span>
      <span>
        expected:{' '}
        <span className="text-slate-200">{EXPECTED_CONTRACT_VERSION}</span>
      </span>
      {result.storageVersion !== null && (
        <span>
          storage:{' '}
          <span className="text-slate-200">v{result.storageVersion}</span>
          {' '}(expected v{EXPECTED_STORAGE_VERSION})
        </span>
      )}
    </div>
  )
}

interface MigrationGuideDetailsProps {
  guide: string
  colorClass: string
  borderClass: string
  bgClass: string
}

function MigrationGuideDetails({
  guide,
  colorClass,
  borderClass,
  bgClass,
}: MigrationGuideDetailsProps) {
  return (
    <details className="mt-2 group">
      <summary
        className={`cursor-pointer select-none text-xs font-medium ${colorClass} hover:opacity-80 transition-opacity list-none flex items-center gap-1`}
      >
        <svg
          viewBox="0 0 20 20"
          fill="currentColor"
          className="w-3.5 h-3.5 transition-transform group-open:rotate-90"
          aria-hidden="true"
        >
          <path
            fillRule="evenodd"
            d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
            clipRule="evenodd"
          />
        </svg>
        Migration guide
      </summary>
      <pre
        className={`mt-1.5 rounded-lg border ${borderClass} ${bgClass} p-3 text-xs text-slate-300 whitespace-pre-wrap font-mono leading-relaxed overflow-x-auto`}
      >
        {guide}
      </pre>
    </details>
  )
}
