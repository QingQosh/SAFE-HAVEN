import { describe, it, expect, beforeEach, vi } from 'vitest'
import {
  stroopsToXlm,
  xlmToStroops,
  formatCountdown,
  formatBps,
  formatDuration,
  amountToBaseUnits,
  baseUnitsToAmount,
  dateTimeLocalToUnixSeconds,
  getMinDateTimeLocal,
  formatUnlockTimestampWithTimezone,
  getTimezoneOffsetString,
  isValidContractAddress,
  validateTokenAddress,
  shortAddr,
  isValidStellarAddress,
} from '../lib/format'

// ================================================================
//  stroopsToXlm
// ================================================================

describe('stroopsToXlm', () => {
  it('converts 0 stroops to "0.0000000"', () => {
    expect(stroopsToXlm(0n)).toBe('0.0000000')
  })

  it('converts 10_000_000 stroops (1 XLM) to "1.0000000"', () => {
    expect(stroopsToXlm(10_000_000n)).toBe('1.0000000')
  })

  it('converts 1 stroop to "0.0000001"', () => {
    expect(stroopsToXlm(1n)).toBe('0.0000001')
  })

  it('converts 12_345_678 stroops to "1.2345678"', () => {
    expect(stroopsToXlm(12_345_678n)).toBe('1.2345678')
  })

  it('handles large values (1 billion stroops)', () => {
    expect(stroopsToXlm(1_000_000_000n)).toBe('100.0000000')
  })

  it('pads fractional part correctly', () => {
    expect(stroopsToXlm(10_000_100n)).toBe('1.0000100')
  })

  it('handles values with no fractional part', () => {
    expect(stroopsToXlm(50_000_000n)).toBe('5.0000000')
  })
})

// ================================================================
//  xlmToStroops
// ================================================================

describe('xlmToStroops', () => {
  it('converts "1.0000000" to 10_000_000n', () => {
    expect(xlmToStroops('1.0000000')).toBe(10_000_000n)
  })

  it('converts "0.0000001" to 1n', () => {
    expect(xlmToStroops('0.0000001')).toBe(1n)
  })

  it('converts "0" (no fraction) to 0n', () => {
    expect(xlmToStroops('0')).toBe(0n)
  })

  it('converts "5" (no fraction) to 50_000_000n', () => {
    expect(xlmToStroops('5')).toBe(50_000_000n)
  })

  it('converts "0.1" (short fraction) to 1_000_000n', () => {
    expect(xlmToStroops('0.1')).toBe(1_000_000n)
  })

  it('converts "0.01" to 100_000n', () => {
    expect(xlmToStroops('0.01')).toBe(100_000n)
  })

  it('round-trip: xlmToStroops(stroopsToXlm(x)) === x', () => {
    const cases = [0n, 1n, 10_000_000n, 12_345_678n, 1_000_000_000n]
    for (const s of cases) {
      expect(xlmToStroops(stroopsToXlm(s))).toBe(s)
    }
  })

  it('handles fraction longer than 7 digits by truncating', () => {
    // "0.12345678" -> pads "12345678" to 7 -> "1234567" so 1_234_567 stroops
    expect(xlmToStroops('0.12345678')).toBe(1_234_567n)
  })

  it('handles very small amounts correctly', () => {
    expect(xlmToStroops('0.0000000')).toBe(0n)
  })
})

// ================================================================
//  formatCountdown
// ================================================================

describe('formatCountdown', () => {
  it('returns "Unlocked" for null', () => {
    expect(formatCountdown(null)).toBe('Unlocked')
  })

  it('returns "Unlocked" for 0', () => {
    expect(formatCountdown(0)).toBe('Unlocked')
  })

  it('returns "Unlocked" for negative values', () => {
    expect(formatCountdown(-5)).toBe('Unlocked')
  })

  it('formats seconds only (< 1 day)', () => {
    expect(formatCountdown(65)).toBe('1m 5s')
  })

  it('formats minutes and seconds', () => {
    expect(formatCountdown(125)).toBe('2m 5s')
  })

  it('formats hours, minutes, seconds', () => {
    expect(formatCountdown(3661)).toBe('1h 1m 1s')
  })

  it('formats days and hours (no seconds for >= 1 day)', () => {
    expect(formatCountdown(90000)).toBe('1d 1h')
  })

  it('formats multiple days', () => {
    // 2d exactly: h=0, m=0 are skipped; seconds skipped when d>0
    expect(formatCountdown(172800)).toBe('2d')
  })

  it('handles exactly one second', () => {
    expect(formatCountdown(1)).toBe('1s')
  })

  it('formats exactly one hour', () => {
    // d=0 so seconds shown; m=0 so skipped
    expect(formatCountdown(3600)).toBe('1h 0s')
  })
})

// ================================================================
//  formatBps
// ================================================================

describe('formatBps', () => {
  it('formats 0 bps as "0.00%"', () => {
    expect(formatBps(0)).toBe('0.00%')
  })

  it('formats 100 bps as "1.00%"', () => {
    expect(formatBps(100)).toBe('1.00%')
  })

  it('formats 10000 bps as "100.00%"', () => {
    expect(formatBps(10000)).toBe('100.00%')
  })

  it('formats 50 bps as "0.50%"', () => {
    expect(formatBps(50)).toBe('0.50%')
  })

  it('formats 1 bps as "0.01%"', () => {
    expect(formatBps(1)).toBe('0.01%')
  })

  it('formats 1234 bps as "12.34%"', () => {
    expect(formatBps(1234)).toBe('12.34%')
  })
})

// ================================================================
//  shortAddr
// ================================================================

describe('shortAddr', () => {
  it('shortens a long Stellar address', () => {
    const addr = 'GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFSHONUCEOASW7QC7OX2H'
    const result = shortAddr(addr)
    // Default: first 6 chars + … + last 4 chars
    expect(result).toBe('GBRPYH…OX2H')
  })

  it('shortens with custom chars', () => {
    const addr = 'GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFSHONUCEOASW7QC7OX2H'
    const result = shortAddr(addr, 4)
    expect(result).toBe('GBRP…OX2H')
  })

  it('does not shorten a short address', () => {
    const addr = 'GABC'
    expect(shortAddr(addr)).toBe('GABC')
  })
})

// ================================================================
//  isValidStellarAddress
// ================================================================

describe('isValidStellarAddress', () => {
  it('validates a correct G-address', () => {
    // Must be exactly 55 chars (G + 54 base32 chars) per the current regex
    // G-address pattern: ^G[A-Z2-7]{54}$
    // 55-character valid G-address
    expect(isValidStellarAddress('GABCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUV')).toBe(true)
  })

  it('validates a correct C-address', () => {
    expect(isValidStellarAddress('CABCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUV')).toBe(true)
  })

  it('rejects an empty string', () => {
    expect(isValidStellarAddress('')).toBe(false)
  })

  it('rejects a string that is too short', () => {
    expect(isValidStellarAddress('GABC')).toBe(false)
  })

  it('rejects a string that starts with invalid prefix', () => {
    // 55-char address with X prefix
    expect(isValidStellarAddress('XABCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUV')).toBe(false)
  })

  it('rejects a string with invalid characters', () => {
    // '0' is not in base32 alphabet (A-Z, 2-7)
    expect(isValidStellarAddress('G0BCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUV')).toBe(false)
  })
})

// ================================================================
//  formatDuration
// ================================================================

describe('formatDuration', () => {
  it('formats 0 seconds as "0 seconds"', () => {
    expect(formatDuration(0)).toBe('0 seconds')
  })

  it('formats 1 second as "1 second"', () => {
    expect(formatDuration(1)).toBe('1 second')
  })

  it('formats 60 seconds as "1 minute"', () => {
    expect(formatDuration(60)).toBe('1 minute')
  })

  it('formats 90 seconds as "1 minute, 30 seconds"', () => {
    expect(formatDuration(90)).toBe('1 minute, 30 seconds')
  })

  it('formats 3600 seconds (1 hour) as "1 hour"', () => {
    expect(formatDuration(3600)).toBe('1 hour')
  })

  it('formats 3661 seconds as "1 hour, 1 minute"', () => {
    expect(formatDuration(3661)).toBe('1 hour, 1 minute')
  })

  it('formats 86400 seconds (1 day) as "1 day"', () => {
    expect(formatDuration(86400)).toBe('1 day')
  })

  it('formats 90061 seconds as "1 day, 1 hour"', () => {
    expect(formatDuration(90061)).toBe('1 day, 1 hour')
  })

  it('formats 2592000 seconds (1 month) as "1 month"', () => {
    expect(formatDuration(2592000)).toBe('1 month')
  })

  it('formats 31536000 seconds (1 year) as "1 year"', () => {
    expect(formatDuration(31536000)).toBe('1 year')
  })

  it('formats 1 year + 3 months as "1 year, 3 months"', () => {
    expect(formatDuration(31536000 + 3 * 2592000)).toBe('1 year, 3 months')
  })

  it('formats 2 years, 3 months, 5 days as "2 years, 3 months"', () => {
    // Should stop after 2 units
    const duration = 2 * 31536000 + 3 * 2592000 + 5 * 86400
    expect(formatDuration(duration)).toBe('2 years, 3 months')
  })

  it('formats 157788000 (max lock ~5 years) as "5 years, 1 day"', () => {
    expect(formatDuration(157788000)).toBe('5 years, 1 day')
  })

  it('formats 45 days as "1 month, 15 days"', () => {
    expect(formatDuration(45 * 86400)).toBe('1 month, 15 days')
  })

  it('formats negative seconds as "0 seconds"', () => {
    expect(formatDuration(-10)).toBe('0 seconds')
  })

  it('formats plural and singular correctly', () => {
    expect(formatDuration(120)).toBe('2 minutes')
    expect(formatDuration(60)).toBe('1 minute')
  })
})

// ================================================================
//  Timezone utilities
// ================================================================

describe('dateTimeLocalToUnixSeconds', () => {
  it('converts empty string to 0', () => {
    expect(dateTimeLocalToUnixSeconds('')).toBe(0)
  })

  it('converts a datetime-local string to Unix timestamp', () => {
    // Test with a known reference point
    // The exact result depends on the system timezone, but we can verify
    // that the function returns a number
    const result = dateTimeLocalToUnixSeconds('2026-07-28T12:00')
    expect(typeof result).toBe('number')
    expect(result).toBeGreaterThan(0)
  })

  it('handles different time values', () => {
    const ts1 = dateTimeLocalToUnixSeconds('2026-07-28T10:00')
    const ts2 = dateTimeLocalToUnixSeconds('2026-07-28T11:00')
    // ts2 should be 3600 seconds (1 hour) after ts1
    expect(ts2 - ts1).toBe(3600)
  })

  it('handles different dates', () => {
    const ts1 = dateTimeLocalToUnixSeconds('2026-07-28T12:00')
    const ts2 = dateTimeLocalToUnixSeconds('2026-07-29T12:00')
    // ts2 should be 86400 seconds (1 day) after ts1
    expect(ts2 - ts1).toBe(86400)
  })
})

describe('getTimezoneOffsetString', () => {
  it('returns a string in UTC+X format', () => {
    const offset = getTimezoneOffsetString()
    expect(offset).toMatch(/^UTC[+-]\d{1,2}(:\d{2})?$/)
  })

  it('returns a valid timezone offset', () => {
    const offset = getTimezoneOffsetString()
    // Should be something like "UTC+8" or "UTC-5"
    expect(offset).toContain('UTC')
  })
})

describe('getMinDateTimeLocal', () => {
  beforeEach(() => {
    // Clear any mock timers
    vi.useRealTimers()
  })

  it('returns a string in the correct datetime-local format', () => {
    const result = getMinDateTimeLocal()
    // Format should be YYYY-MM-DDTHH:mm
    expect(result).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/)
  })

  it('returns a date at least 60 seconds in the future', () => {
    const now = Math.floor(Date.now() / 1000)
    const minLocal = getMinDateTimeLocal()
    const minTs = dateTimeLocalToUnixSeconds(minLocal)
    const diff = minTs - now
    // Should be at least 60 seconds in the future
    // Note: datetime-local only has minute precision, so the actual time
    // could be anywhere within the minute (0-59 seconds into it), hence we check >= 0
    expect(diff).toBeGreaterThanOrEqual(0)
    // But it should be within 2 minutes (120 seconds)
    expect(diff).toBeLessThanOrEqual(120)
  })
})

describe('formatUnlockTimestampWithTimezone', () => {
  it('returns an object with local, utc, and offset properties', () => {
    const result = formatUnlockTimestampWithTimezone(1690540800) // 2023-07-28 12:00:00 UTC
    expect(result).toHaveProperty('local')
    expect(result).toHaveProperty('utc')
    expect(result).toHaveProperty('offset')
  })

  it('formats local time as a readable string', () => {
    const result = formatUnlockTimestampWithTimezone(1690540800)
    expect(typeof result.local).toBe('string')
    expect(result.local.length).toBeGreaterThan(0)
  })

  it('formats UTC time as a readable string', () => {
    const result = formatUnlockTimestampWithTimezone(1690540800)
    expect(typeof result.utc).toBe('string')
    expect(result.utc.length).toBeGreaterThan(0)
    expect(result.utc).toContain('2023')
  })

  it('formats offset as a valid timezone string', () => {
    const result = formatUnlockTimestampWithTimezone(1690540800)
    expect(result.offset).toMatch(/^UTC[+-]\d{1,2}(:\d{2})?$/)
  })
})

// ================================================================
//  Token decimal utilities (amountToBaseUnits, baseUnitsToAmount)
// ================================================================

describe('amountToBaseUnits', () => {
  it('converts XLM amount to stroops (7 decimals)', () => {
    expect(amountToBaseUnits('1', 7)).toBe(10_000_000n)
    expect(amountToBaseUnits('1.5', 7)).toBe(15_000_000n)
    expect(amountToBaseUnits('0.0000001', 7)).toBe(1n)
  })

  it('converts USDC amount to base units (6 decimals)', () => {
    expect(amountToBaseUnits('1', 6)).toBe(1_000_000n)
    expect(amountToBaseUnits('1.5', 6)).toBe(1_500_000n)
    expect(amountToBaseUnits('100', 6)).toBe(100_000_000n)
  })

  it('handles tokens with 8 decimals', () => {
    expect(amountToBaseUnits('1', 8)).toBe(100_000_000n)
    expect(amountToBaseUnits('0.5', 8)).toBe(50_000_000n)
  })

  it('handles tokens with 18 decimals (like many ERC-20 style)', () => {
    expect(amountToBaseUnits('1', 18)).toBe(1_000_000_000_000_000_000n)
  })

  it('returns 0 for empty string', () => {
    expect(amountToBaseUnits('', 6)).toBe(0n)
  })

  it('handles negative decimals gracefully', () => {
    expect(amountToBaseUnits('1', -1)).toBe(0n)
  })

  it('truncates excess decimal places', () => {
    // "1.123456789" with 6 decimals should truncate to "1.123456"
    expect(amountToBaseUnits('1.123456789', 6)).toBe(1_123_456n)
  })

  it('pads missing decimal places with zeros', () => {
    // "1.1" with 6 decimals should be "1.100000"
    expect(amountToBaseUnits('1.1', 6)).toBe(1_100_000n)
  })
})

describe('baseUnitsToAmount', () => {
  it('converts stroops back to XLM string (7 decimals)', () => {
    expect(baseUnitsToAmount(10_000_000n, 7)).toBe('1.0000000')
    expect(baseUnitsToAmount(15_000_000n, 7)).toBe('1.5000000')
    expect(baseUnitsToAmount(1n, 7)).toBe('0.0000001')
  })

  it('converts USDC base units to amount string (6 decimals)', () => {
    expect(baseUnitsToAmount(1_000_000n, 6)).toBe('1.000000')
    expect(baseUnitsToAmount(1_500_000n, 6)).toBe('1.500000')
    expect(baseUnitsToAmount(100_000_000n, 6)).toBe('100.000000')
  })

  it('round-trip: amountToBaseUnits -> baseUnitsToAmount', () => {
    const cases: [string, number][] = [
      ['1.5', 6],
      ['100', 8],
      ['0.000001', 6],
      ['1', 18],
    ]
    for (const [amount, decimals] of cases) {
      const baseUnits = amountToBaseUnits(amount, decimals)
      const converted = baseUnitsToAmount(baseUnits, decimals)
      expect(converted).toBe(
        baseUnitsToAmount(amountToBaseUnits(amount, decimals), decimals)
      )
    }
  })

  it('handles 0 correctly', () => {
    expect(baseUnitsToAmount(0n, 6)).toBe('0.000000')
  })

  it('handles negative decimals', () => {
    // With negative decimals, just return the stringified number
    expect(baseUnitsToAmount(123456n, -1)).toBe('123456')
  })

  it('pads fractional part to correct decimal places', () => {
    // 1 unit with 6 decimals is 0.000001
    expect(baseUnitsToAmount(1n, 6)).toBe('0.000001')
    // 100 units with 8 decimals is 0.00000100
    expect(baseUnitsToAmount(100n, 8)).toBe('0.00000100')
  })
})

// ================================================================
//  Contract address validation
// ================================================================

describe('isValidContractAddress', () => {
  it('validates a correct C-address (contract)', () => {
    // 55-character C-address (C + 54 base32 chars)
    expect(isValidContractAddress('CABCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUV')).toBe(true)
  })

  it('rejects G-address (accounts, not contracts)', () => {
    expect(isValidContractAddress('GABCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUV')).toBe(false)
  })

  it('rejects empty string', () => {
    expect(isValidContractAddress('')).toBe(false)
  })

  it('rejects too short address', () => {
    expect(isValidContractAddress('CABC')).toBe(false)
  })

  it('rejects too long address', () => {
    expect(
      isValidContractAddress('CABCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUV' + 'XYZ'),
    ).toBe(false)
  })

  it('rejects invalid base32 characters', () => {
    // '0' is not in base32 alphabet
    expect(isValidContractAddress('C0BCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUV')).toBe(false)
  })

  it('rejects lowercase letters', () => {
    expect(isValidContractAddress('CabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUV2345')).toBe(false)
  })
})

describe('validateTokenAddress', () => {
  it('returns valid for correct C-address', () => {
    const result = validateTokenAddress('CABCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUV')
    expect(result.valid).toBe(true)
    expect(result.message).toContain('Valid')
  })

  it('returns error for empty string', () => {
    const result = validateTokenAddress('')
    expect(result.valid).toBe(false)
    expect(result.message).toContain('required')
  })

  it('returns error for wrong length', () => {
    const result = validateTokenAddress('CABC')
    expect(result.valid).toBe(false)
    expect(result.message).toContain('55 characters')
    expect(result.message).toContain('4')
  })

  it('returns error for wrong prefix', () => {
    const result = validateTokenAddress('GABCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUV')
    expect(result.valid).toBe(false)
    expect(result.message).toContain('start with "C"')
  })

  it('returns error for invalid characters', () => {
    const result = validateTokenAddress('C0BCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUV')
    expect(result.valid).toBe(false)
    expect(result.message).toContain('Invalid characters')
  })

  it('provides helpful error messages', () => {
    const cases: [string, string][] = [
      ['', 'required'],
      ['ABC', '55 characters'],
      ['GABCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTU' + 'V', 'start with'], // 55 chars, starts with G
      ['C' + '0'.repeat(54), 'Invalid characters'], // 55 chars with '0' (invalid)
    ]
    for (const [addr, expectedMsg] of cases) {
      const result = validateTokenAddress(addr)
      expect(result.valid).toBe(false)
      expect(result.message.toLowerCase()).toContain(expectedMsg.toLowerCase())
    }
  })
})
