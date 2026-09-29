# Wallet Signing Error Handling Fix

## Problem
The wallet signing catch block was suppressing errors whose message included "reject" or "cancel" (assumed to be user-initiated). Any other error — including network failures, internal Freighter bugs, or signing errors on malformed XDR — was shown as a toast but the function returned `null`. The calling code could not distinguish "user rejected" from "signing failed due to an error," which matters for UX:
- **User rejection** should silently reset state
- **Signing failure** should show a retry prompt

## Solution
Implemented a **discriminated union return type** that allows callers to handle each case appropriately.

### Type Definition (types.ts)
```typescript
export type SigningResult = 
  | { signed: true; xdr: string }           // Successfully signed
  | { signed: false; rejected: true }       // User rejected the signing request
  | { signed: false; rejected: false; error: string } // Signing failed with error
```

### Context Implementation (WalletContext.tsx)
The `signTransaction` function now:
1. Returns `{ signed: true; xdr: "..." }` on success
2. Returns `{ signed: false; rejected: true }` when user rejects (silently, no toast)
3. Returns `{ signed: false; rejected: false; error: "..." }` on signing failure (toast shown by context, but caller resets state)

```typescript
const signTransaction = useCallback(async (txXdr: string): Promise<SigningResult> => {
  try {
    // ... setup ...
    if (error) {
      toast.error(`Signing failed: ${error}`)
      return { signed: false, rejected: false, error }
    }
    return { signed: true, xdr: signedTxXdr as string }
  } catch (e) {
    const msg = e instanceof Error ? e.message : 'Signing rejected'
    const isUserReject = msg.toLowerCase().includes('reject') || msg.toLowerCase().includes('cancel')
    
    if (!isUserReject) {
      toast.error(`Signing error: ${msg}`)
      return { signed: false, rejected: false, error: msg }
    }
    
    // User rejection — silent, just return rejected flag
    return { signed: false, rejected: true }
  }
}, [])
```

### Calling Code Pattern (DepositPage, WithdrawPage, Dashboard, AdminPage)
```typescript
const sigResult = await signTransaction(xdr)

if (sigResult.signed) {
  // Success: proceed with transaction submission
  setTxStatus('submitting')
  const result = await submitTx(sigResult.xdr)
  // ... handle result ...
} else if (sigResult.rejected) {
  // User rejected: silently reset state, no toast needed
  setTxStatus('idle')
} else {
  // Signing error: toast already shown by context, reset state
  setTxStatus('idle')
}
```

## Benefits
1. **Type safety**: TypeScript catches incorrect handling of signing results
2. **Clear intent**: Discriminated union makes the three outcomes explicit
3. **Better UX**: Callers can now:
   - Show a retry prompt only for actual signing failures
   - Silently dismiss user rejections without confusing the user
4. **Proper error handling**: Network errors, malformed XDR, etc. are distinguished from user cancellations

## Test Coverage
- 11 tests verify the implementation:
  - Union type structure and narrowing
  - Real-world handling patterns in UI pages
  - Distinction between rejection and error for UX decisions
  - Type guards work correctly across all three cases

## Files Modified
1. `src/types.ts` - Added `SigningResult` type
2. `src/context/WalletContext.tsx` - Updated `signTransaction` return type and implementation
3. `src/pages/DepositPage.tsx` - Updated signing result handling
4. `src/pages/WithdrawPage.tsx` - Updated signing result handling
5. `src/pages/Dashboard.tsx` - Updated signing result handling (2 functions)
6. `src/pages/AdminPage.tsx` - Updated signing result handling (5 functions)

## Tests Added
1. `src/__tests__/SigningResult.test.ts` - Type structure validation (5 tests)
2. `src/__tests__/SigningResult.behavior.test.ts` - Real-world behavior patterns (6 tests)
