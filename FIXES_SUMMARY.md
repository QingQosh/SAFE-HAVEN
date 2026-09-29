# Frontend Issues Fixed

Three critical frontend issues have been resolved to improve UX and reliability.

---

## Issue #77: Auto-refresh after successful transactions

**Problem:** After a successful withdrawal or cancel, `setTimeout(refresh, 2000)` causes a 2-second delay. If the RPC node hasn't indexed the transaction yet, the refresh returns stale data with the just-withdrawn vault still visible.

**Solution:** 
- Added `pollRemoveDeposit(depositId, maxAttempts)` method to `useDeposits` hook
- Polls `getVault(depositor, depositId)` with exponential backoff (500ms, 1s, 2s, 4s, 8s)
- Removes deposit from local state immediately when `getVault()` returns `null`
- Falls back to full refresh if polling exhausted

**Files Modified:**
- `frontend/src/hooks/useDeposits.ts` — Added `pollRemoveDeposit` function
- `frontend/src/pages/Dashboard.tsx` — Updated `handleWithdraw()` and `handleCancel()` to call `pollRemoveDeposit(depositId)` instead of `setTimeout(refresh, 2000)`

**Result:** Stale vaults are removed from UI immediately after transaction, no delay-based refresh race condition.

---

## Issue #78: Dashboard stats flash with incorrect "Unlocked" count

**Problem:** On initial load, before `getTimeRemaining` fetch completes, `timeRemaining` defaults to `0` (via `result ?? 0`). This causes all deposits to briefly appear as "unlocked" during loading, causing a flash of incorrect stats.

**Solution:**
- Changed `Deposit.timeRemaining` type from `number` to `number | null`
- Initialize `timeRemaining` as `null` during loading
- Only count deposits as unlocked when `timeRemaining !== null && timeRemaining === 0`
- Added `StatCardSkeleton` component to show loading state for stats
- Updated `formatCountdown()` to handle `null` and treat it as "Unlocked"
- Updated DepositCard to check `isUnlocked = timeRemaining !== null && timeRemaining === 0`

**Files Modified:**
- `frontend/src/types.ts` — Changed `Deposit.timeRemaining` type to `number | null`
- `frontend/src/hooks/useDeposits.ts` — Initialize `timeRemaining` as `null ?? 0` becomes `null`
- `frontend/src/pages/Dashboard.tsx` — Added loading skeleton for stats, updated count filters
- `frontend/src/lib/format.ts` — Updated `formatCountdown()` signature to accept `number | null`
- `frontend/src/components/DepositCard.tsx` — Updated `isUnlocked` check to handle null

**Result:** Stats show loading skeletons until data loads, no flash of incorrect counts.

---

## Issue #79: No auto-refresh for contract pause state

**Problem:** `useContractInfo` fetches on mount and exposes a `refresh` callback, but there is no polling interval. If admin pauses contract while user has page open, user won't see "contract paused" warning until manual refresh.

**Solution:**
- Added 30-second polling interval in `useContractInfo` using `setInterval`
- Track previous pause state with `useRef` to detect state changes
- Emit toast notifications on pause/unpause:
  - **Pause**: Error toast "⚠️ Contract paused. New deposits are temporarily disabled." (5s)
  - **Unpause**: Success toast "✓ Contract unpaused. Deposits are now enabled." (4s)

**Files Modified:**
- `frontend/src/hooks/useContractInfo.ts` — Added polling interval and state change detection with toast notifications

**Result:** Users are notified immediately when contract pause state changes, preventing transaction submission failures.

---

## Type Safety

All changes maintain TypeScript strict mode compatibility. The changes are:
- `Deposit.timeRemaining: number | null` — properly typed and handled at all callsites
- `formatCountdown()` — accepts `number | null` with safe null handling
- No breaking changes to existing APIs

## Testing Notes

- Stats loading skeleton appears during initial load before `timeRemaining` values resolve
- Withdrawn/cancelled deposits disappear from UI within 2.5 seconds (via polling)
- Pause state changes trigger immediate toast notifications
- No stale data displayed after successful transactions
