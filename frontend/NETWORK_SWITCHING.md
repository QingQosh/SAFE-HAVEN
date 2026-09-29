# Network Switching Feature Documentation

This guide explains how to use the network switching feature in SAFE-HAVEN.

## Overview

The network switching feature allows users to quickly switch between Stellar testnet and mainnet directly from the app header. The selected network is persisted in localStorage and includes visual indicators and warning messages when switching networks.

### Features

- **Prominent Network Display** — Network name displayed with distinct color badges (red for testnet, green for mainnet)
- **Quick Network Switcher** — Dropdown menu to change networks without reloading
- **Persistent Selection** — Network choice saved in localStorage
- **Warning Alerts** — Toast notifications warn when switching networks
- **Mismatch Indicators** — Visual warning if selected network differs from environment config
- **Network Info** — Shows RPC URL and Explorer link for current network
- **Wallet Validation** — Detects when wallet is on different network than app

## Architecture

### Components

#### `NetworkContext` (`src/context/NetworkContext.tsx`)

Manages network state and persistence.

**Provides:**
- `currentNetwork` — Currently selected network (testnet/mainnet)
- `envNetwork` — Environment-configured network
- `isMismatched` — Boolean flag if selection differs from environment
- `switchNetwork(networkId)` — Change to a different network
- `resetToEnvNetwork()` — Reset to environment configuration

**Storage:**
- Key: `safe-haven_selected_network`
- Values: `testnet` or `mainnet`

#### `NetworkSwitcher` Component (`src/components/NetworkSwitcher.tsx`)

Dropdown menu for network selection.

**Features:**
- Shows current network with badge
- Displays all available networks
- Shows checkmark for selected network
- Displays RPC URL and Explorer link
- Handles click-outside to close menu
- Pulsing indicator when mismatched from environment

#### `NetworkDisplay` Component (`src/components/NetworkDisplay.tsx`)

Simple badge display of current network (used in other components).

**Shows:**
- Network name (Testnet/Mainnet)
- Color-coded badge
- Warning pulse if mismatched

### Utilities

#### `networks.ts` (`src/lib/networks.ts`)

Network configuration and helpers.

**Constants:**
- `TESTNET` — Testnet configuration (red badge)
- `MAINNET` — Mainnet configuration (green badge)
- `NETWORKS` — Map of network configs
- `NETWORK_LIST` — Array of all networks

**Functions:**
- `getNetworkConfig(id)` — Get network config by ID
- `getNetworkIdByPassphrase(passphrase)` — Detect network from passphrase
- `getNetworkByPassphrase(passphrase)` — Get config from passphrase
- `detectNetworkFromEnv()` — Auto-detect from environment
- `getAlternateNetwork(id)` — Get the opposite network
- `getNetworkBadgeColor(id)` — Get Tailwind color class

## Usage

### For Users

1. **Check Current Network** — Look at the badge in the header (red = testnet, green = mainnet)
2. **Switch Network** — Click the network badge to open the dropdown
3. **Select New Network** — Click the network you want to switch to
4. **View Network Info** — See RPC URL and Explorer link at bottom of dropdown
5. **Monitor Wallet** — Ensure Freighter wallet is set to the same network

### For Developers

#### Access Network Context

```typescript
import { useNetwork } from '../context/NetworkContext'

function MyComponent() {
  const { currentNetwork, switchNetwork, isMismatched } = useNetwork()
  
  // currentNetwork is either 'testnet' or 'mainnet'
  // isMismatched is true if different from environment config
}
```

#### Get Network Configuration

```typescript
import { getNetworkConfig, NetworkId } from '../lib/networks'

const testnetConfig = getNetworkConfig(NetworkId.TESTNET)
console.log(testnetConfig.rpcUrl)     // https://soroban-testnet.stellar.org
console.log(testnetConfig.explorerUrl) // https://stellar.expert/explorer/testnet
```

#### Display Network Badge

```typescript
import { NetworkDisplay } from '../components/NetworkDisplay'

function Header() {
  return (
    <div>
      <NetworkDisplay />
    </div>
  )
}
```

#### Use Network Switcher Dropdown

```typescript
import { NetworkSwitcher } from '../components/NetworkSwitcher'

function Header() {
  return (
    <div>
      <NetworkSwitcher />
    </div>
  )
}
```

## Network Configurations

### Testnet (Red Badge)

```
Name:           Testnet
Passphrase:     Test SDF Network ; September 2015
RPC URL:        https://soroban-testnet.stellar.org
Horizon URL:    https://horizon-testnet.stellar.org
Explorer:       https://stellar.expert/explorer/testnet
Native Token:   CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC
```

**Use for:**
- Development and testing
- Deploying test contracts
- Testing transactions without real funds

### Mainnet (Green Badge)

```
Name:           Mainnet
Passphrase:     Public Global Stellar Network ; September 2015
RPC URL:        https://soroban.stellar.org
Horizon URL:    https://horizon.stellar.org
Explorer:       https://stellar.expert/explorer/public
Native Token:   CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC
```

**Use for:**
- Production deployments
- Real transactions with actual funds
- Live contracts

## Workflow

### Normal Operation (Matched Network)

```
User runs app with VITE_NETWORK_PASSPHRASE=testnet
↓
NetworkContext detects testnet from environment
↓
UI shows "Testnet" badge in red
↓
No warning pulse indicator
```

### Switching Networks

```
User clicks Network Switcher dropdown
↓
Selects "Mainnet"
↓
Toast warning appears: "You're switching from Testnet to Mainnet"
↓
Network changes to mainnet
↓
Selection saved to localStorage
↓
UI updates to show "Mainnet" badge in green
```

### Mismatch Scenario

```
Environment configured for testnet
↓
User switches to mainnet via UI
↓
Badge shows "Mainnet" (green)
↓
Pulsing yellow warning indicator appears
↓
Message: "Network selection differs from environment"
↓
User should either:
  a) Switch wallet to mainnet, OR
  b) Reset app to environment network
```

## localStorage Persistence

The selected network is saved automatically:

```javascript
// Key
'safe-haven_selected_network'

// Values
'testnet' | 'mainnet'
```

**Behavior:**
- Saved when user switches networks
- Restored on page reload
- Cleared if invalid value stored
- Falls back to environment network if not set

## Warning Messages

### Network Switch Warning

**When:** Switching away from environment-configured network

**Message:**
```
Network Switch Warning

You're switching from [environment network] to [selected network].
This may cause contract interactions to fail.

Ensure your wallet is on the selected network.
```

**Duration:** 6 seconds

### Network Switched Notification

**When:** Network successfully changed

**Message:**
```
Switched to [network name]
```

**Duration:** 3 seconds

## Visual Indicators

### Network Badge Colors

| Network | Color | Badge Class |
|---|---|---|
| Testnet | Red | `bg-red-600` |
| Mainnet | Green | `bg-green-600` |

### Mismatch Indicator

- **Visual:** Pulsing yellow dot on badge
- **Location:** Top-left of network name
- **Meaning:** Selected network differs from environment
- **Action:** User should verify wallet is on correct network

## Integration Points

### With Wallet Context

The NetworkSwitcher works alongside WalletContext:

```
WalletContext:
- Detects wallet's network passphrase
- Shows "Network Mismatch" warning if different from app

NetworkSwitcher:
- Allows changing selected network
- Shows warning when switching away from environment
```

### With Contract Calls

Before making contract calls:

```typescript
const { currentNetwork } = useNetwork()

// Use currentNetwork RPC URL for transactions
// Ensure it matches wallet network
```

### Environment Variables

App still respects `VITE_NETWORK_PASSPHRASE`:

```bash
# In .env
VITE_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
```

This is the "environment network" that the UI references.

## Troubleshooting

### Network Switch Not Persisting

**Problem:** Network resets to environment network on page reload

**Solutions:**
1. Check browser's localStorage is not disabled
2. Check browser console for any errors
3. Try clearing localStorage and refreshing

### "Network Mismatch" Warning

**Problem:** See warning about network mismatch

**Causes:**
- App environment network differs from browser localStorage
- Manual reset needed

**Solutions:**
1. Reload page to clear localStorage
2. Or, click the network badge and switch to environment network
3. Or, change `VITE_NETWORK_PASSPHRASE` in `.env` to match your selection

### Wallet Shows Different Network

**Problem:** Freighter shows different network than app

**Causes:**
- Wallet is connected to different network than selected network
- Transactions will fail with "Network Mismatch" error

**Solutions:**
1. Switch Freighter to match the network shown in SAFE-HAVEN header
2. Or, use SAFE-HAVEN network switcher to match your wallet

## Best Practices

1. **Always verify network before making transactions** — Check header badge
2. **Ensure wallet matches app network** — Look for network mismatch warnings
3. **Use testnet for development** — Safer and no real fund requirements
4. **Understand network switching costs** — Different fees on mainnet vs testnet
5. **Document your network choice** — When sharing links, specify which network

## API Reference

### useNetwork Hook

```typescript
const {
  currentNetwork,      // 'testnet' | 'mainnet'
  envNetwork,         // Network from VITE_NETWORK_PASSPHRASE
  isMismatched,       // boolean
  switchNetwork,      // (networkId: NetworkId) => void
  resetToEnvNetwork   // () => void
} = useNetwork()
```

### NetworkId Enum

```typescript
enum NetworkId {
  TESTNET = 'testnet',
  MAINNET = 'mainnet',
}
```

### NetworkConfig Interface

```typescript
interface NetworkConfig {
  id: NetworkId
  name: string
  displayName: string
  color: 'red' | 'green'
  passphrase: string
  rpcUrl: string
  horizonUrl: string
  explorerUrl: string
  nativeToken: string
}
```

## Future Enhancements

Potential improvements:

- [ ] Network history (remember last 3 networks used)
- [ ] Custom network configuration
- [ ] Network-specific contract deployments
- [ ] Automatic network detection from Freighter
- [ ] Network-aware transaction simulation
- [ ] Multi-network dashboard view

## FAQ

**Q: Can I use testnet and mainnet simultaneously?**  
A: No, SAFE-HAVEN can only use one network at a time. The app is environment-configured at deploy time.

**Q: Will switching networks affect my data?**  
A: No, your deposits are stored on the blockchain. Switching networks just changes which blockchain you interact with.

**Q: What if I switch networks but my wallet is on a different network?**  
A: You'll see a "Network Mismatch" warning. Transactions will fail. Switch your wallet to match the app.

**Q: Is network selection stored permanently?**  
A: Yes, in localStorage. It persists across browser sessions until you clear your cache.

**Q: Can I set a default network?**  
A: Yes, set `VITE_NETWORK_PASSPHRASE` in `.env` to your preferred network.

## Support

For questions or issues with network switching:

1. Check this documentation
2. Review browser console for errors
3. Verify wallet configuration matches app network
4. File an issue in the SAFE-HAVEN repository
