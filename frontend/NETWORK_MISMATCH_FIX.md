# Network Mismatch Detection

## Problem
When a user connects their Freighter wallet to the dApp, there was no verification that the wallet is configured for the same network as the dApp. A user on mainnet Freighter could connect to a testnet dApp (or vice versa) and would receive confusing transaction failures with no clear explanation.

## Solution
Implemented **network mismatch detection** that:
1. Calls `freighter.getNetworkDetails()` after wallet connection
2. Compares the wallet's network passphrase with `CONFIG.NETWORK_PASSPHRASE`
3. Shows a prominent warning banner if networks don't match
4. Prevents transaction signing if there's a mismatch
5. Tracks the mismatch state throughout the session

## Implementation

### Type Updates (types.ts)
```typescript
export interface WalletInfo {
  address: string
  displayAddress: string
  networkMismatch?: boolean  // True if wallet network differs from app network
  walletNetwork?: string     // Network passphrase from Freighter (for display)
}
```

### Context Updates (WalletContext.tsx)

#### 1. Added networkMismatch state
```typescript
const [networkMismatch, setNetworkMismatch] = useState(false)
```

#### 2. Network check in connect function
```typescript
// Check network mismatch
let walletNetworkPassphrase: string | undefined
try {
  const networkDetails = await freighter.getNetworkDetails()
  walletNetworkPassphrase = networkDetails?.networkPassphrase
} catch (e) {
  // If getNetworkDetails fails, we can't verify the network
  console.warn('Could not get network details from Freighter:', e)
}

const hasNetworkMismatch = walletNetworkPassphrase && 
  walletNetworkPassphrase !== CONFIG.NETWORK_PASSPHRASE

const info: WalletInfo = {
  address,
  displayAddress: shortAddr(address),
  networkMismatch: !!hasNetworkMismatch,
  walletNetwork: walletNetworkPassphrase,
}
setWallet(info)
setNetworkMismatch(!!hasNetworkMismatch)
```

#### 3. Network check during session restoration
Same check performed in the validation effect to detect if wallet network changed since last session.

#### 4. Prevent signing on mismatch
```typescript
if (networkMismatch) {
  const msg = `Network mismatch: Wallet is on ${wallet?.walletNetwork}, but app is on ${CONFIG.NETWORK_PASSPHRASE}`
  toast.error(msg, { duration: 0 })
  return { signed: false, rejected: false, error: msg }
}
```

### Header Updates (Header.tsx)
Shows a prominent red warning banner when network mismatch is detected:

```typescript
{networkMismatch && wallet?.walletNetwork && (
  <div className="w-full bg-red-900/30 border-b border-red-700/40 px-4 py-3">
    <div className="max-w-6xl mx-auto flex items-center gap-3">
      <svg><!-- Warning icon --></svg>
      <div className="flex-1">
        <p className="text-sm font-semibold text-red-400 mb-1">Network Mismatch</p>
        <p className="text-xs text-red-300">
          Your Freighter wallet is on <span className="font-mono">{wallet.walletNetwork}</span>, 
          but this app is configured for <span className="font-mono">{CONFIG.NETWORK_PASSPHRASE}</span>. 
          Disconnect and switch your wallet to the correct network.
        </p>
      </div>
    </div>
  </div>
)}
```

## Behavior

### Connection Flow
1. User clicks "Connect Wallet"
2. Freighter requests permission and returns address
3. `getNetworkDetails()` is called to check wallet network
4. If networks match:
   - Show success toast "Connected: [address]"
   - Set `networkMismatch: false`
5. If networks don't match:
   - Show error toast "Network mismatch! Wallet: [X], App: [Y]"
   - Set `networkMismatch: true`
   - Show red warning banner on page
6. If `getNetworkDetails()` fails:
   - Log warning but allow connection
   - Default to `networkMismatch: false`

### Transaction Signing
- If `networkMismatch: true`:
  - Return error immediately: `{ signed: false, rejected: false, error: "Network mismatch: ..." }`
  - Show error toast
  - Transaction is NOT sent to Freighter
- If `networkMismatch: false`:
  - Proceed with signing normally

### Session Restoration
When a user returns and their wallet is restored from localStorage:
1. Wallet address is restored immediately
2. Async validation calls `getNetworkDetails()` again
3. Network mismatch is re-checked
4. State is updated if network has changed

### Disconnection
- When user disconnects, `networkMismatch` is reset to `false`
- User must connect again if they switch networks

## Error Handling

### getNetworkDetails() failure
- If the call fails (Freighter older version, network error, etc.):
  - Warning is logged to console
  - Connection is NOT blocked
  - `networkMismatch` defaults to `false` (assume matching)
  - User can proceed but should be aware

### Display in Header
- Only shown if: `networkMismatch && wallet?.walletNetwork`
- Provides clear instructions: "Disconnect and switch your wallet to the correct network"
- Banner is persistent (no auto-dismiss) to ensure user sees it

## Network Passphrases Supported
- **Testnet**: `Test SDF Network ; September 2015`
- **Mainnet**: `Public Global Stellar Network ; September 2015`
- **Futurenet** (if configured): `Test SDF Future Network ; September 2015`

## Files Modified
1. **src/types.ts** - Added `networkMismatch` and `walletNetwork` to `WalletInfo`
2. **src/context/WalletContext.tsx** - Implemented network detection and prevention
3. **src/components/Header.tsx** - Added warning banner for mismatch

## Test Coverage
- 12 comprehensive tests verify:
  - Network mismatch detection
  - Wallet info tracking
  - Signing prevention
  - Warning message display
  - Error handling for `getNetworkDetails()` failure
  - Session restoration behavior
  - Dynamic network changes

## Benefits
1. **User Safety**: Prevents accidental transactions on wrong network
2. **Clear Communication**: Shows which networks are involved
3. **Graceful Degradation**: Doesn't block if network check fails
4. **Visible Warning**: Persistent banner ensures user sees mismatch
5. **Session Persistent**: Detects if user switched networks between sessions

## Known Limitations
- Only detects network mismatch at connection time and session restoration
- If user switches networks in Freighter while connected, won't detect until next page load or session restoration
- Some older versions of Freighter may not support `getNetworkDetails()`

## Future Enhancements
- Periodic re-check of network during session (e.g., every 5 minutes)
- "Auto-switch" option if app can recommend network to switch to
- Better error messages for specific network detection failures
