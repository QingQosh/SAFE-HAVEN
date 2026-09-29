// ============================================================
//  Yield calculation utilities for compound interest deposits
// ============================================================

import type { Deposit } from '../types'

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
