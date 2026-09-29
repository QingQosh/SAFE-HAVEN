# Requirements Document

## Introduction

This feature adds real-time wallet balance syncing to the SAFE-HAVEN frontend application. Currently, the wallet balance is only fetched once on connection and is not updated when external transactions change the on-chain balance. The Wallet Balance Sync feature introduces an automated polling mechanism in `WalletContext`, an optional WebSocket subscription path when the Soroban RPC supports it, a manual refresh button in the `Header` component, and a visual indicator showing when the balance was last updated. Network errors are surfaced gracefully by showing stale data alongside a warning rather than blanking the display.

Out of scope: syncing token balances held inside the safe-haven contract, transaction history, and aggregating balances across multiple wallets.

---

## Glossary

- **Balance_Sync_Service**: The module responsible for fetching, caching, and distributing wallet balance data to the rest of the frontend.
- **WalletContext**: The React context provider (`frontend/src/context/WalletContext.tsx`) that manages wallet connection state for the application.
- **Poller**: The interval-based mechanism inside Balance_Sync_Service that periodically fetches the native XLM balance from the Stellar RPC.
- **WebSocket_Subscription**: An optional persistent connection to the Stellar RPC that receives push notifications when the wallet account changes on-chain.
- **Header**: The top navigation component (`frontend/src/components/Header.tsx`) that displays wallet information and action buttons.
- **Horizon_Client**: The Horizon REST API client used to fetch account balances via `CONFIG.HORIZON_URL`.
- **RPC_Client**: The Soroban RPC client (`getRpc()`) used for on-chain interactions.
- **Stale_Balance**: A balance value that was fetched successfully in the past but has not been confirmed fresh within the current sync cycle due to a network error.
- **Last_Updated_Timestamp**: The UTC wall-clock time recorded when the balance was last successfully fetched.
- **Refresh_Button**: A UI control in the Header that triggers an immediate, on-demand balance fetch.

---

## Requirements

### Requirement 1: Automatic Balance Polling

**User Story:** As a SAFE-HAVEN user, I want my wallet balance to update automatically while the app is open, so that I can see the current balance without manually refreshing the page.

#### Acceptance Criteria

1. WHEN a wallet is connected, THE Balance_Sync_Service SHALL begin polling the wallet's native XLM balance at an interval of 5 seconds.
2. WHEN a wallet is disconnected, THE Balance_Sync_Service SHALL stop all active polling and cancel any pending fetch operations.
3. WHILE polling is active, THE Balance_Sync_Service SHALL fetch the balance using the Horizon_Client account endpoint.
4. WHEN a poll completes successfully, THE WalletContext SHALL update the exposed balance value and record the Last_Updated_Timestamp.
5. WHEN the wallet address changes (account switch), THE Balance_Sync_Service SHALL reset the polling cycle and fetch the balance for the new address immediately.
6. THE Balance_Sync_Service SHALL NOT initiate a new poll while a previous fetch for the same wallet address is still in flight.

---

### Requirement 2: WebSocket Subscription (RPC-Supported Path)

**User Story:** As a SAFE-HAVEN user, I want balance updates to arrive as close to real-time as possible when the RPC supports push notifications, so that I see balance changes immediately after a transaction lands.

#### Acceptance Criteria

1. WHEN a wallet is connected and the Soroban RPC supports event streaming, THE Balance_Sync_Service SHALL establish a WebSocket_Subscription for the connected wallet address.
2. WHEN a WebSocket_Subscription receives an account-change event, THE Balance_Sync_Service SHALL trigger an immediate balance fetch and update the WalletContext balance value.
3. IF the WebSocket_Subscription cannot be established or is not supported by the RPC, THEN THE Balance_Sync_Service SHALL fall back to the polling mechanism defined in Requirement 1.
4. WHEN a wallet is disconnected, THE Balance_Sync_Service SHALL close any open WebSocket_Subscription before removing wallet state.
5. IF the WebSocket_Subscription drops unexpectedly, THEN THE Balance_Sync_Service SHALL automatically revert to the polling mechanism and log a warning to the browser console.

---

### Requirement 3: Manual Refresh Button

**User Story:** As a SAFE-HAVEN user, I want a manual refresh button in the header, so that I can force an immediate balance update at any time without waiting for the next poll cycle.

#### Acceptance Criteria

1. WHILE a wallet is connected, THE Header SHALL display a Refresh_Button that is visible and accessible.
2. WHEN the Refresh_Button is activated, THE Balance_Sync_Service SHALL immediately fetch the current balance and update the WalletContext balance value.
3. WHILE a manual refresh fetch is in progress, THE Refresh_Button SHALL display a loading indicator and SHALL be disabled to prevent duplicate concurrent requests.
4. WHEN the manual refresh fetch completes (success or error), THE Refresh_Button SHALL return to its default enabled state within 500 milliseconds.
5. WHERE a wallet is not connected, THE Header SHALL NOT render the Refresh_Button.

---

### Requirement 4: Last-Updated Timestamp Display

**User Story:** As a SAFE-HAVEN user, I want to see when the balance was last updated, so that I can judge whether the displayed value is current.

#### Acceptance Criteria

1. WHILE a wallet is connected and a balance has been fetched at least once, THE Header SHALL display the Last_Updated_Timestamp in a human-readable relative format (e.g., "Updated 3s ago").
2. THE Header SHALL update the Last_Updated_Timestamp display at most every second so that the relative label stays accurate.
3. WHEN a new balance fetch completes successfully, THE Header SHALL reset the Last_Updated_Timestamp to the current UTC wall-clock time.
4. IF no balance has been fetched yet since the wallet was connected, THEN THE Header SHALL display a neutral placeholder (e.g., "Updating…") instead of a timestamp.
5. WHERE the balance is stale due to a network error, THE Header SHALL render the Last_Updated_Timestamp with a visual warning style to distinguish it from a fresh value.

---

### Requirement 5: Graceful Network Error Handling

**User Story:** As a SAFE-HAVEN user, I want to see a warning when the balance cannot be refreshed due to a network error, so that I know the displayed value may be out of date rather than assuming the display is current.

#### Acceptance Criteria

1. IF a balance fetch fails due to a network error or HTTP error response, THEN THE Balance_Sync_Service SHALL retain the most recently successfully fetched balance as the Stale_Balance.
2. IF a balance fetch fails, THEN THE Balance_Sync_Service SHALL set an error state that WalletContext exposes to consumers.
3. WHILE the error state is set, THE Header SHALL display a non-blocking inline warning indicating that the balance data may be stale.
4. WHEN a subsequent balance fetch succeeds, THE Balance_Sync_Service SHALL clear the error state and restore normal display.
5. THE Balance_Sync_Service SHALL NOT display a toast notification for every individual poll failure; toast notifications SHALL only be shown after 3 consecutive failures for the same wallet address.
6. IF a balance fetch fails and no previously successful balance exists, THEN THE Header SHALL display a dash or "—" placeholder instead of a zero balance.

---

### Requirement 6: Performance Constraints

**User Story:** As a SAFE-HAVEN user, I want balance syncing to run without causing visible lag or degrading the responsiveness of the UI, so that the app remains smooth while syncing is active.

#### Acceptance Criteria

1. THE Balance_Sync_Service SHALL perform all network I/O asynchronously so that balance fetch operations do not block the React rendering pipeline.
2. WHILE polling is active, THE Balance_Sync_Service SHALL consume no more than one concurrent outstanding HTTP request to the Horizon_Client per connected wallet.
3. WHEN the browser tab becomes hidden (document visibility "hidden"), THE Balance_Sync_Service SHALL suspend polling to avoid unnecessary network traffic.
4. WHEN the browser tab becomes visible again, THE Balance_Sync_Service SHALL resume polling immediately and trigger one fetch without waiting for the next interval.
5. THE Balance_Sync_Service SHALL debounce rapid consecutive wallet-state-change events, waiting at least 300 milliseconds before initiating a new fetch cycle, to avoid redundant requests during re-renders.
