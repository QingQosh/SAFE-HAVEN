# Getting Started with SAFE-HAVEN

This guide takes a new developer from a clean machine to a fully verified
build — contract compiled, tests passing, and frontend running — in under
30 minutes. A deployed contract and funded wallet are only required to use
the application against a live network; everything before that section works
offline.

---

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [OS-Specific Setup](#2-os-specific-setup)
   - [macOS](#macos)
   - [Linux (Debian / Ubuntu)](#linux-debian--ubuntu)
   - [Windows (WSL 2)](#windows-wsl-2)
3. [Clone the Repository](#3-clone-the-repository)
4. [Install Dev Tools](#4-install-dev-tools)
5. [Build and Test the Contract](#5-build-and-test-the-contract)
6. [Set Up the Frontend](#6-set-up-the-frontend)
7. [Verify Your Setup](#7-verify-your-setup)
8. [Full Local Dev Environment](#8-full-local-dev-environment)
9. [Deploy to Testnet](#9-deploy-to-testnet)
10. [Development Workflow Tips](#10-development-workflow-tips)
11. [Common Commands Reference](#11-common-commands-reference)
12. [Troubleshooting](#12-troubleshooting)
13. [Useful Resources](#13-useful-resources)

---

## 1. Prerequisites

You need the following tools before you can build or run anything. Install
each one and confirm the version check passes.

| Tool | Minimum Version | Used For |
|---|---|---|
| Git | Any current release | Cloning and branching |
| Rust + Cargo | Stable (MSRV 1.81) | Compiling the Soroban contract |
| `wasm32-unknown-unknown` target | — | WASM compilation of the contract |
| Stellar CLI (`stellar`) | 26.x | Building, deploying, and invoking contracts |
| Node.js + npm | Node.js 20 LTS | Installing and building the frontend |
| Freighter browser extension | Current | Signing transactions in the UI |
| `jq` (optional) | Any | Parsing JSON in local smoke tests |

> The repository ships a `rust-toolchain.toml` that pins the stable Rust
> channel and the `wasm32-unknown-unknown` target. Running any `cargo`
> command inside the repo automatically activates the correct toolchain via
> `rustup`. You do **not** need to manage the toolchain version manually.

Run the following after installing each tool to confirm they are on your PATH:

```bash
git --version
rustc --version
cargo --version
rustup target list --installed | grep wasm32-unknown-unknown
stellar --version
node --version
npm --version
```

Expected output (versions may differ):

```
git version 2.x.x
rustc 1.8x.x (...)
cargo 1.8x.x (...)
wasm32-unknown-unknown (installed)
stellar 26.x.x
v20.x.x
10.x.x
```

---

## 2. OS-Specific Setup

### macOS

Install [Homebrew](https://brew.sh) if you don't have it, then:

```bash
# Install Git and Node.js
brew install git node

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Add the WASM compilation target
rustup target add wasm32-unknown-unknown

# Install the Stellar CLI
# Follow the official guide at https://developers.stellar.org/docs/tools/developer-tools/stellar-cli/install
# The recommended method on macOS is via Homebrew or direct binary download.
# Example (check docs for latest):
brew install stellar-cli

# Optionally install jq for smoke tests
brew install jq
```

Restart your terminal if the installers updated your shell profile.

---

### Linux (Debian / Ubuntu)

```bash
# System packages
sudo apt update
sudo apt install -y build-essential curl git jq pkg-config libssl-dev

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# WASM target
rustup target add wasm32-unknown-unknown

# Node.js 20+ via NodeSource (or use nvm)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Confirm Node.js version
node --version   # should be v20.x.x or higher

# Stellar CLI — follow the official installation guide:
# https://developers.stellar.org/docs/tools/developer-tools/stellar-cli/install
# Quick install via pre-built binary (check the page for the latest version):
curl -sSL "https://github.com/stellar/stellar-cli/releases/download/v26.0.0/stellar-cli-v26.0.0-x86_64-unknown-linux-gnu.tar.gz" \
  | tar -xz -C ~/.local/bin stellar
# Ensure ~/.local/bin is in your PATH, or use /usr/local/bin instead.
```

---

### Windows (WSL 2)

The Soroban toolchain is not natively supported on Windows. Use
[WSL 2](https://learn.microsoft.com/windows/wsl/install) with Ubuntu:

```powershell
# In PowerShell (as Administrator)
wsl --install -d Ubuntu
```

After WSL is set up, open the Ubuntu terminal and follow the
**Linux (Debian / Ubuntu)** instructions above. Clone the repository
**inside** the WSL filesystem (e.g., `~/projects/SAFE-HAVEN`) rather than
under `/mnt/c/` — I/O performance across the filesystem boundary is
significantly worse.

**Freighter** runs in your Windows browser. The Vite dev server started in
WSL is accessible at `http://localhost:5173` from Windows automatically.

> **Note:** `make dev` starts a local Stellar node. The node listens on
> `http://localhost:8000` inside WSL, which is also accessible from the
> Windows browser on the same machine.

---

## 3. Clone the Repository

```bash
git clone https://github.com/kenedybok3/SAFE-HAVEN.git
cd SAFE-HAVEN
```

Verify the clone is clean:

```bash
git status
# On branch main
# nothing to commit, working tree clean
```

Activate the pre-commit hook that runs formatting and lint checks locally
before each commit:

```bash
git config core.hooksPath .githooks
```

This hook runs `cargo fmt --check` and `cargo clippy` so you catch issues
locally before CI does.

---

## 4. Install Dev Tools

One command installs all recommended Rust tools:

```bash
make install-tools
```

This installs:

| Tool | Purpose |
|---|---|
| `cargo-watch` | Re-run tests automatically on file save (`make watch`) |
| `soroban-cli` (as `stellar`) | Build, deploy, and invoke Soroban contracts |
| `cargo-audit` | Scan dependencies for known vulnerabilities |
| `cargo-deny` | Enforce license and ban policy on dependencies |

If any tool is already installed at the right version, the command skips it
safely.

---

## 5. Build and Test the Contract

```bash
# Compile to WASM (release mode)
make build

# Run all 48+ unit tests (no network or wallet required)
make test
```

Both commands should complete without errors. The unit tests run natively
using Soroban's test environment — they do not require a running Stellar
node or a deployed contract.

To continuously run tests while you edit code:

```bash
make watch
```

For the full CI-equivalent check (formatting + linting + tests + security
audit + license check):

```bash
make check
```

> `make check` requires `cargo-audit` and `cargo-deny` which are installed
> by `make install-tools`. Run install-tools first if you skipped step 4.

---

## 6. Set Up the Frontend

The frontend needs a deployed contract address and matching network
endpoints. For quick setup verification, you can point it at the testnet
contract if one is available, or use `make dev` (see
[section 8](#8-full-local-dev-environment)) to run a full local stack.

```bash
cd frontend
npm install
cp .env.example .env
```

Open `.env` in your editor and fill in the values:

```env
# Required — your deployed contract address (starts with C, 56 chars)
VITE_CONTRACT_ID=C...

# Network identity — must match the network your contract is on
VITE_NETWORK_PASSPHRASE=Test SDF Network ; September 2015

# RPC endpoint for Soroban contract calls
VITE_RPC_URL=https://soroban-testnet.stellar.org

# Horizon endpoint for account lookups
VITE_HORIZON_URL=https://horizon-testnet.stellar.org

# Explorer base URL for transaction links
VITE_EXPLORER_URL=https://stellar.expert/explorer/testnet

# Optional: a funded account used for read-only simulations when no wallet is
# connected. Defaults to the contract address if not set.
# VITE_SIMULATION_ACCOUNT=G...
```

For **mainnet**, replace the values:

| Variable | Mainnet value |
|---|---|
| `VITE_NETWORK_PASSPHRASE` | `Public Global Stellar Network ; September 2015` |
| `VITE_RPC_URL` | `https://soroban.stellar.org` |
| `VITE_HORIZON_URL` | `https://horizon.stellar.org` |
| `VITE_EXPLORER_URL` | `https://stellar.expert/explorer/public` |

> **Never commit `.env`** — it is in `.gitignore`. Never put secret keys in
> any `.env` file checked into version control.

Start the dev server:

```bash
npm run dev
```

Open [http://localhost:5173](http://localhost:5173). Connect Freighter and
make sure it is set to the same network as your `.env`.

---

## 7. Verify Your Setup

Run each check below. All should pass before you start developing.

### Contract

```bash
# Should complete with 0 failures
make test

# Should print the WASM size and confirm it is under 64 KB
make build && make optimize && make check-wasm-size

# Should pass all six checks
make check
```

### Frontend

```bash
cd frontend

# Should produce a dist/ directory with no TypeScript errors
npm run build

# Should complete with no errors
npm run typecheck
```

If every command above passes, your environment is correctly set up.

---

## 8. Full Local Dev Environment

`make dev` builds the contract, starts a local Stellar node, deploys the
contract, writes `frontend/.env`, and starts the Vite dev server — all in
one command:

```bash
make dev
```

What it does, step by step:

1. Compiles the contract to WASM (`make build`).
2. Starts a local Stellar standalone node on `http://localhost:8000`.
3. Creates a `dev-deployer` key and funds it via the local network.
4. Deploys the contract and calls `initialize`.
5. Writes the contract ID and local network values into `frontend/.env`.
6. Runs `npm install` and starts the Vite dev server.

The local Stellar node runs in the background. Stop it when you are done:

```bash
make dev-stop
```

> **Prerequisites for `make dev`:** `stellar` CLI must be installed and on
> your PATH. The local node requires Docker or the `stellar` binary's
> built-in quickstart support — check the Stellar CLI docs for your
> platform.

---

## 9. Deploy to Testnet

To deploy to Stellar testnet, set your secret key and run:

```bash
export SOROBAN_SECRET_KEY=S...   # your Stellar secret key (starts with S)
make deploy-testnet
```

This compiles, optimizes, and deploys the WASM, then records the contract
ID in `deploy_testnet.log`. The script in `scripts/deploy_testnet.sh` runs
the full deployment sequence.

**Fund your testnet account** if you haven't already:

```bash
curl "https://friendbot.stellar.org/?addr=YOUR_G_ADDRESS"
```

After deployment, update `frontend/.env` with the new contract ID and set
`VITE_NETWORK_PASSPHRASE` to the testnet passphrase.

---

## 10. Development Workflow Tips

### Branch naming

Always create a focused branch from `main`:

```
feat/<short-description>   # new feature
fix/<short-description>    # bug fix
docs/<short-description>   # documentation only
chore/<short-description>  # tooling, dependencies
```

### Iterating on the contract

```bash
# Edit a .rs file, then in another terminal:
make watch        # re-runs tests on every save

# Or manually:
make test         # fast feedback loop
make lint         # catch Clippy warnings early
```

### Iterating on the frontend

```bash
cd frontend
npm run dev       # hot-reloads on every save
npm run test      # vitest unit tests
npm run typecheck # TypeScript check without building
```

### Before opening a PR

```bash
# From the repo root
make check        # full Rust CI check

# From frontend/
npm run build
npm run typecheck
```

Both must pass cleanly. See [CONTRIBUTING.md](CONTRIBUTING.md) for the full
checklist.

### Caching `env.ledger().timestamp()`

When writing contract code, cache host function results in local variables
rather than calling them multiple times. Every host call has a non-trivial
instruction cost in Soroban:

```rust
// Good
let now = env.ledger().timestamp();
if now < entry.unlock_time { ... }
// use `now` again without a second host call

// Bad — calls the host twice
if env.ledger().timestamp() < entry.unlock_time { ... }
let elapsed = env.ledger().timestamp() - start;
```

---

## 11. Common Commands Reference

### Contract (run from repo root)

| Command | What it does |
|---|---|
| `make build` | Compile contract to WASM (release) |
| `make test` | Run all unit tests (no network needed) |
| `make watch` | Auto-rerun tests on file save |
| `make fmt` | Format all Rust source files |
| `make fmt-check` | Check formatting without modifying files |
| `make lint` | Run Clippy (fails on any warning) |
| `make check` | `fmt-check` + `lint` + `test` + `audit` + `deny` |
| `make audit` | Scan dependencies for vulnerabilities |
| `make deny` | Check license and ban policy |
| `make optimize` | Optimize WASM with Stellar CLI |
| `make check-wasm-size` | Fail if optimized WASM > 64 KB |
| `make doc` | Build and open Rust API docs |
| `make clean` | Remove all build artifacts |
| `make install-tools` | Install all recommended dev tools |
| `make dev` | Full local stack (build + deploy + frontend) |
| `make dev-stop` | Stop the local Stellar node |
| `make deploy-testnet` | Deploy optimized WASM to Stellar testnet |
| `make smoke-test-local` | End-to-end smoke test against local node |

### Frontend (run from `frontend/`)

| Command | What it does |
|---|---|
| `npm run dev` | Start Vite dev server (`localhost:5173`) |
| `npm run build` | Production build to `dist/` |
| `npm run preview` | Preview production build locally |
| `npm run typecheck` | TypeScript check without building |
| `npm run lint` | ESLint check |
| `npm run test` | Run Vitest unit tests |
| `npm run test:watch` | Vitest in watch mode |

---

## 12. Troubleshooting

### `stellar: command not found`

The Stellar CLI is not on your PATH. Re-run the installation for your OS
(see [section 2](#2-os-specific-setup)), confirm the binary was written to a
directory on `$PATH`, and restart your shell. Do not assume the older
`soroban` binary name is still available in all installations.

### `wasm32-unknown-unknown` target missing

```bash
rustup target add wasm32-unknown-unknown
rustup show   # confirm the target appears
```

The `rust-toolchain.toml` should add it automatically, but if you bypassed
`rustup` you may need to add it manually.

### Rust compilation errors about linker or `pkg-config`

On Linux, you may be missing system build tools:

```bash
sudo apt install -y build-essential pkg-config libssl-dev
```

On macOS, install Xcode command-line tools:

```bash
xcode-select --install
```

### `npm install` or `npm ci` fails

- Ensure you are running Node.js 20+: `node --version`
- Delete `frontend/node_modules` and `frontend/package-lock.json`, then
  retry: `npm install`
- In CI or clean environments prefer `npm ci` over `npm install`

### Frontend shows "network mismatch"

Every value in `frontend/.env` must refer to the same network, and the
Freighter wallet must be connected to that same network. Common mismatches:

- `VITE_NETWORK_PASSPHRASE` set to testnet but Freighter is on mainnet
- `VITE_CONTRACT_ID` from one network but `VITE_RPC_URL` from another

Copy fresh values from `frontend/.env.example` and reconfigure from scratch.

### Transactions fail or stay pending

- Check that your wallet account is funded on the target network.
- For testnet accounts, use [Friendbot](https://friendbot.stellar.org/).
- Verify the contract is initialized (`stellar contract invoke ... -- is_initialized`).
- Check that the Soroban RPC endpoint is healthy (try opening the URL in a
  browser — it should return a JSON response).
- Wait a few seconds and retry; Stellar testnet can occasionally be slow.

### `make check` fails on `cargo audit`

A dependency has a known CVE. Either:

1. Update the dependency: `cargo update <crate>` and re-run `make audit`.
2. Add an ignore entry in `deny.toml` if the advisory does not apply to
   your usage, with a comment explaining why.

### `make dev` — local node doesn't start

The local Stellar node requires Docker or `stellar network` quickstart
support. Confirm Docker is running on your machine, or follow the Stellar
CLI docs for the `stellar network start local` command:

```bash
docker info    # must succeed
stellar network start local --background
```

### Port 5173 already in use

Stop the conflicting process, or start Vite on a different port:

```bash
npm run dev -- --port 5174
```

### Pre-commit hook not running

Activate the hook:

```bash
git config core.hooksPath .githooks
```

The hook must be executable:

```bash
chmod +x .githooks/pre-commit
```

---

## 13. Useful Resources

### Project docs

| Document | Purpose |
|---|---|
| [README.md](README.md) | Project overview, contract API, error codes |
| [frontend/README.md](frontend/README.md) | Frontend setup and architecture |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute, git workflow, PR checklist |
| [TESTING_GUIDE.md](TESTING_GUIDE.md) | Contract and frontend testing in depth |
| [SECURITY.md](SECURITY.md) | Security policy and responsible disclosure |
| [STRUCTURE.md](STRUCTURE.md) | Detailed directory layout |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [docs/adr/](docs/adr/README.md) | Architecture Decision Records |

### Stellar ecosystem

| Link | Purpose |
|---|---|
| [Stellar Developer Docs](https://developers.stellar.org/docs) | Official Soroban and Stellar docs |
| [Stellar CLI reference](https://developers.stellar.org/docs/tools/developer-tools/stellar-cli) | All `stellar` CLI commands |
| [Soroban SDK docs](https://docs.rs/soroban-sdk/latest/soroban_sdk/) | Rust SDK API reference |
| [Stellar Expert (testnet)](https://stellar.expert/explorer/testnet) | Block explorer for testnet |
| [Stellar Expert (mainnet)](https://stellar.expert/explorer/public) | Block explorer for mainnet |
| [Friendbot](https://friendbot.stellar.org/) | Fund a testnet account |
| [Soroban Playground](https://www.stellar.org/developers/tools) | Interactive contract testing |

### External tools referenced in this project

| Link | Purpose |
|---|---|
| [Freighter](https://freighter.app) | Browser wallet for Stellar |
| [cargo-audit](https://crates.io/crates/cargo-audit) | Vulnerability scanning |
| [cargo-deny](https://embarkstudios.github.io/cargo-deny/) | Dependency policy |
| [cargo-watch](https://crates.io/crates/cargo-watch) | File-watch test runner |
| [Conventional Commits](https://www.conventionalcommits.org/) | Commit message format used by this project |
