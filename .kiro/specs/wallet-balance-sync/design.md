# Design Document: Wallet Balance Sync

## Overview

The Wallet Balance Sync feature adds a continuous, low-overhead mechanism to keep the displayed XLM balance up-to-date while the SAFE-HAVEN app is open. Today the `SmallBalanceWarning` component performs a one-shot Horizon fetch on wallet connect; nothing else re-checks the balance. After this feature ships, `WalletContext` will own the canonical balance state and keep it fresh via a new `balanceSyncService` module.

The design is split into three layers:

1. **`frontend/src/lib/balanceSyncService.ts`** — pure service module; owns all fetch logic, timers, WebSocket lifecycle, and failure counting. Has no React dependency.
2. **`frontend/src/context/WalletContext.tsx`** — consumes the service, exposes balance state to the rest of the app via six new context fields.
3. **`frontend/src/components/Header.tsx`** — renders the refresh button, last-updated timestamp, and stale/error indicators using the new context fields.

---

## Architecture

```mermaid
flowchart TD
    subgraph Browser
        direction TB
        PVA["Page Visibility API\n(document.visibilitychange)"]
        WS["WebSocket\n(optional)"]
    end

    subgraph balanceSyncService["balanceSyncService.ts"]
        direction TB
        CAP["capabilityCheck()\ndetects WS support"]
        POLL["Poller\nsetInterval 5 s"]
        WSSUB["WS Subscription\n(optional path)"]
        FETCH["fetchBalance(address)\nHorizon /accounts/{addr}"]
        FAIL["consecutiveFailures\ncounter"]
        CAP -->|supported| WSSUB
        CAP -->|not supported| POLL
        WSSUB -->|account-change event| FETCH
        POLL --> FETCH
        FETCH --> FAIL
    end

    subgraph WalletContext["WalletContext.tsx"]
        STATE["balance\nbalanceError\nisBalanceStale\nlastBalanceUpdate\nisRefreshingBalance"]
    end

    subgraph Header["Header.tsx"]
        BTN["RefreshButton"]
        TS["TimestampDisplay\n(setInterval 1 s)"]
        WARN["StaleWarning / ErrorBanner"]
    end

    PVA -->|visibilitychange| balanceSyncService
    WS <-->|connect/close| WSSUB
    balanceSyncService -->|callbacks| WalletContext
    WalletContext -->|context| Header
    BTN -->|calls refreshBalance()| WalletContext
```

### Data flow summary

1. `WalletProvider` calls `balanceSyncService.startSync(address, callbacks)` when a wallet connects.
2. The service performs an immediate fetch, then sets a 5-second polling interval (or establishes a WebSocket if the capability check passes).
3. Successful fetches invoke `onSuccess(balance, timestamp)`; failures invoke `onError(message, consecutiveCount)`.
4. `WalletProvider` maps those callbacks to `setState` calls, updating the six new context fields.
5. `Header` reads the context and renders reactively.

---

## New Module: `frontend/src/lib/balanceSyncService.ts`

### Public API

```ts
/** Callbacks supplied by WalletContext when starting a sync session */
export interface BalanceSyncCallbacks {
  /** Called after every successful Horizon fetch */
  onSuccess: (balance: string, timestamp: Date) => void
  /** Called after every failed fetch; consecutiveCount is the running failure streak */
  onError: (message: string, consecutiveCount: number) => void
}

/** Options for startSync — separated from callbacks for clarity */
export interface BalanceSyncOptions {
  /** Polling interval in milliseconds (default: 5000) */
  pollIntervalMs?: number
  /** Debounce delay in milliseconds for address changes (default: 300) */
  debounceMs?: number
  /** Horizon base URL — defaults to CONFIG.HORIZON_URL */
  horizonUrl?: string
}

/**
 * Start (or restart) the balance sync for a given wallet address.
 * Safe to call repeatedly: any existing sync session for a different address
 * is cleanly stopped before the new one starts.
 *
 * @returns A stop function for the caller to invoke on cleanup.
 */
export function startSync(
  address: string,
  callbacks: BalanceSyncCallbacks,
  options?: BalanceSyncOptions,
): () => void

/**
 * Immediately stop all polling and close any open WebSocket.
 * Safe to call even if no sync is active.
 */
export function stopSync(): void

/**
 * Trigger an immediate, on-demand balance fetch outside the normal poll cycle.
 * The same onSuccess / onError callbacks registered in the current startSync
 * call are used. Resolves when the fetch settles.
 *
 * @throws if called when no sync session is active
 */
export function refreshBalance(): Promise<void>

/**
 * Check at runtime whether the configured Soroban RPC endpoint supports
 * WebSocket event streaming (the optional fast-path).
 *
 * Returns true only if a WebSocket can be opened to the RPC streaming
 * endpoint and a valid subscription acknowledgement is received within 3 s.
 * All errors are caught — the function never throws.
 */
export async function checkWebSocketCapability(rpcUrl: string): Promise<boolean>
```

### Internal State

All mutable state lives in module-level variables (the module is a singleton because only one wallet can be active at a time):

```ts
// Opaque handle to the current polling interval
let _pollTimer: ReturnType<typeof setInterval> | null = null

// AbortController for the currently in-flight Horizon fetch; null when idle
let _abortController: AbortController | null = null

// Active WebSocket connection (optional fast path)
let _ws: WebSocket | null = null

// Consecutive fetch failure streak (resets to 0 on any success)
let _consecutiveFailures = 0

// Debounce timer for rapid address changes
let _debounceTimer: ReturnType<typeof setTimeout> | null = null

// Whether polling is suspended due to tab visibility
let _paused = false

// The address currently being synced (null when idle)
let _currentAddress: string | null = null

// Stored callbacks for the current session (used by refreshBalance())
let _callbacks: BalanceSyncCallbacks | null = null

// Resolved options for the current session
let _options: Required<BalanceSyncOptions> = { ... }
```

### Key Algorithms

#### `startSync(address, callbacks, options?)`

```
1. Clear any existing debounce timer.
2. Schedule the actual _doStartSync(address, callbacks, options) after debounceMs
   (default 300 ms) to absorb rapid consecutive calls.
```

#### `_doStartSync(address, callbacks, options)`

```
1. Call stopSync() to clean up any existing session.
2. Store address, callbacks, and resolved options in module state.
3. Register a `visibilitychange` listener (idempotent — only added once).
4. Call _fetchAndNotify(address) immediately (no wait for first interval).
5. Attempt checkWebSocketCapability(rpcUrl):
   a. If true  → call _startWebSocket(address, callbacks)
      WebSocket path: no polling interval needed; polling is the fallback only.
   b. If false → call _startPoller(address, callbacks, intervalMs)
```

#### `_startPoller(address, callbacks, intervalMs)`

```
1. Set _pollTimer = setInterval(() => {
     if (_paused) return          // tab hidden — skip
     if (_abortController) return // previous fetch still in flight — skip
     _fetchAndNotify(address)
   }, intervalMs)
```

#### `_fetchAndNotify(address)`

```
1. Create a new AbortController; store in _abortController.
2. Fetch `${horizonUrl}/accounts/${address}` with the AbortSignal.
3. On success:
   a. Parse the native balance string from balances array.
   b. Reset _consecutiveFailures = 0.
   c. Clear _abortController = null.
   d. Call callbacks.onSuccess(balanceString, new Date()).
4. On failure (network error or non-2xx HTTP):
   a. Clear _abortController = null.
   b. Increment _consecutiveFailures.
   c. Call callbacks.onError(errorMessage, _consecutiveFailures).
   Note: AbortError (from stopSync) is silently swallowed — not counted as failure.
```

#### `stopSync()`

```
1. Clear _debounceTimer.
2. Clear _pollTimer.
3. Abort _abortController (if any).
4. Close _ws (if any) with code 1000.
5. Remove visibilitychange listener.
6. Reset all module state to initial values.
```

#### `refreshBalance()`

```
1. If no active session (_currentAddress is null), throw Error("No active sync session").
2. If _abortController is non-null (fetch in flight), wait for it by awaiting a
   re-fetch after clearing it — or simply return early (no duplicate).
   Implementation choice: return early if in-flight; WalletContext will set
   isRefreshingBalance=false on the next onSuccess/onError callback.
3. Await _fetchAndNotify(_currentAddress).
```

#### `_startWebSocket(address, callbacks)`

```
1. Construct ws = new WebSocket(wsUrl) where wsUrl is derived from CONFIG.RPC_URL
   by replacing the protocol (https → wss, http → ws).
2. On ws.open: send a subscription message for the account address.
3. On ws.message: parse event; if it's an account-change event for `address`,
   call _fetchAndNotify(address).
4. On ws.error / ws.close (unexpected, code !== 1000):
   a. Log a console.warn.
   b. _ws = null.
   c. Fall back: _startPoller(address, callbacks, options.pollIntervalMs).
```

#### Page Visibility Handling

```
// Registered once via addEventListener('visibilitychange', _onVisibilityChange)

function _onVisibilityChange() {
  if (document.hidden) {
    _paused = true
    // Don't clear the interval — just gate inside it with the _paused flag.
  } else {
    _paused = false
    // Resume immediately: trigger a fetch right now without waiting for next tick.
    if (_currentAddress && !_abortController) {
      _fetchAndNotify(_currentAddress)
    }
  }
}
```

### Horizon Response Parsing

```ts
// The raw shape returned by GET /accounts/{address}
interface HorizonBalance {
  asset_type: string
  balance: string // decimal string e.g. "1234.5000000"
}

function parseNativeBalance(data: { balances: HorizonBalance[] }): string {
  const native = data.balances.find((b) => b.asset_type === 'native')
  return native?.balance ?? '0.0000000'
}
```

---

## WalletContext Changes

### New Fields on `WalletContextValue`

```ts
interface WalletContextValue {
  // --- existing fields (unchanged) ---
  wallet: WalletInfo | null
  wallets: WalletInfo[]
  isConnecting: boolean
  isRestoringSession: boolean
  networkMismatch: boolean
  connect: () => Promise<void>
  disconnect: () => void
  signTransaction: (xdr: string) => Promise<SigningResult>

  // --- new balance sync fields ---
  /** Native XLM balance as a decimal string (e.g. "1234.5000000"), or null before first fetch */
  balance: string | null
  /** Human-readable error message when the last fetch failed; null when healthy */
  balanceError: string | null
  /** True when the displayed balance is from a prior successful fetch and the latest attempt failed */
  isBalanceStale: boolean
  /** Wall-clock Date of the most recent successful balance fetch; null before first fetch */
  lastBalanceUpdate: Date | null
  /** Triggers an immediate on-demand fetch (debounced by the service layer) */
  refreshBalance: () => Promise<void>
  /** True while a manual refreshBalance() call is in flight */
  isRefreshingBalance: boolean
}
```

### State Variables to Add in `WalletProvider`

```ts
const [balance, setBalance]                     = useState<string | null>(null)
const [balanceError, setBalanceError]           = useState<string | null>(null)
const [isBalanceStale, setIsBalanceStale]       = useState(false)
const [lastBalanceUpdate, setLastBalanceUpdate] = useState<Date | null>(null)
const [isRefreshingBalance, setIsRefreshingBalance] = useState(false)
```

### Lifecycle Integration

Add a `useEffect` that depends on `wallet?.address`:

```ts
useEffect(() => {
  if (!wallet?.address) {
    stopSync()
    // Reset all balance state on disconnect
    setBalance(null)
    setBalanceError(null)
    setIsBalanceStale(false)
    setLastBalanceUpdate(null)
    setIsRefreshingBalance(false)
    return
  }

  const stop = startSync(
    wallet.address,
    {
      onSuccess: (bal, ts) => {
        setBalance(bal)
        setLastBalanceUpdate(ts)
        setBalanceError(null)
        setIsBalanceStale(false)
        setIsRefreshingBalance(false)
      },
      onError: (msg, count) => {
        setBalanceError(msg)
        setIsBalanceStale(true)
        setIsRefreshingBalance(false)
        if (count === 3) {
          toast.error('Balance sync is failing — displayed balance may be stale.', {
            id: 'balance-sync-error', // deduplicate via react-hot-toast id
          })
        }
      },
    },
  )

  return stop  // cleanup on address change or unmount
}, [wallet?.address])
```

### `refreshBalance` callback

```ts
const refreshBalance = useCallback(async () => {
  if (!wallet?.address) return
  setIsRefreshingBalance(true)
  try {
    await balanceSyncService.refreshBalance()
  } catch {
    setIsRefreshingBalance(false)
  }
  // isRefreshingBalance is set false inside onSuccess / onError callbacks above
}, [wallet?.address])
```

---

## Header Changes

### New Imports

```ts
import { formatDistanceToNow } from 'date-fns'
```

### Destructuring from `useWallet()`

```ts
const {
  wallet, isConnecting, isRestoringSession, networkMismatch,
  connect, disconnect,
  // new:
  balance, balanceError, isBalanceStale,
  lastBalanceUpdate, refreshBalance, isRefreshingBalance,
} = useWallet()
```

### Relative Timestamp State

A local `useState` + `useEffect` keeps the display ticking every second:

```ts
const [, forceTickUpdate] = useState(0)

useEffect(() => {
  if (!lastBalanceUpdate) return
  const id = setInterval(() => forceTickUpdate((n) => n + 1), 1000)
  return () => clearInterval(id)
}, [lastBalanceUpdate])
```

### Wallet Section JSX

The wallet section (right side of the header, inside the `wallet ? (...)` branch) gains a balance sub-row and a refresh button:

```tsx
{wallet && (
  <div className="flex items-center gap-2">
    {/* Balance display */}
    <div className="hidden sm:block text-right">
      {/* Top line: address */}
      <p className="text-xs text-slate-400">Connected</p>
      <p className="text-xs md:text-sm font-mono text-slate-200">
        {shortAddr(wallet.address)}
      </p>

      {/* Balance line */}
      <div className="flex items-center justify-end gap-1 mt-0.5">
        {balanceError && !balance ? (
          <span className="text-xs font-mono text-slate-500">—</span>
        ) : (
          <span className={`text-xs font-mono ${isBalanceStale ? 'text-amber-400' : 'text-slate-300'}`}>
            {balance ?? '…'} XLM
          </span>
        )}
        {isBalanceStale && (
          <svg viewBox="0 0 16 16" fill="currentColor" className="w-3 h-3 text-amber-400 flex-shrink-0" aria-label="Stale balance">
            <path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm.75 3.75v3.5l2.5 1.5-.75 1.25-3-1.75V4.75h1.25z" />
          </svg>
        )}
      </div>

      {/* Last-updated timestamp */}
      <p className={`text-[10px] leading-tight ${isBalanceStale || balanceError ? 'text-amber-500' : 'text-slate-500'}`}>
        {lastBalanceUpdate
          ? `Updated ${formatDistanceToNow(lastBalanceUpdate, { addSuffix: true })}`
          : 'Updating…'}
      </p>

      {/* Inline error hint */}
      {balanceError && (
        <p className="text-[10px] text-amber-500 leading-tight">Balance sync error</p>
      )}
    </div>

    {/* Refresh button */}
    <button
      onClick={refreshBalance}
      disabled={isRefreshingBalance}
      className="btn-secondary text-xs px-2 py-2 h-9 hidden sm:flex items-center gap-1"
      aria-label="Refresh balance"
      title="Refresh balance"
    >
      <svg
        viewBox="0 0 20 20"
        fill="currentColor"
        className={`w-4 h-4 ${isRefreshingBalance ? 'animate-spin' : ''}`}
      >
        <path
          fillRule="evenodd"
          d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z"
          clipRule="evenodd"
        />
      </svg>
    </button>

    {/* Disconnect button */}
    <button onClick={disconnect} className="btn-secondary text-xs px-2 md:px-3 py-2 h-10 md:h-9">
      Disconnect
    </button>
  </div>
)}
```

---

## Data Models

### `BalanceSyncCallbacks` (in `balanceSyncService.ts`)

| Field | Type | Description |
|---|---|---|
| `onSuccess` | `(balance: string, timestamp: Date) => void` | Called on every successful Horizon fetch |
| `onError` | `(message: string, consecutiveCount: number) => void` | Called on every failed fetch |

### Balance fields in `WalletContextValue`

| Field | Type | Initial value | Reset on disconnect |
|---|---|---|---|
| `balance` | `string \| null` | `null` | `null` |
| `balanceError` | `string \| null` | `null` | `null` |
| `isBalanceStale` | `boolean` | `false` | `false` |
| `lastBalanceUpdate` | `Date \| null` | `null` | `null` |
| `isRefreshingBalance` | `boolean` | `false` | `false` |

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Polling targets the correct Horizon endpoint

*For any* wallet address and Horizon base URL, every HTTP fetch issued by the Balance_Sync_Service should target the URL `${horizonUrl}/accounts/${address}`.

**Validates: Requirements 1.3**

---

### Property 2: Successful fetch updates balance and timestamp atomically

*For any* valid Horizon account JSON response containing a native-asset balance entry, after `_fetchAndNotify` completes successfully, `onSuccess` should be called exactly once with the parsed balance string and a `Date` whose `.getTime()` is within 1 000 ms of `Date.now()`.

**Validates: Requirements 1.4**

---

### Property 3: Failed fetch retains previous balance and sets error

*For any* previously-fetched balance value, after a subsequent fetch failure (network error or non-2xx HTTP response), the balance exposed by the service callbacks must equal the previously fetched value, and `balanceError` must be a non-empty string.

**Validates: Requirements 5.1, 5.2**

---

### Property 4: Consecutive failure counter triggers toast at threshold 3

*For any* sequence of N consecutive fetch failures: if N < 3, `toast.error` must not be called; if N = 3, `toast.error` must be called exactly once; if N > 3, `toast.error` must not be called again (the `id`-based deduplication prevents re-firing).

**Validates: Requirements 5.5**

---

### Property 5: Successful fetch after error clears error state

*For any* state where `balanceError` is non-null and `isBalanceStale` is true, a subsequent successful fetch must result in `balanceError === null`, `isBalanceStale === false`, and `_consecutiveFailures === 0`.

**Validates: Requirements 5.4**

---

### Property 6: Tab hidden suspends polling

*For any* wallet address, after `document.hidden` transitions to `true` and a `visibilitychange` event fires, advancing the fake timer by any multiple of the poll interval should produce zero additional fetch calls beyond those already in flight.

**Validates: Requirements 6.3**

---

### Property 7: Tab visible resumes polling with immediate fetch

*For any* wallet address that has an active sync session, after `document.hidden` transitions to `false` and a `visibilitychange` event fires, exactly one fetch should be triggered immediately (before the next poll interval expires).

**Validates: Requirements 6.4**

---

### Property 8: Debounce coalesces rapid address changes

*For any* sequence of N address changes that all occur within the debounce window (300 ms), exactly one fetch cycle should be started for the last address in the sequence — and zero fetch cycles should be started for any intermediate address.

**Validates: Requirements 6.5, 1.5**

---

### Property 9: WebSocket event triggers exactly one balance fetch

*For any* mocked account-change WebSocket message, receiving it should trigger exactly one call to `_fetchAndNotify` for the currently active wallet address.

**Validates: Requirements 2.2**

---

## Error Handling

### Fetch failure flow

```
_fetchAndNotify()
  └─ fetch() throws or non-2xx
       ├─ if AbortError → silently return (stopSync was called; not a real failure)
       ├─ increment _consecutiveFailures
       ├─ call onError(message, count)
       │    └─ WalletContext: setBalanceError, setIsBalanceStale
       │    └─ if count === 3: toast.error (with id="balance-sync-error")
       └─ retain existing balance state (no setBalance call on failure)
```

### HTTP error response parsing

```ts
if (!response.ok) {
  if (response.status === 404) {
    // Account not yet funded on-chain — treat as 0 balance, not an error
    return '0.0000000'
  }
  throw new Error(`Horizon HTTP ${response.status}`)
}
```

404 is treated as "account not funded" (valid state on testnet) and resolves with `'0.0000000'`, resetting the failure counter. All other non-2xx responses throw and increment the failure counter.

### WebSocket failure handling

If `checkWebSocketCapability` throws or times out (> 3 000 ms), it returns `false` and polling starts. If an established WebSocket emits an unexpected `close` or `error` event, a `console.warn` is logged and the poller is started as a fallback. The WebSocket path is never retried within the same session — once it falls back to polling, it stays on polling.

### Toast deduplication

`react-hot-toast` supports an `id` field; using `id: 'balance-sync-error'` ensures only one toast is visible at a time even if `onError` is called multiple times with `count >= 3`.

---

## Testing Strategy

The test framework present in the project is **Vitest** (see `package.json` `devDependencies`). Property-based testing will use **fast-check** (to be added as a `devDependency`):

```
npm install --save-dev fast-check
```

### Unit tests — `balanceSyncService`

File: `frontend/src/__tests__/balanceSyncService.test.ts`

- Use `vitest` fake timers (`vi.useFakeTimers()`) for all interval/debounce tests.
- Mock `fetch` using `vi.stubGlobal('fetch', vi.fn())`.
- Mock `WebSocket` using a minimal stub class.

Key test scenarios (example-based):
- Connect → fetch fires immediately
- After `stopSync()` → no more fetches when timer advances
- 404 response → balance set to `'0.0000000'`, no error
- 3 consecutive failures → `toast.error` called once
- 4th failure → `toast.error` not called again
- After failure → success → error state cleared

### Property tests — `balanceSyncService`

File: `frontend/src/__tests__/balanceSyncService.property.test.ts`

Each property test runs a minimum of **100 iterations** via fast-check.

```ts
// Feature: wallet-balance-sync, Property 1: polling targets correct Horizon endpoint
it.prop([fc.string({ minLength: 56, maxLength: 56 }), fc.webUrl()])
  ('P1: fetch URL matches horizon + address', async (address, horizonUrl) => { ... })

// Feature: wallet-balance-sync, Property 2: successful fetch updates balance and timestamp
it.prop([fc.record({ balance: fc.string() })])
  ('P2: onSuccess called with parsed balance and recent Date', async (balanceData) => { ... })

// Feature: wallet-balance-sync, Property 3: failed fetch retains previous balance
it.prop([fc.string(), fc.constantFrom(400, 500, 503)])
  ('P3: error retains stale balance', async (prevBalance, httpStatus) => { ... })

// Feature: wallet-balance-sync, Property 4: toast only at consecutive failure count 3
it.prop([fc.integer({ min: 1, max: 10 })])
  ('P4: toast fires iff count reaches 3', async (n) => { ... })

// Feature: wallet-balance-sync, Property 5: success clears error state
it.prop([fc.string(), fc.string()])
  ('P5: recovery from error', async (prevBalance, errorMsg) => { ... })

// Feature: wallet-balance-sync, Property 6: hidden tab suspends polling
it.prop([fc.string({ minLength: 56, maxLength: 56 }), fc.integer({ min: 1, max: 10 })])
  ('P6: no fetches while tab hidden', async (address, intervals) => { ... })

// Feature: wallet-balance-sync, Property 7: visible tab triggers immediate fetch
it.prop([fc.string({ minLength: 56, maxLength: 56 })])
  ('P7: immediate fetch on tab show', async (address) => { ... })

// Feature: wallet-balance-sync, Property 8: debounce coalesces rapid changes
it.prop([fc.array(fc.string({ minLength: 56, maxLength: 56 }), { minLength: 2, maxLength: 5 })])
  ('P8: only last address fetched after rapid changes', async (addresses) => { ... })

// Feature: wallet-balance-sync, Property 9: WS event triggers one fetch
it.prop([fc.record({ type: fc.constant('account'), account: fc.string() })])
  ('P9: WS event → exactly one fetch', async (event) => { ... })
```

### Component tests — `Header`

File: `frontend/src/__tests__/Header.test.tsx`

Use `@testing-library/react`. Mock `useWallet()` to inject controlled balance state.

Key assertions:
- Refresh button rendered when wallet connected; absent when not connected
- Refresh button disabled and shows spinner while `isRefreshingBalance=true`
- "Updating…" placeholder when `balance=null` and `lastBalanceUpdate=null`
- "—" when `balance=null` and `balanceError` is set
- Amber colour class applied to timestamp when `isBalanceStale=true`
- Inline "Balance sync error" hint visible when `balanceError` is set

### Integration guidance

For the optional WebSocket path, full integration tests require a mock SSE/WebSocket server (e.g., `msw` with WebSocket handler). These are recommended but out of scope for the initial implementation. Cover the capability-check fallback path (returns false → polling starts) with unit tests using a mocked `WebSocket` constructor that throws immediately.
