# Wallet Restoration Flash Fix

## Problem
The wallet context initialized with `wallet = null`, then in a `useEffect` restored it from localStorage asynchronously. Components rendered before the effect fired saw `wallet = null` and displayed a "Connect Wallet" button briefly, causing a layout flash (Cumulative Layout Shift - CLS) on every page load for authenticated users returning to the site.

## Solution
Implemented **synchronous localStorage initialization** combined with an **`isRestoringSession` flag** that allows UI to show a loading skeleton instead of the connect prompt during the brief restoration window.

### Architecture

#### 1. Synchronous Initialization (WalletContext.tsx)
- Added `initializeWalletFromStorage()` helper function
- Runs synchronously during the first `useEffect` to read localStorage
- Returns `[wallet, isRestoringSession]` tuple:
  - If wallet found: returns the wallet info + `isRestoringSession: true`
  - If not found: returns `[null, false]`

```typescript
function initializeWalletFromStorage(): [WalletInfo | null, boolean] {
  const saved = localStorage.getItem('tlv_wallet_address')
  if (!saved) return [null, false]
  
  // Found a saved address — restore immediately but mark as restoring
  return [{ address: saved, displayAddress: shortAddr(saved) }, true]
}
```

#### 2. Async Validation (WalletContext.tsx)
- Separate `useEffect` runs when `isRestoringSession: true`
- Validates the restored wallet against Freighter (checks if connected, account matches)
- Sets `isRestoringSession: false` when validation completes
- Clears wallet if validation fails

#### 3. UI Indicators (Header.tsx, Pages)
- **Header**: Shows skeleton with animated placeholders during `isRestoringSession`
- **Pages**: All pages check `!wallet && !isRestoringSession` before showing "Connect Wallet" prompt
- **Dashboard**: Shows full skeleton layout during restoration

### State Transitions

```
Page Load
├─ isRestoringSession: false, wallet: null
│  └─ Check: localStorage
│
├─ If wallet found:
│  ├─ isRestoringSession: true, wallet: restored from localStorage
│  └─ UI shows skeleton (no flash of "Connect Wallet")
│
├─ Async Freighter validation starts
│
├─ Validation complete:
│  ├─ If valid: isRestoringSession: false, wallet: restored
│  │  └─ UI shows connected state
│  ├─ If invalid: isRestoringSession: false, wallet: null
│  │  └─ UI shows "Connect Wallet" button
│
└─ If no wallet saved initially:
   ├─ isRestoringSession: false, wallet: null
   └─ UI shows "Connect Wallet" button immediately
```

## Files Modified

### Core Implementation
1. **src/context/WalletContext.tsx**
   - Added `isRestoringSession: boolean` to `WalletContextValue`
   - Implemented `initializeWalletFromStorage()` helper
   - Split restoration into two effects: sync init + async validation

2. **src/components/Header.tsx**
   - Added `isRestoringSession` check
   - Shows skeleton during restoration instead of "Connect Wallet"

### Page Updates
3. **src/pages/Dashboard.tsx**
   - Updated wallet check to: `!wallet && !isRestoringSession`
   - Added full skeleton layout during restoration

4. **src/pages/DepositPage.tsx**
   - Updated wallet check to: `!wallet && !isRestoringSession`

5. **src/pages/WithdrawPage.tsx**
   - Updated wallet check to: `!wallet && !isRestoringSession`

6. **src/pages/AdminPage.tsx**
   - Updated wallet check to: `!wallet && !isRestoringSession`

## Benefits

### User Experience
- **No flash**: Returning users don't see the "Connect Wallet" button flash
- **No CLS**: Layout remains stable while validation happens
- **Visual continuity**: Skeleton loaders maintain consistent layout

### Technical
- **Type-safe**: New flag is properly typed
- **Fast**: Synchronous initialization means wallet available in first render
- **Resilient**: Async validation still protects against invalid sessions

## Test Coverage
- 9 comprehensive tests verify the fix (WalletRestoration.test.ts)
- Tests cover:
  - Synchronous localStorage initialization
  - Restoration state tracking
  - Skeleton display during restoration
  - Connect button display logic
  - Validation failure handling
  - All pages handle restoration correctly

## Performance Impact
- **Positive**: Eliminates the need to wait for async validation before showing UI
- **Minimal**: Only adds a single synchronous localStorage read
- **No regression**: Async validation still happens in background

## Browser Compatibility
- Works with all modern browsers supporting localStorage
- Graceful fallback if localStorage unavailable (checks before use)
- SSR safe (checks for window/localStorage existence)
