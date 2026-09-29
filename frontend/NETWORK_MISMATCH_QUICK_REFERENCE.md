# Network Mismatch Detection - Quick Reference

## What Was Fixed
Users connecting a Freighter wallet to the wrong network (mainnet dApp using testnet wallet, or vice versa) would get confusing transaction failures. Now we detect and prevent this.

## Key Features
1. **Detection**: Calls `freighter.getNetworkDetails()` after wallet connects
2. **Prevention**: Blocks transaction signing if networks don't match
3. **Visibility**: Shows red warning banner with network info
4. **Safety**: Never sends transaction to wrong network

## How It Works

### Flow for User
```
User clicks "Connect Wallet"
    ↓
Freighter connection succeeds
    ↓
App calls getNetworkDetails()
    ↓
    ├─ If networks match:
    │   └─ Show: "Connected: [address]"
    │   └─ Can sign transactions
    │
    └─ If networks DON'T match:
        └─ Show: Red banner warning
        └─ Toast: "Network mismatch! Wallet: [X], App: [Y]"
        └─ Cannot sign transactions
        └─ Must disconnect and switch networks
```

### When User Tries to Sign
```
User clicks "Deposit" or other transaction
    ↓
signTransaction() is called
    ↓
    ├─ If networkMismatch = true:
    │   └─ Return error immediately
    │   └─ Toast: "Network mismatch: Wallet is on X, but app is on Y"
    │   └─ Freighter is NEVER called
    │
    └─ If networkMismatch = false:
        └─ Proceed with normal signing
```

## Implementation Details

### Added to WalletInfo Type
```typescript
networkMismatch?: boolean  // true if wallet and app on different networks
walletNetwork?: string     // Wallet's network passphrase (for display)
```

### Added to WalletContextValue
```typescript
networkMismatch: boolean  // State flag
```

### Network Checking Code
In `WalletContext.tsx` `connect()` function:
```typescript
// Get wallet's network
const networkDetails = await freighter.getNetworkDetails()
const walletNetworkPassphrase = networkDetails?.networkPassphrase

// Compare with app's network
const hasNetworkMismatch = walletNetworkPassphrase && 
  walletNetworkPassphrase !== CONFIG.NETWORK_PASSPHRASE

// Store in wallet info
const info: WalletInfo = {
  address,
  displayAddress,
  networkMismatch: !!hasNetworkMismatch,
  walletNetwork: walletNetworkPassphrase,
}
```

### Signing Prevention
In `WalletContext.tsx` `signTransaction()` function:
```typescript
if (networkMismatch) {
  toast.error(`Network mismatch: Wallet is on ${wallet?.walletNetwork}...`)
  return { signed: false, rejected: false, error: "Network mismatch..." }
}
// Only reaches here if networks match
```

### Warning Banner
In `Header.tsx`:
```typescript
{networkMismatch && wallet?.walletNetwork && (
  <div className="bg-red-900/30 border-red-700/40 ...">
    <p>Network Mismatch</p>
    <p>Your wallet is on {walletNetwork}</p>
    <p>App is configured for {CONFIG.NETWORK_PASSPHRASE}</p>
    <p>Disconnect and switch your wallet to the correct network.</p>
  </div>
)}
```

## Testing
All 12 tests pass verifying:
- ✅ Network mismatch detection
- ✅ Wallet info tracking
- ✅ Transaction signing prevention
- ✅ Warning message display
- ✅ Error handling
- ✅ Session restoration

## Networks Supported
- **Testnet**: `Test SDF Network ; September 2015`
- **Mainnet**: `Public Global Stellar Network ; September 2015`
- **Futurenet**: `Test SDF Future Network ; September 2015` (configurable)

## Error Scenarios Handled
1. **Freighter not available**: Shows generic error
2. **getNetworkDetails() fails**: Logs warning, allows connection (default to no mismatch)
3. **User switches networks in Freighter**: Detected on next session restoration
4. **User tries to sign on wrong network**: Blocked with clear error

## UI Changes
- Red warning banner appears at top of page when mismatch detected
- Error toast on connection shows network names
- Error toast on signing attempt prevented
- Warning banner is persistent (no auto-dismiss)

## Backward Compatibility
- Works with existing Freighter installations
- Graceful fallback if `getNetworkDetails()` not available
- No breaking changes to existing code
