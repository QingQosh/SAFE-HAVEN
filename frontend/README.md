# 🖥️ SAFE-HAVEN — Frontend

React + TypeScript + Vite frontend for the [SAFE-HAVEN](../contracts/) Soroban smart contract on Stellar.

## Stack

| Layer | Technology |
|---|---|
| Framework | React 18 + TypeScript |
| Build tool | Vite 5 |
| Styling | Tailwind CSS 3 |
| Stellar SDK | `@stellar/stellar-sdk` v12 |
| Wallet | Freighter browser extension |
| Toasts | `react-hot-toast` |

---

## Features

| Feature | Description |
|---|---|
| 🛡️ Security Education | Built-in tips on wallet security, best practices, and threat warnings |
| 🔐 Wallet connect | Freighter wallet integration with session persistence |
| 🌐 Network switcher | Toggle between testnet and mainnet with persistent selection |
| 🛒 Buy Tokens | Fiat on-ramp via Ramp Network (buy XLM with fiat) |
| 📊 Dashboard | Live view of all your deposits with countdown timers |
| 🔑 Account recovery | Register a recovery contact and start a seven-day, verifiable recovery request |
| 💰 Deposit | Lock any SEP-41 token with custom unlock time and penalty |
| ⬆️ Withdraw | Claim unlocked tokens or cancel early with penalty |
| 🔗 Explorer links | Every address and tx links to Stellar Expert |

---

## Environment Variables

| Variable | Purpose | Required | Example |
|---|---|---|---|
| `VITE_CONTRACT_ID` | Deployed Soroban contract ID (from `stellar contract deploy`) | **Yes** | `CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4` |
| `VITE_NETWORK_PASSPHRASE` | Stellar network passphrase | **Yes** | `Test SDF Network ; September 2015` |
| `VITE_RPC_URL` | Soroban RPC endpoint | **Yes** | `https://soroban-testnet.stellar.org` |
| `VITE_HORIZON_URL` | Horizon endpoint for account queries | **Yes** | `https://horizon-testnet.stellar.org` |
| `VITE_EXPLORER_URL` | Stellar Expert explorer base URL | **Yes** | `https://stellar.expert/explorer/testnet` |
| `VITE_SIMULATION_ACCOUNT` | Source account used for read-only simulations when no wallet is connected. Defaults to the contract ID if unset. Set to a funded account on your target network to avoid synthetic-account fallback. | No | `G...` (any funded Stellar account) |
| `VITE_RAMP_API_KEY` | Ramp Network API key for fiat on-ramp | No | `rampnetwork` (staging) or your production key |
| `VITE_RAMP_ENVIRONMENT` | Ramp environment: `staging` (test mode) or `production` | No | `staging` (default) |

---

## Getting Started

### 1. Prerequisites

- [Node.js 20+](https://nodejs.org/)
- [Freighter wallet](https://freighter.app) browser extension
- A deployed SAFE-HAVEN contract (see [`../contracts/`](../contracts/))

### 2. Install dependencies

```bash
cd frontend
npm install
```

### 3. Configure environment

│   └── recovery.ts       # Frontend-only recovery contacts and timelock state
```bash
cp .env.example .env
```

Edit `.env` and set at minimum:

```env
VITE_CONTRACT_ID=C...   # Your deployed contract address
```

    ├── SettingsPage.tsx  # Recovery contacts and account recovery
### 4. Run dev server

```bash
npm run dev
```

Open [http://localhost:5173](http://localhost:5173).

## Public documentation

- [User onboarding guide](../docs/USER_ONBOARDING.md)
- [Development roadmap](../docs/ROADMAP.md)
- [Operator performance guide](../docs/OPERATOR_PERFORMANCE.md)

### 5. Build for production

```bash
npm run build
npm run preview   # preview the production build locally
```

---

## Project Structure

```
src/
├── main.tsx              # React entry point
├── App.tsx               # Root component, tab routing
├── config.ts             # Contract ID, RPC URLs, constants
├── types.ts              # Shared TypeScript types
├── index.css             # Tailwind base + custom components
│
├── context/
│   ├── WalletContext.tsx  # Freighter wallet state + signing
│   └── NetworkContext.tsx # Network selection state + persistence
│
├── hooks/
│   ├── useDeposits.ts       # Load deposits for connected address
│   ├── useContractInfo.ts   # Contract admin/paused/constants
│   └── useRampOnramp.ts     # Ramp Network SDK initialization
│
├── lib/
│   ├── stellar.ts     # Contract reads + tx builders
│   ├── format.ts      # Stroops, dates, countdown, BPS
│   ├── networks.ts    # Network configs + utilities (testnet/mainnet)
│   └── security.ts    # Security tips + checklist data
│
├── context/
│   ├── WalletContext.tsx  # Freighter wallet state + signing
│   ├── NetworkContext.tsx # Network selection state + persistence
│   └── SecurityContext.tsx # Security checklist tracking
│
├── components/
│   ├── Header.tsx              # Top nav + wallet + security + network + buy tokens
│   ├── NetworkSwitcher.tsx     # Network selection dropdown
│   ├── NetworkDisplay.tsx      # Network badge display
│   ├── SecurityTipsModal.tsx   # Security education modal
│   ├── SmallBalanceWarning.tsx # Low balance alert
│   ├── PublicWifiWarning.tsx   # Public WiFi detection
│   ├── BuyTokensModal.tsx      # Ramp Network widget modal
│   ├── TabNav.tsx              # Page tab switcher
│   ├── DepositCard.tsx         # Single deposit UI card
│   └── TxStatusBadge.tsx       # Signing → submitting → confirmed
│
└── pages/
    ├── Dashboard.tsx     # My vaults overview
    ├── DepositPage.tsx   # New deposit form
    ├── WithdrawPage.tsx  # Withdraw / cancel form
    └── AdminPage.tsx     # Admin controls
```

---

## Security Education

SAFE-HAVEN includes comprehensive security education features to help you protect your assets.

### Features

- **Security Tips Modal** — 13+ best practices organized by category and priority
- **Small Balance Warning** — Alerts when wallet balance < $10 USD
- **Public WiFi Detection** — Warns when connecting from public networks
- **Security Tracking** — Tracks which recommendations you've reviewed

### Quick Start

1. Click the **"Security"** button in the header (🛡️)
2. Browse security tips by category
3. Expand any tip to read detailed guidance
4. Look for 🔴 high-priority recommendations
5. ✓ marks indicate immediately actionable steps

For detailed information, see [**SECURITY_EDUCATION.md**](./SECURITY_EDUCATION.md) and [**SECURITY_QUICKSTART.md**](./SECURITY_QUICKSTART.md).

---

## Network Switching

SAFE-HAVEN supports switching between Stellar **testnet** and **mainnet** directly from the app header.

### Features

- **Persistent Selection** — Network choice saved in localStorage
- **Visual Indicators** — Red badge for testnet, green for mainnet
- **Quick Switcher** — Dropdown menu to change networks
- **Warning Alerts** — Notifications when switching networks
- **Network Info** — Shows RPC URL and Explorer links

### Quick Start

1. Click the network badge in the header (red = testnet, green = mainnet)
2. Select the network you want
3. Network switches and selection is saved

For detailed docs, see [**NETWORK_SWITCHING.md**](./NETWORK_SWITCHING.md) and [**NETWORK_QUICKSTART.md**](./NETWORK_QUICKSTART.md).

---

## Fiat On-Ramp (Ramp Network)

SAFE-HAVEN includes integrated fiat on-ramp powered by **Ramp Network**, allowing users to purchase XLM using fiat currency (credit/debit cards, bank transfers, etc.).

### Quick Setup

1. Add to `.env`:
   ```env
   VITE_RAMP_API_KEY=rampnetwork        # Use for testing
   VITE_RAMP_ENVIRONMENT=staging         # Use staging for testing
   ```

2. The "Buy Tokens" button will appear in the header when a wallet is connected.

For detailed setup, configuration, and troubleshooting, see [**RAMP_ONRAMP.md**](./RAMP_ONRAMP.md).

---

Change the values in `.env`:

| Variable | Testnet | Mainnet |
|---|---|---|
| `VITE_NETWORK_PASSPHRASE` | `Test SDF Network ; September 2015` | `Public Global Stellar Network ; September 2015` |
| `VITE_RPC_URL` | `https://soroban-testnet.stellar.org` | `https://soroban.stellar.org` |
| `VITE_HORIZON_URL` | `https://horizon-testnet.stellar.org` | `https://horizon.stellar.org` |
| `VITE_EXPLORER_URL` | `https://stellar.expert/explorer/testnet` | `https://stellar.expert/explorer/public` |

---

## Wallet Support

Currently the frontend integrates with **Freighter** directly via `window.freighter`.

To extend with more wallets (Albedo, xBull, Lobstr, etc.), the `WalletContext.tsx` signing logic can be replaced with `@creit.tech/stellar-wallets-kit` — the package is already included in `package.json`.
