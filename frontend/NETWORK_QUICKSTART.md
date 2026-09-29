# Network Switching — Quick Reference

## Visual Guide

```
Header Layout:
┌─────────────────────────────────────────────────────┐
│  SAFE-HAVEN        [Testnet🔴] [💰] [Connect]      │
└─────────────────────────────────────────────────────┘
                      ↑
            Click to open switcher
```

## Usage (30 seconds)

1. **See Current Network** — Look at the colored badge in the header
   - 🔴 Red = Testnet (for testing)
   - 🟢 Green = Mainnet (production)

2. **Switch Networks** — Click the badge to open dropdown menu

3. **Select Network** — Click the network you want

4. **Confirm** — Toast notification appears confirming the switch

## Network Colors

| Network | Color | Badge | Use For |
|---|---|---|---|
| Testnet | Red 🔴 | Red | Testing, development |
| Mainnet | Green 🟢 | Green | Production, real funds |

## What's Persisted

Your network selection is automatically saved to your browser. When you reload the page, the app remembers your choice.

**Storage:** Browser localStorage  
**Key:** `safe-haven_selected_network`  
**Values:** `testnet` or `mainnet`

## Warnings

### When Switching Networks
```
⚠️  Network Switch Warning

You're switching from [environment network] to [selected network].
This may cause contract interactions to fail.

Ensure your wallet is on the selected network.
```

**Action:** Verify your Freighter wallet is set to the same network as the app.

### When Networks Don't Match
```
🟡 Pulsing indicator on badge

Meaning: Selected network differs from environment.

Action: Either:
  1. Switch your wallet to match the app network
  2. Reset app to environment network
```

## Troubleshooting

**Q: Network badge shows red (testnet) but I want mainnet**  
A: Click the badge and select "Mainnet" from the dropdown

**Q: Network keeps resetting**  
A: Clear your browser's localStorage:
   - Settings → Privacy → Clear browsing data → Cookies/Site data

**Q: Transactions failing with "Network Mismatch"**  
A: Ensure Freighter wallet is on the same network as app header badge

**Q: What network is the app using?**  
A: Check the colored badge in the header (red = testnet, green = mainnet)

## Files to Know

- **Network Context:** `src/context/NetworkContext.tsx`
- **Network Utils:** `src/lib/networks.ts`
- **Switcher Component:** `src/components/NetworkSwitcher.tsx`
- **Display Component:** `src/components/NetworkDisplay.tsx`
- **Full Docs:** `NETWORK_SWITCHING.md`

## Environment Configuration

Set the default network in `.env`:

```bash
# For testnet (default)
VITE_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# For mainnet
VITE_NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
```

This is the "environment network" — the one the app starts with.

## One More Thing

**Always verify the network badge before making transactions!**

It takes 2 seconds to check and prevents costly mistakes. 🚀
