# Project Structure

This repository is split into two independent layers:

```
SAFE-HAVEN/
├── contracts/                  ← Rust / Soroban smart contract (backend)
│   └── safe-haven/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs          # Crate root
│           ├── contract.rs     # All public entry points
│           ├── types.rs        # VaultKey (incl. DepositByLedger), VaultEntry, LedgerVaultEntry, constants
│           ├── errors.rs       # VaultError enum (12 codes)
│           ├── events.rs       # Event emission helpers
│           ├── storage.rs      # Persistent storage + TTL helpers
│           └── test.rs         # 48+ unit tests
│
├── frontend/                   ← React + TypeScript + Vite (UI layer)
│   ├── src/
│   │   ├── App.tsx             # Root component, tab routing
│   │   ├── config.ts           # Contract ID, RPC URLs, constants
│   │   ├── types.ts            # Shared TypeScript types
│   │   ├── context/
│   │   │   └── WalletContext.tsx    # Freighter wallet state + signing
│   │   ├── hooks/
│   │   │   ├── useDeposits.ts       # Load deposits for connected wallet
│   │   │   └── useContractInfo.ts   # Admin/paused/constants
│   │   ├── lib/
│   │   │   ├── stellar.ts      # Contract reads + tx builders
│   │   │   └── format.ts       # Stroops, dates, countdown, BPS
│   │   ├── components/
│   │   │   ├── Header.tsx      # Top nav + wallet connect button
│   │   │   ├── TabNav.tsx      # Page tab switcher
│   │   │   ├── DepositCard.tsx # Single deposit UI card
│   │   │   └── TxStatusBadge.tsx # Signing → submitting → confirmed
│   │   └── pages/
│   │       ├── Dashboard.tsx   # My vaults overview
│   │       ├── DepositPage.tsx # New deposit form
│   │       ├── WithdrawPage.tsx # Withdraw / cancel form
│   │       └── AdminPage.tsx   # Admin controls
│   ├── .env.example            # Environment variable template
│   ├── package.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   └── README.md               # Frontend-specific docs
│
├── Cargo.toml                  # Rust workspace
├── Cargo.lock
├── Makefile                    # Build / test / lint / deploy helpers
├── rust-toolchain.toml
├── README.md                   # Contract documentation
└── STRUCTURE.md                # This file
```

## Quick Start

### Smart Contract

```bash
# Build the WASM
make build

# Run all tests
make test

# Deploy to testnet
export SOROBAN_SECRET_KEY=S...
make deploy-testnet
```

### Frontend

```bash
cd frontend
npm install
cp .env.example .env      # paste your contract ID into VITE_CONTRACT_ID
npm run dev               # → http://localhost:5173
```

See [`frontend/README.md`](./frontend/README.md) for the full frontend guide.

## Architecture Overview

```
Browser (React + Freighter)
        │
        │  signTransaction(xdr)
        ▼
Freighter Wallet Extension
        │
        │  signed XDR
        ▼
 Soroban RPC Server
        │
        │  sendTransaction / simulateTransaction
        ▼
SAFE-HAVEN Contract (Soroban / Stellar)
        │
        ├─ deposit()           → creates VaultEntry in persistent storage
        ├─ deposit_by_ledger()  → creates LedgerVaultEntry keyed by ledger sequence
        ├─ withdraw()          → validates timestamp/ledger, removes entry, transfers tokens
        ├─ cancel_deposit()    → early exit with configurable penalty
        └─ emergency_withdraw() → admin-only recovery (funds → depositor)
```
