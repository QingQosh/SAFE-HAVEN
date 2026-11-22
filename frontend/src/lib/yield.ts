// ============================================================
//  Yield calculation utilities for compound interest deposits
// ============================================================

import type { Deposit } from '../types'

// ============================================================
//  Sustainability metrics (carbon footprint & milestones)
// ============================================================

/**
 * Signature of the read-only contract invocation helper used by the
 * sustainability helpers. It should resolve the contract method and
 * return the decoded (or raw) result.
 */
export type InvokeRead = (
  method: string,
  args?: Record<string, unknown>,
) => Promise<unknown>

/**
 * Per-deposit sustainability metrics as returned by the contract's
 * `get_sustainability_metrics` query.
 *
 * Data types:
 * - `depositId`: number
 * - `carbonFootprintGrams`: bigint (grams of CO2e)
 * - `carbonOffsetGrams`: bigint (grams of CO2e offset)
 * - `netCarbonGrams`: bigint (footprint minus offset, may be negative)
 * - `greenScore`: number (0-100)
 * - `milestones`: string[] (achievement identifiers)
 */
export interface SustainabilityMetrics {
  depositId: number
  carbonFootprintGrams: bigint
  carbonOffsetGrams: bigint
  netCarbonGrams: bigint
  greenScore: number
  milestones: string[]
}

/**
 * Aggregate sustainability report as returned by the contract's
 * `generate_sustainability_report` query.
 *
 * Data types:
 * - `depositor`: string (address)
 * - `totalCarbonFootprintGrams`: bigint
 * - `totalCarbonOffsetGrams`: bigint
 * - `netCarbonGrams`: bigint
 * - `averageGreenScore`: number (0-100)
 * - `milestones`: string[] (all milestones achieved)
 * - `depositCount`: number
 */
export interface SustainabilityReport {
  depositor: string
  totalCarbonFootprintGrams: bigint
  totalCarbonOffsetGrams: bigint
  netCarbonGrams: bigint
  averageGreenScore: number
  milestones: string[]
  depositCount: number
}

function toBigInt(value: unknown): bigint {
  if (typeof value === 'bigint') return value
  if (typeof value === 'number') return BigInt(Math.trunc(value))
  if (typeof value === 'string' && value.trim() !== '') return BigInt(value)
  return 0n
}

function toNumber(value: unknown): number {
  if (typeof value === 'number') return value
  if (typeof value === 'bigint') return Number(value)
  if (typeof value === 'string' && value.trim() !== '') return Number(value)
  return 0
}

function toStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  return value.map((item) => String(item))
}

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === 'object'
    ? (value as Record<string, unknown>)
    : {}
}

/**
 * Fetch sustainability metrics for a single deposit.
 *
 * @param invokeRead - Contract read helper
 * @param depositor - Depositor address
 * @param depositId - Deposit identifier
 * @returns Decoded {@link SustainabilityMetrics}
 *
 * @example
 * ```ts
 * // Dashboard integration: fetch and render metrics + milestones
 * const metrics = await getSustainabilityMetrics(invokeRead, address, depositId)
 *
 * renderCard({
 *   footprint: formatCarbonFootprint(metrics.carbonFootprintGrams),
 *   offset: formatCarbonFootprint(metrics.carbonOffsetGrams),
 *   net: formatCarbonFootprint(metrics.netCarbonGrams),
 *   score: `${metrics.greenScore}/100`,
 *   milestones: metrics.milestones,
 * })
 *
 * const report = await generateSustainabilityReport(invokeRead, address)
 * renderSummary({
 *   total: formatCarbonFootprint(report.totalCarbonFootprintGrams),
 *   averageScore: `${report.averageGreenScore}/100`,
 *   milestones: report.milestones,
 * })
 * ```
 */
export async function getSustainabilityMetrics(
  invokeRead: InvokeRead,
  depositor: string,
  depositId: number,
): Promise<SustainabilityMetrics> {
  const raw = asRecord(
    await invokeRead('get_sustainability_metrics', { depositor, depositId }),
  )

  return {
    depositId: toNumber(raw.depositId ?? raw.deposit_id ?? depositId),
    carbonFootprintGrams: toBigInt(
      raw.carbonFootprintGrams ?? raw.carbon_footprint_grams,
    ),
    carbonOffsetGrams: toBigInt(
      raw.carbonOffsetGrams ?? raw.carbon_offset_grams,
    ),
    netCarbonGrams: toBigInt(raw.netCarbonGrams ?? raw.net_carbon_grams),
    greenScore: toNumber(raw.greenScore ?? raw.green_score),
    milestones: toStringArray(raw.milestones),
  }
}

/**
 * Fetch the aggregate sustainability report for a depositor.
 *
 * @param invokeRead - Contract read helper
 * @param depositor - Depositor address
 * @returns Decoded {@link SustainabilityReport}
 */
export async function generateSustainabilityReport(
  invokeRead: InvokeRead,
  depositor: string,
): Promise<SustainabilityReport> {
  const raw = asRecord(
    await invokeRead('generate_sustainability_report', { depositor }),
  )

  return {
    depositor: String(raw.depositor ?? depositor),
    totalCarbonFootprintGrams: toBigInt(
      raw.totalCarbonFootprintGrams ?? raw.total_carbon_footprint_grams,
    ),
    totalCarbonOffsetGrams: toBigInt(
      raw.totalCarbonOffsetGrams ?? raw.total_carbon_offset_grams,
    ),
    netCarbonGrams: toBigInt(raw.netCarbonGrams ?? raw.net_carbon_grams),
    averageGreenScore: toNumber(
      raw.averageGreenScore ?? raw.average_green_score,
    ),
    milestones: toStringArray(raw.milestones),
    depositCount: toNumber(raw.depositCount ?? raw.deposit_count),
  }
}

/**
 * Format a carbon amount (in grams) for display.
 * Scales to kg or tonnes for readability.
 *
 * @param grams - Carbon amount in grams (bigint or number)
 * @returns Human-readable string, e.g. "1.25 kg CO2e"
 */
export function formatCarbonFootprint(grams: bigint | number): string {
  const value = typeof grams === 'bigint' ? Number(grams) : grams
  const abs = Math.abs(value)
  const sign = value < 0 ? '-' : ''

  if (abs >= 1_000_000) {
    return `${sign}${(abs / 1_000_000).toFixed(2)} t CO2e`
  }
  if (abs >= 1_000) {
    return `${sign}${(abs / 1_000).toFixed(2)} kg CO2e`
  }
  return `${sign}${abs.toFixed(0)} g CO2e`
}

/**
 * Annual compound interest rate used by the contract.
 * Contract uses 5% APY (500 bps / 10_000).
 */
export const ANNUAL_INTEREST_RATE = 0.05 // 5%

/**
 * Seconds in a year (365 days) for APY calculations.
 */
export const SECONDS_PER_YEAR = 31_536_000

/**
 * Calculate the accrued amount after compound interest.
 * Mirrors the contract's compute_accrued_amount logic.
 * 
 * Formula: A = P * (1 + r/n)^(nt)
 * Where:
 *   P = principal (initial amount)
 *   r = annual interest rate (5%)
 *   n = number of compounding periods per year
 *   t = time in years
 * 
 * @param principal - Initial deposit amount
 * @param frequencySecs - Compound frequency in seconds
 * @param lastAccrualTime - Last accrual timestamp
 * @param currentTime - Current timestamp
 * @returns Accrued amount including compound interest
 */
export function calculateAccruedAmount(
  principal: bigint,
  frequencySecs: number,
  lastAccrualTime: number,
  currentTime: number,
): bigint {
  if (frequencySecs === 0 || currentTime <= lastAccrualTime) {
    return principal
  }

  const elapsed = currentTime - lastAccrualTime
  const periods = Math.floor(elapsed / frequencySecs)
  
  if (periods === 0) {
    return principal
  }

  // Calculate (1 + r/n) for each period
  // r = 0.05 (5% annual), n = periods per year = SECONDS_PER_YEAR / frequencySecs
  const periodsPerYear = SECONDS_PER_YEAR / frequencySecs
  const ratePerPeriod = ANNUAL_INTEREST_RATE / periodsPerYear
  
  // Compound: amount * (1 + rate)^periods
  // Use floating point for calculation, then convert back to bigint
  const multiplier = Math.pow(1 + ratePerPeriod, periods)
  const accrued = Number(principal) * multiplier
  
  return BigInt(Math.floor(accrued))
}

/**
 * Calculate projected yield at a future timestamp.
 * 
 * @param deposit - Deposit entry with compound settings
 * @param projectionTime - Future timestamp to project to
 * @param currentTime - Current timestamp (defaults to now)
 * @returns Projected amount at future time
 */
export function projectYield(
  deposit: Deposit,
  projectionTime: number,
  currentTime: number = Math.floor(Date.now() / 1000),
): bigint {
  if (deposit.compoundFrequencySecs === 0) {
    return deposit.amount // No compounding
  }

  // First, calculate current accrued amount
  const currentAccrued = calculateAccruedAmount(
    deposit.amount,
    deposit.compoundFrequencySecs,
    deposit.lastAccrualTimestamp || currentTime,
    currentTime,
  )

  // Then project from current to future
  return calculateAccruedAmount(
    currentAccrued,
    deposit.compoundFrequencySecs,
    currentTime,
    projectionTime,
  )
}

/**
 * Calculate total interest earned so far.
 * 
 * @param deposit - Deposit entry
 * @param currentTime - Current timestamp
 * @returns Interest earned (current_amount - initial_amount)
 */
export function calculateInterestEarned(
  deposit: Deposit,
  currentTime: number = Math.floor(Date.now() / 1000),
): bigint {
  const current = calculateAccruedAmount(
    deposit.amount,
    deposit.compoundFrequencySecs,
    deposit.lastAccrualTimestamp || currentTime,
    currentTime,
  )
  return current - deposit.amount
}

/**
 * Calculate APY (Annual Percentage Yield) for a deposit.
 * Returns the effective annual rate accounting for compounding.
 * 
 * @param frequencySecs - Compound frequency in seconds
 * @returns APY as a percentage (e.g., 5.12 for 5.12%)
 */
export function calculateAPY(frequencySecs: number): number {
  if (frequencySecs === 0) return 0

  const periodsPerYear = SECONDS_PER_YEAR / frequencySecs
  const ratePerPeriod = ANNUAL_INTEREST_RATE / periodsPerYear
  
  // APY = (1 + r/n)^n - 1
  const apy = Math.pow(1 + ratePerPeriod, periodsPerYear) - 1
  return apy * 100 // Convert to percentage
}

/**
 * Calculate yield summary for a single deposit.
 */
export interface YieldSummary {
  depositId: number
  principal: bigint
  currentBalance: bigint
  interestEarned: bigint
  projectedAtUnlock: bigint
  projectedInterestAtUnlock: bigint
  apy: number
  daysUntilUnlock: number
  isCompounding: boolean
}

export function calculateYieldSummary(
  deposit: Deposit,
  currentTime: number = Math.floor(Date.now() / 1000),
): YieldSummary {
  const currentBalance = calculateAccruedAmount(
    deposit.amount,
    deposit.compoundFrequencySecs,
    deposit.lastAccrualTimestamp || currentTime,
    currentTime,
  )
  
  const projectedAtUnlock = projectYield(deposit, deposit.unlockTime, currentTime)
  const interestEarned = currentBalance - deposit.amount
  const projectedInterestAtUnlock = projectedAtUnlock - deposit.amount
  
  const secondsUntilUnlock = Math.max(0, deposit.unlockTime - currentTime)
  const daysUntilUnlock = Math.ceil(secondsUntilUnlock / 86400)
  
  return {
    depositId: deposit.depositId,
    principal: deposit.amount,
    currentBalance,
    interestEarned,
    projectedAtUnlock,
    projectedInterestAtUnlock,
    apy: calculateAPY(deposit.compoundFrequencySecs),
    daysUntilUnlock,
    isCompounding: deposit.compoundFrequencySecs > 0,
  }
}

/**
 * Calculate aggregate yield metrics across all deposits.
 */
export interface AggregateYield {
  totalPrincipal: bigint
  totalCurrentBalance: bigint
  totalInterestEarned: bigint
  totalProjectedAtUnlock: bigint
  totalProjectedInterest: bigint
  weightedAPY: number
  activeCompoundingDeposits: number
}

export function calculateAggregateYield(
  deposits: Deposit[],
  currentTime: number = Math.floor(Date.now() / 1000),
): AggregateYield {
  let totalPrincipal = 0n
  let totalCurrentBalance = 0n
  let totalInterestEarned = 0n
  let totalProjectedAtUnlock = 0n
  let totalProjectedInterest = 0n
  let weightedAPYSum = 0
  let activeCompoundingDeposits = 0

  for (const deposit of deposits) {
    const summary = calculateYieldSummary(deposit, currentTime)
    
    totalPrincipal += summary.principal
    totalCurrentBalance += summary.currentBalance
    totalInterestEarned += summary.interestEarned
    totalProjectedAtUnlock += summary.projectedAtUnlock
    totalProjectedInterest += summary.projectedInterestAtUnlock
    
    if (summary.isCompounding) {
      // Weight APY by principal amount
      weightedAPYSum += summary.apy * Number(summary.principal)
      activeCompoundingDeposits++
    }
  }

  const weightedAPY = totalPrincipal > 0n
    ? weightedAPYSum / Number(totalPrincipal)
    : 0

  return {
    totalPrincipal,
    totalCurrentBalance,
    totalInterestEarned,
    totalProjectedAtUnlock,
    totalProjectedInterest,
    weightedAPY,
    activeCompoundingDeposits,
  }
}

/**
 * Generate yield projections at multiple future time points.
 * Useful for charting.
 */
export interface YieldProjection {
  timestamp: number
  label: string
  projectedAmount: bigint
  projectedInterest: bigint
}

export function generateYieldProjections(
  deposit: Deposit,
  currentTime: number = Math.floor(Date.now() / 1000),
): YieldProjection[] {
  if (deposit.compoundFrequencySecs === 0) {
    return [] // No projections for non-compounding deposits
  }

  const unlockTime = deposit.unlockTime
  const duration = unlockTime - currentTime
  
  if (duration <= 0) {
    return [] // Already unlocked
  }

  const projections: YieldProjection[] = []
  const intervals = [0.25, 0.5, 0.75, 1.0] // 25%, 50%, 75%, 100% of lock duration

  for (const interval of intervals) {
    const projectionTime = currentTime + Math.floor(duration * interval)
    const projected = projectYield(deposit, projectionTime, currentTime)
    const interest = projected - deposit.amount
    
    const daysFromNow = Math.ceil((projectionTime - currentTime) / 86400)
    
    projections.push({
      timestamp: projectionTime,
      label: interval === 1.0 ? 'At unlock' : `${daysFromNow} days`,
      projectedAmount: projected,
      projectedInterest: interest,
    })
  }

  return projections
}

/**
 * Compare yield vs early exit penalty.
 * Helps users understand the trade-off.
 */
export interface YieldVsPenaltyComparison {
  currentBalance: bigint
  interestEarned: bigint
  penaltyAmount: bigint
  netIfExitNow: bigint
  projectedAtUnlock: bigint
  opportunityCost: bigint
  breakEvenDays: number
}

export function compareYieldVsPenalty(
  deposit: Deposit,
  currentTime: number = Math.floor(Date.now() / 1000),
): YieldVsPenaltyComparison {
  const currentBalance = calculateAccruedAmount(
    deposit.amount,
    deposit.compoundFrequencySecs,
    deposit.lastAccrualTimestamp || currentTime,
    currentTime,
  )
  
  const interestEarned = currentBalance - deposit.amount
  const penaltyAmount = (currentBalance * BigInt(deposit.penaltyBps)) / 10_000n
  const netIfExitNow = currentBalance - penaltyAmount
  
  const projectedAtUnlock = projectYield(deposit, deposit.unlockTime, currentTime)
  const opportunityCost = projectedAtUnlock - netIfExitNow
  
  // Calculate break-even: days until interest earned exceeds penalty
  let breakEvenDays = 0
  if (deposit.compoundFrequencySecs > 0 && penaltyAmount > interestEarned) {
    // Binary search for break-even point
    let low = 0
    let high = Math.ceil((deposit.unlockTime - currentTime) / 86400)
    
    while (low < high) {
      const mid = Math.floor((low + high) / 2)
      const futureTime = currentTime + (mid * 86400)
      const futureBalance = projectYield(deposit, futureTime, currentTime)
      const futureInterest = futureBalance - deposit.amount
      
      if (futureInterest >= penaltyAmount) {
        high = mid
      } else {
        low = mid + 1
      }
    }
    
    breakEvenDays = low
  }
  
  return {
    currentBalance,
    interestEarned,
    penaltyAmount,
    netIfExitNow,
    projectedAtUnlock,
    opportunityCost,
    breakEvenDays,
  }
}
