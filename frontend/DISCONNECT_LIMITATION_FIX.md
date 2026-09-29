# Disconnect Limitation Documentation

## Problem
When users click "Disconnect" in the dApp, the app clears the local React state and removes the address from localStorage, but the Freighter browser extension still considers the origin "allowed". On the next page load or visit, the user is automatically reconnected without explicitly connecting again. This creates confusing behavior where "disconnecting" from the UI doesn't actually disconnect the wallet.

## Root Cause
Freighter's public JavaScript API does not expose any method to revoke access programmatically. The available methods are:
- `isConnected()` - Check if wallet is unlocked
- `getAddress()` - Get connected address
- `getNetworkDetails()` - Get current network
- `signTransaction()` - Sign a transaction

There is **no** `revokeAccess()`, `disconnect()`, or similar method available.

## Why This Design?
This is intentional on Freighter's part - it gives users centralized control over wallet permissions. Instead of each dApp having the power to disconnect the wallet, users manage connections directly in the Freighter extension. This prevents a malicious dApp from forcing disconnection.

## Solution: Clear Documentation
Since we cannot programmatically revoke access, we document the limitation clearly in the UI so users understand what's actually happening.

### UI Changes

#### 1. Disconnect Button Tooltip (Header.tsx)
The Disconnect button now has a detailed tooltip explaining:
- Freighter access is NOT revoked
- User will be auto-reconnected on next load
- Instructions to revoke access in Freighter's settings

```typescript
title="Disconnect from this app (Freighter access not revoked)"
```

Tooltip content shows on hover with detailed explanation.

#### 2. Wallet Info Modal (WalletInfoModal.tsx)
A help icon (bottom-right of screen) opens a modal with:

**Disconnect section:**
- Explains what clicking "Disconnect" actually does
- Emphasizes that Freighter access is NOT revoked
- Instructions: "Freighter → Settings → Connected sites → Remove this site"

**Auto-reconnect section:**
- Explains why users are auto-reconnected
- Notes this is a technical limitation of browser extensions

**Why this design? section:**
- Explains that Freighter's API doesn't provide revocation
- Notes this is intentional for security
- Empowers users to revoke access directly in Freighter

**Security tip:**
- Recommends revoking in Freighter for security-conscious users
- Emphasizes user has centralized control

### User Flow

```
User clicks "Disconnect"
    ↓
App clears local state + localStorage
    ↓
User sees tooltip or help modal explaining:
    - "Freighter access is NOT revoked"
    - "You'll be auto-reconnected"
    - "To fully disconnect: Use Freighter's settings"
    ↓
If user wants full disconnect:
    - Open Freighter extension
    - Settings → Connected sites
    - Find this dApp and remove it
```

## Technical Details

### WalletContext.tsx - Disconnect Function
```typescript
const disconnect = useCallback(() => {
  setWallet(null)
  setNetworkMismatch(false)
  localStorage.removeItem('tlv_wallet_address')
  toast.success('Wallet disconnected')
}, [])
```

The function:
1. Clears React state
2. Removes localStorage entry
3. Shows success toast
4. Does NOT attempt to call Freighter API (no such method exists)

### Session Restoration Effect
On page load, if localStorage has the address:
```typescript
useEffect(() => {
  const [initialWallet, isRestoring] = initializeWalletFromStorage()
  setWallet(initialWallet)
  setIsRestoring(isRestoring)
}, [])
```

This is intentional for UX - returning users are restored automatically.

## Freighter Extension Settings
To fully revoke dApp access in Freighter:
1. Click Freighter extension icon
2. Click Settings (gear icon)
3. Go to "Connected sites" tab
4. Find the dApp URL
5. Click "Remove" or "X" button

## Limitations
- Programmatic revocation not possible via API
- User must manually revoke in Freighter extension
- Auto-reconnection happens on page load (design feature, not a bug)
- Can't prevent this behavior - it's browser extension architecture

## Workarounds if Needed
1. **For developers testing:** Clear browser LocalStorage manually (DevTools → Application)
2. **For users:** Revoke in Freighter settings, then clear browser cache/storage
3. **For high-security needs:** Use Freighter's account switching or create a separate browser profile

## Future Considerations
If Freighter eventually adds a disconnect/revoke API:
1. Check for method existence: `typeof window.freighter?.revokeAccess`
2. Call it in the disconnect function
3. Update documentation to note revocation now works
4. Consider removing auto-restoration if users expect full disconnect

## Files Modified
1. **src/components/Header.tsx** - Enhanced tooltip on Disconnect button
2. **src/components/WalletInfoModal.tsx** - New help modal with detailed explanation
3. **src/App.tsx** - Added WalletInfoModal to main app

## Testing Checklist
- ✅ Disconnect button shows enhanced tooltip
- ✅ Help icon appears in bottom-right corner
- ✅ Modal opens with comprehensive information
- ✅ Modal closes on button click or backdrop click
- ✅ No TypeScript errors
- ✅ Dev server starts successfully
- ✅ Auto-reconnection still works (intentional)
- ✅ Disconnect removes localStorage (confirmed)

## Security Notes
- This design gives users full control over permissions
- Users manage connections directly in Freighter
- No malicious dApp can force disconnection
- Clear UI prevents user confusion
- Better than attempting workarounds

## References
- [Freighter API Documentation](https://developers.stellar.org/docs/build/guides/freighter)
- [Freighter GitHub](https://github.com/stellar/freighter)
- Browser extension permissions architecture (WebExtensions API)
