// ============================================================
//  Formatting helpers
// ============================================================

import { formatDistanceToNow, format, fromUnixTime, isPast } from 'date-fns'
import { CONFIG } from '../config'

/**
 * Convert stroops → XLM string, e.g. 10_000_000n → "1.0000000"
 * Assumes XLM has 7 decimal places (the Stellar native token standard).
 */
export function stroopsToXlm(stroops: bigint): string {
  const whole = stroops / BigInt(CONFIG.STROOPS_PER_XLM)
  const frac  = stroops % BigInt(CONFIG.STROOPS_PER_XLM)
  const fracStr = frac.toString().padStart(7, '0')
  return `${whole}.${fracStr}`
}

/**
 * Convert XLM string → stroops bigint
 * Assumes XLM has 7 decimal places (the Stellar native token standard).
 */
export function xlmToStroops(xlm: string): bigint {
  const [whole, frac = ''] = xlm.split('.')
  const fracPadded = frac.padEnd(7, '0').slice(0, 7)
  return BigInt(whole) * BigInt(CONFIG.STROOPS_PER_XLM) + BigInt(fracPadded)
}

/**
 * Convert a decimal-based amount string to token base units.
 * Handles arbitrary decimal precisions.
 *
 * @param amount - Human-readable amount (e.g., "1.5" USDC)
 * @param decimals - Number of decimal places for the token (e.g., 6 for USDC)
 * @returns Amount in token base units as bigint
 *
 * Example:
 *   amountToBaseUnits("1.5", 6) → 1500000n (1.5 USDC with 6 decimals)
 *   amountToBaseUnits("100", 8) → 10000000000n (100 of an 8-decimal token)
 */
export function amountToBaseUnits(amount: string, decimals: number): bigint {
  if (!amount || decimals < 0) return 0n
  try {
    const [whole, frac = ''] = amount.split('.')
    const divisor = BigInt(10 ** decimals)
    const fracPadded = frac.padEnd(decimals, '0').slice(0, decimals)
    return BigInt(whole) * divisor + BigInt(fracPadded)
  } catch {
    return 0n
  }
}

/**
 * Convert token base units to a decimal-based amount string.
 * Handles arbitrary decimal precisions.
 *
 * @param baseUnits - Amount in token base units
 * @param decimals - Number of decimal places for the token
 * @returns Human-readable amount string
 *
 * Example:
 *   baseUnitsToAmount(1500000n, 6) → "1.500000"
 *   baseUnitsToAmount(10000000000n, 8) → "100.00000000"
 */
export function baseUnitsToAmount(baseUnits: bigint, decimals: number): string {
  if (decimals < 0) return baseUnits.toString()
  const divisor = BigInt(10 ** decimals)
  const whole = baseUnits / divisor
  const frac = baseUnits % divisor
  const fracStr = frac.toString().padStart(decimals, '0')
  return `${whole}.${fracStr}`
}

/** Format a Unix timestamp as a human-readable date+time string */
export function formatUnlockDate(unixSecs: number): string {
  return format(fromUnixTime(unixSecs), 'MMM d, yyyy HH:mm')
}

/** Format a Unix timestamp as a relative string like "in 3 days" */
export function formatRelativeTime(unixSecs: number): string {
  const date = fromUnixTime(unixSecs)
  if (isPast(date)) return 'Unlocked'
  return formatDistanceToNow(date, { addSuffix: true })
}

/** Format seconds duration into "2d 4h 30m 10s" */
export function formatCountdown(seconds: number | null): string {
  if (seconds === null || seconds <= 0) return 'Unlocked'
  const d = Math.floor(seconds / 86400)
  const h = Math.floor((seconds % 86400) / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  const parts: string[] = []
  if (d > 0) parts.push(`${d}d`)
  if (h > 0) parts.push(`${h}h`)
  if (m > 0) parts.push(`${m}m`)
  if (d === 0) parts.push(`${s}s`) // show seconds only when < 1 day
  return parts.join(' ')
}

/** Format basis points as a percentage string */
export function formatBps(bps: number): string {
  return `${(bps / 100).toFixed(2)}%`
}

/** Shorten a Stellar address for display */
export function shortAddr(addr: string, chars = 6): string {
  if (addr.length <= chars * 2 + 1) return addr
  return `${addr.slice(0, chars)}…${addr.slice(-4)}`
}

/** Explorer URL for a transaction */
export function explorerTxUrl(txHash: string): string {
  return `${CONFIG.EXPLORER_URL}/tx/${txHash}`
}

/** Explorer URL for an address */
export function explorerAddrUrl(addr: string): string {
  return `${CONFIG.EXPLORER_URL}/account/${addr}`
}

/**
 * Format a duration in seconds into human-readable form.
 * Examples: "2 years, 3 months", "45 days, 6 hours", "30 minutes"
 */
export function formatDuration(seconds: number): string {
  if (seconds <= 0) return '0 seconds'

  const units = [
    { label: 'year', seconds: 365 * 24 * 3600 },
    { label: 'month', seconds: 30 * 24 * 3600 },
    { label: 'day', seconds: 24 * 3600 },
    { label: 'hour', seconds: 3600 },
    { label: 'minute', seconds: 60 },
    { label: 'second', seconds: 1 },
  ]

  const parts: string[] = []
  let remaining = Math.floor(seconds)

  for (const unit of units) {
    const count = Math.floor(remaining / unit.seconds)
    if (count > 0) {
      parts.push(`${count} ${unit.label}${count > 1 ? 's' : ''}`)
      remaining %= unit.seconds
      if (parts.length === 2) break // stop after 2 units (e.g., "2 years, 3 months")
    }
  }

  return parts.length > 0 ? parts.join(', ') : '0 seconds'
}

/**
 * Convert a datetime-local input string to a Unix timestamp (seconds).
 * The datetime-local input returns a string like "2026-07-28T15:30" which
 * represents the user's local time. We must convert it to UTC first before
 * calculating the Unix timestamp, accounting for the browser's timezone offset.
 *
 * @param dateTimeLocalStr - String from datetime-local input (e.g., "2026-07-28T15:30")
 * @returns Unix timestamp in seconds (UTC)
 */
export function dateTimeLocalToUnixSeconds(dateTimeLocalStr: string): number {
  if (!dateTimeLocalStr) return 0
  const date = new Date(dateTimeLocalStr)
  // The datetime-local input is interpreted as local time, but we need to adjust
  // for the timezone offset to get the correct UTC timestamp
  const offset = date.getTimezoneOffset() * 60 * 1000 // convert to milliseconds
  const utcTime = date.getTime() + offset
  return Math.floor(utcTime / 1000)
}

/**
 * Get the user's timezone offset string (e.g., "UTC+8" or "UTC-5")
 */
export function getTimezoneOffsetString(): string {
  const offset = new Date().getTimezoneOffset()
  const hours = Math.abs(Math.floor(offset / 60))
  const minutes = Math.abs(offset % 60)
  const sign = offset <= 0 ? '+' : '-'
  const minutesPart = minutes > 0 ? `:${minutes.toString().padStart(2, '0')}` : ''
  return `UTC${sign}${hours}${minutesPart}`
}

/**
 * Format a Unix timestamp for both local and UTC display
 * Returns: "local-date local-time (UTC+8)\nUTC: utc-date utc-time"
 */
export function formatUnlockTimestampWithTimezone(unixSecs: number): { local: string; utc: string; offset: string } {
  const date = new Date(unixSecs * 1000)
  const offset = getTimezoneOffsetString()
  
  // Format local time
  const localStr = date.toLocaleString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
  
  // Format UTC time
  const utcStr = date.toUTCString().replace(' GMT', '')
  
  return {
    local: localStr,
    utc: utcStr,
    offset,
  }
}

/**
 * Generate the minimum datetime-local value for a datetime input.
 * Must be in local time (not UTC) to match what datetime-local displays.
 * Minimum is 60 seconds from now.
 */
export function getMinDateTimeLocal(): string {
  const now = new Date()
  const minDate = new Date(now.getTime() + 60_000) // 60 seconds from now
  
  // Convert to local time string in the format datetime-local expects: YYYY-MM-DDTHH:mm
  const year = minDate.getFullYear()
  const month = String(minDate.getMonth() + 1).padStart(2, '0')
  const day = String(minDate.getDate()).padStart(2, '0')
  const hours = String(minDate.getHours()).padStart(2, '0')
  const minutes = String(minDate.getMinutes()).padStart(2, '0')
  
  return `${year}-${month}-${day}T${hours}:${minutes}`
}

/** Validate if a string is a valid Stellar address (G-address or C-address) */
export function isValidStellarAddress(addr: string): boolean {
  if (!addr) return false
  
  // G-address: starts with 'G', 56 characters total, alphanumeric
  const gAddressPattern = /^G[A-Z2-7]{54}$/
  
  // C-address: starts with 'C', 56 characters total, alphanumeric
  const cAddressPattern = /^C[A-Z2-7]{54}$/
  
  return gAddressPattern.test(addr) || cAddressPattern.test(addr)
}

/**
 * Validate if a string is a valid Stellar contract address (C-address).
 * Contracts are always C-addresses: 55 characters total (C + 54 base32 chars), using base32 alphabet.
 *
 * @param addr - Potential contract address
 * @returns true if valid contract address format, false otherwise
 */
export function isValidContractAddress(addr: string): boolean {
  if (!addr) return false
  // Contract address: starts with 'C', exactly 54 more base32 characters (total 55)
  const contractPattern = /^C[A-Z2-7]{54}$/
  return contractPattern.test(addr)
}

/**
 * Validate a token contract address and return a detailed status.
 * Provides user feedback on what went wrong if validation fails.
 *
 * @param addr - Token contract address to validate
 * @returns object with valid flag and message
 */
export function validateTokenAddress(addr: string): { valid: boolean; message: string } {
  if (!addr) {
    return { valid: false, message: 'Address is required' }
  }

  if (addr.length !== 55) {
    return { valid: false, message: `Address must be 55 characters (got ${addr.length})` }
  }

  if (!addr.startsWith('C')) {
    return { valid: false, message: 'Contract addresses must start with "C"' }
  }

  if (!isValidContractAddress(addr)) {
    return { valid: false, message: 'Invalid characters in address (use A-Z and 2-7 only)' }
  }

  return { valid: true, message: 'Valid contract address format' }
}

/**
 * Format a token amount with optional USD value.
 * Example: "1.5000000 XLM ($1.89)" or "1.5000000 XLM" if no price
 */
export function formatTokenWithUsd(amount: bigint, symbol: string, priceUsd?: number): string {
  const amountStr = stroopsToXlm(amount)
  if (priceUsd !== undefined && priceUsd > 0) {
    const usdValue = parseFloat(amountStr) * priceUsd
    return `${amountStr} ${symbol} ($${usdValue.toFixed(2)})`
  }
  return `${amountStr} ${symbol}`
}

/**
 * Format when a price was last updated.
 * Example: "Updated 2 minutes ago"
 */
export function formatPriceUpdate(timestamp: number): string {
  const date = fromUnixTime(timestamp)
  return `Updated ${formatDistanceToNow(date, { addSuffix: true })}`
}
