# Implementation Plan: Wallet Balance Sync

## Overview

Implement a continuous, low-overhead wallet balance sync mechanism for the SAFE-HAVEN frontend. The work proceeds in four stages: (1) build the standalone `balanceSyncService` module, (2) wire it into `WalletContext`, (3) update the `Header` UI, and (4) add unit and property-based tests. Each stage builds directly on the previous one.

## Tasks

- [x] 1. Install fast-check dev dependency
  - Run `npm install --save-dev fast-check` in the `frontend/` directory to add the property-based testing library.
  - Verify it appears in `devDependencies` in `frontend/package.json`.
  - _Requirements: Testing Strategy (design §Testing Strategy)_

- [x] 2. Create `balanceSyncService.ts` — core data types and fetch logic
  - Create `frontend/src/lib/balanceSyncService.ts`.
  - Define and export `BalanceSyncCallbacks`, `BalanceSyncOptions` interfaces as described in the design's Public API section.
  - Define the `HorizonBalance` internal interface and implement `parseNativeBalance(data)` to extract the native XLM balance string from a Horizon account response; return `'0.0000000'` when no native entry is found.
  - Implement `_fetchAndNotify(address)`:
    - Create a new `AbortController`; store it in `_abortController`.
    - `fetch(${horizonUrl}/accounts/${address})` with the `AbortSignal`.
    - On HTTP 404 treat as `'0.0000000'` (unfunded account) — call `onSuccess` and reset `_consecutiveFailures`.
    - On any other non-2xx status throw `Error(\`Horizon HTTP ${response.status}\`)`.
    - On `AbortError` silently return without incrementing the failure counter.
    - On any other error increment `_consecutiveFailures` and call `onError(message, _consecutiveFailures)`.
    - On success reset `_consecutiveFailures = 0`, clear `_abortController = null`, call `onSuccess(balance, new Date())`.
  - Declare all module-level singleton state variables (`_pollTimer`, `_abortController`, `_ws`, `_consecutiveFailures`, `_debounceTimer`, `_paused`, `_currentAddress`, `_callbacks`, `_options`).
  - _Requirements: 1.3, 1.4, 5.1, 5.2, 5.6_

- [x] 3. Implement `stopSync()` and page-visibility handling
  - Implement `stopSync()`: clear `_debounceTimer`, clear `_pollTimer`, abort `_abortController`, close `_ws` with code 1000, remove the `visibilitychange` listener, reset all module state to initial values.
  - Implement the `_onVisibilityChange()` handler: set `_paused = true` when `document.hidden` is true; set `_paused = false` and trigger `_fetchAndNotify(_currentAddress)` immediately when the tab becomes visible again (only if no fetch is already in-flight).
  - Export `stopSync`.
  - _Requirements: 1.2, 6.3, 6.4_

- [x] 4. Implement `checkWebSocketCapability()` and `_startWebSocket()`
  - Implement `checkWebSocketCapability(rpcUrl: string): Promise<boolean>`:
    - Derive the WebSocket URL by replacing `https` → `wss` and `http` → `ws` on `rpcUrl`.
    - Open a `WebSocket` to that URL; resolve `true` on receiving a valid subscription acknowledgement within 3 000 ms; resolve `false` on error, timeout, or any thrown exception (never re-throw).
  - Implement `_startWebSocket(address, callbacks)`:
    - Assign `_ws = new WebSocket(wsUrl)`.
    - On `ws.open`: send the account subscription message.
    - On `ws.message`: parse the event; call `_fetchAndNotify(address)` when the message is an account-change event for the current address.
    - On `ws.error` or unexpected `ws.close` (code ≠ 1000): log `console.warn`, set `_ws = null`, fall back to `_startPoller(address, callbacks, _options.pollIntervalMs)`.
  - Export `checkWebSocketCapability`.
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 5. Implement `_startPoller()`, `startSync()`, and `refreshBalance()`
  - Implement `_startPoller(address, callbacks, intervalMs)`:
    - Set `_pollTimer = setInterval(...)` with the given interval.
    - Inside the interval callback: return immediately if `_paused` is true or `_abortController` is non-null (previous fetch still in flight); otherwise call `_fetchAndNotify(address)`.
  - Implement `startSync(address, callbacks, options?)`:
    - Clear any existing `_debounceTimer`.
    - Schedule `_doStartSync(address, callbacks, options)` after `debounceMs` (default 300 ms).
    - Return a cleanup function that calls `stopSync()`.
  - Implement `_doStartSync(address, callbacks, options)`:
    - Call `stopSync()` to clean up any previous session.
    - Store `address`, `callbacks`, and resolved `options` in module state.
    - Register the `visibilitychange` listener (idempotent).
    - Call `_fetchAndNotify(address)` immediately.
    - Call `checkWebSocketCapability(rpcUrl)`: if `true` call `_startWebSocket`; otherwise call `_startPoller`.
  - Implement `refreshBalance(): Promise<void>`:
    - Throw if `_currentAddress` is null.
    - Return early (no-op) if `_abortController` is non-null (fetch already in flight).
    - Otherwise `await _fetchAndNotify(_currentAddress)`.
  - Export `startSync` and `refreshBalance`.
  - _Requirements: 1.1, 1.5, 1.6, 2.3, 3.2, 6.1, 6.2, 6.5_

- [x] 6. Checkpoint — verify `balanceSyncService` builds cleanly
  - Run `tsc --noEmit` (or the project's `typecheck` script) inside `frontend/` to confirm `balanceSyncService.ts` has no TypeScript errors.
  - Ensure all exports are correctly typed and no implicit `any` types remain.
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Extend `WalletContext` with balance sync state and lifecycle
  - In `frontend/src/context/WalletContext.tsx`, extend `WalletContextValue` with the six new fields: `balance`, `balanceError`, `isBalanceStale`, `lastBalanceUpdate`, `refreshBalance`, `isRefreshingBalance`.
  - Add the five new `useState` declarations inside `WalletProvider` with their initial values as defined in the design's Data Models table.
  - Import `startSync`, `stopSync`, and `refreshBalance as balanceSyncRefresh` from `../lib/balanceSyncService`.
  - Add a `useEffect` that depends on `wallet?.address`:
    - When address is `undefined`/`null`: call `stopSync()` and reset all five balance state fields to their initial values.
    - When address is present: call `startSync(wallet.address, { onSuccess, onError })` and return the resulting stop function as the effect cleanup.
    - In `onSuccess(bal, ts)`: call `setBalance`, `setLastBalanceUpdate`, `setBalanceError(null)`, `setIsBalanceStale(false)`, `setIsRefreshingBalance(false)`.
    - In `onError(msg, count)`: call `setBalanceError`, `setIsBalanceStale(true)`, `setIsRefreshingBalance(false)`; when `count === 3` call `toast.error` with `id: 'balance-sync-error'`.
  - Implement the `refreshBalance` callback with `useCallback` as described in the design, guarding against no active session.
  - Include all six new fields in the `WalletContext.Provider` value object.
  - _Requirements: 1.1, 1.2, 1.4, 1.5, 3.2, 5.1, 5.2, 5.4, 5.5_

- [x] 8. Update `Header.tsx` with balance display, refresh button, and stale indicators
  - In `frontend/src/components/Header.tsx`, destructure the six new context fields from `useWallet()`.
  - Import `formatDistanceToNow` from `date-fns`.
  - Add a local `useState` tick counter and a `useEffect` that runs a 1-second `setInterval` whenever `lastBalanceUpdate` is non-null (clear on cleanup), to keep the relative timestamp label live.
  - Inside the `wallet ? (...)` branch of the header's wallet section, add:
    - A balance line: show `—` when `balance` is null and `balanceError` is set; otherwise show `{balance ?? '…'} XLM` with `text-amber-400` styling when `isBalanceStale` is true and `text-slate-300` otherwise.
    - An amber clock icon (`aria-label="Stale balance"`) beside the balance line when `isBalanceStale` is true.
    - A last-updated timestamp line: show `"Updating…"` when `lastBalanceUpdate` is null; otherwise show `"Updated X ago"` via `formatDistanceToNow`; apply `text-amber-500` when `isBalanceStale || balanceError`, otherwise `text-slate-500`.
    - An inline `"Balance sync error"` hint in `text-amber-500` when `balanceError` is non-null.
    - A refresh button with `aria-label="Refresh balance"`, `disabled` when `isRefreshingBalance` is true, wired to `onClick={refreshBalance}`; the icon spins (`animate-spin`) while `isRefreshingBalance` is true.
  - Do NOT render the refresh button when `wallet` is null.
  - _Requirements: 3.1, 3.3, 3.4, 3.5, 4.1, 4.2, 4.3, 4.4, 4.5, 5.3, 5.6_

- [x] 9. Checkpoint — verify full app builds and types check
  - Run `tsc --noEmit` inside `frontend/` to confirm zero type errors across all changed files.
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. Write unit tests for `balanceSyncService`
  - Create `frontend/src/__tests__/balanceSyncService.test.ts`.
  - Configure `vi.useFakeTimers()` in `beforeEach` and restore in `afterEach`.
  - Mock `fetch` with `vi.stubGlobal('fetch', vi.fn())` and mock `WebSocket` with a minimal stub class.
  - [x] 10.1 Implement test: connect → immediate fetch fires
    - After `startSync(address, callbacks)`, advance fake timers past the debounce delay and assert `fetch` was called once with `${horizonUrl}/accounts/${address}`.
    - _Requirements: 1.1, 1.3_
  - [ ]* 10.2 Write unit test: stopSync halts polling
    - Call `startSync`, advance past debounce, call `stopSync()`, advance timers by several poll intervals, assert `fetch` call count does not increase.
    - _Requirements: 1.2_
  - [ ]* 10.3 Write unit test: 404 response sets balance to `'0.0000000'` with no error
    - Mock `fetch` to return `{ ok: false, status: 404 }`.
    - Assert `onSuccess` called with `'0.0000000'` and `onError` not called.
    - _Requirements: Error handling (design §Error Handling)_
  - [ ]* 10.4 Write unit test: 3 consecutive failures trigger toast once, 4th does not
    - Mock `fetch` to reject four times.
    - Assert `toast.error` called exactly once (at count 3) using a spy.
    - _Requirements: 5.5_
  - [ ]* 10.5 Write unit test: success after failure clears error state
    - Drive two failures then one success via mocked `fetch`.
    - Assert `onSuccess` called with a non-null balance and `onError` not called on the third call.
    - _Requirements: 5.4_

- [x] 11. Write property-based tests for `balanceSyncService`
  - Create `frontend/src/__tests__/balanceSyncService.property.test.ts`.
  - Import `fc` from `fast-check` and use `it.prop` (or `fc.assert` + `fc.property`) with Vitest.
  - Each property runs at minimum 100 iterations.
  - [ ]* 11.1 Write property test for Property 1: polling targets correct Horizon endpoint
    - **Property 1: Polling targets the correct Horizon endpoint**
    - **Validates: Requirements 1.3**
    - For any wallet address and Horizon base URL, every HTTP fetch issued by the service should target `${horizonUrl}/accounts/${address}`.
  - [ ]* 11.2 Write property test for Property 2: successful fetch updates balance and timestamp atomically
    - **Property 2: Successful fetch updates balance and timestamp atomically**
    - **Validates: Requirements 1.4**
    - For any valid Horizon account JSON with a native balance entry, `onSuccess` is called exactly once with the parsed balance string and a `Date` within 1 000 ms of `Date.now()`.
  - [ ]* 11.3 Write property test for Property 3: failed fetch retains previous balance and sets error
    - **Property 3: Failed fetch retains previous balance and sets error**
    - **Validates: Requirements 5.1, 5.2**
    - After a prior successful fetch, a subsequent non-2xx (non-404) response must not trigger `onSuccess` and must trigger `onError` with a non-empty message.
  - [ ]* 11.4 Write property test for Property 4: consecutive failure counter triggers toast at threshold 3
    - **Property 4: Consecutive failure counter triggers toast at threshold 3**
    - **Validates: Requirements 5.5**
    - For N consecutive failures, `toast.error` is called exactly once when N ≥ 3, and zero times when N < 3.
  - [ ]* 11.5 Write property test for Property 5: successful fetch after error clears error state
    - **Property 5: Successful fetch after error clears error state**
    - **Validates: Requirements 5.4**
    - After any error sequence, a successful fetch must result in `onSuccess` being called and `_consecutiveFailures` resetting to 0.
  - [ ]* 11.6 Write property test for Property 6: tab hidden suspends polling
    - **Property 6: Tab hidden suspends polling**
    - **Validates: Requirements 6.3**
    - After `document.hidden` becomes `true`, advancing the fake timer by any multiple of the poll interval produces zero additional fetch calls.
  - [ ]* 11.7 Write property test for Property 7: tab visible resumes polling with immediate fetch
    - **Property 7: Tab visible resumes polling with immediate fetch**
    - **Validates: Requirements 6.4**
    - After `document.hidden` transitions to `false`, exactly one fetch is triggered before the next poll interval expires.
  - [ ]* 11.8 Write property test for Property 8: debounce coalesces rapid address changes
    - **Property 8: Debounce coalesces rapid address changes**
    - **Validates: Requirements 6.5, 1.5**
    - For N address changes (N ≥ 2) all occurring within the debounce window, exactly one fetch cycle is started for the last address, and zero for all intermediate addresses.
  - [ ]* 11.9 Write property test for Property 9: WebSocket event triggers exactly one balance fetch
    - **Property 9: WebSocket event triggers exactly one balance fetch**
    - **Validates: Requirements 2.2**
    - For any mocked account-change WebSocket message containing the active wallet address, exactly one call to `_fetchAndNotify` is triggered.

- [x] 12. Write component tests for `Header`
  - Create `frontend/src/__tests__/Header.test.tsx`.
  - Use `@testing-library/react` with `jsdom` or `happy-dom` (already present in `devDependencies`).
  - Mock `useWallet()` by wrapping the component in a provider stub or using `vi.mock('../context/WalletContext')`.
  - [x] 12.1 Implement test: refresh button rendered when wallet connected, absent when not
    - Render `<Header isPaused={false} />` with `wallet` set to a stub; assert the `aria-label="Refresh balance"` button exists.
    - Render again with `wallet = null`; assert the button is absent.
    - _Requirements: 3.1, 3.5_
  - [ ]* 12.2 Write unit test: refresh button disabled and spinning while `isRefreshingBalance=true`
    - Inject `isRefreshingBalance: true`; assert the button has `disabled` attribute and the icon has the `animate-spin` class.
    - _Requirements: 3.3_
  - [ ]* 12.3 Write unit test: "Updating…" placeholder when no balance yet
    - Inject `balance: null`, `lastBalanceUpdate: null`, `balanceError: null`; assert the text "Updating…" is present.
    - _Requirements: 4.4_
  - [ ]* 12.4 Write unit test: "—" placeholder when balance is null and error is set
    - Inject `balance: null`, `balanceError: 'fetch failed'`; assert the "—" placeholder is rendered.
    - _Requirements: 5.6_
  - [ ]* 12.5 Write unit test: amber styling applied when `isBalanceStale=true`
    - Inject `isBalanceStale: true`, `lastBalanceUpdate: new Date()`; assert the timestamp element has the `text-amber-500` class.
    - _Requirements: 4.5_
  - [ ]* 12.6 Write unit test: inline error hint visible when `balanceError` is set
    - Inject `balanceError: 'Network error'`; assert "Balance sync error" text is visible.
    - _Requirements: 5.3_

- [x] 13. Final checkpoint — run full test suite
  - Run `vitest --run` inside `frontend/` and confirm all tests pass.
  - Run `tsc --noEmit` to confirm zero type errors.
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP delivery.
- The property-based tests in task 11 correspond directly to the nine correctness properties defined in the design document; each sub-task explicitly references its property number.
- `fast-check` must be installed (task 1) before any property tests can be written or run.
- `balanceSyncService` is a pure TypeScript module with no React dependency — test it in isolation with Vitest and fake timers before touching the React layer.
- The `Header.tsx` currently has some inconsistencies in its JSX (malformed nested ternary); resolve those while adding the new balance UI elements in task 8.
