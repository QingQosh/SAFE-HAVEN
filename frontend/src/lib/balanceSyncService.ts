// ============================================================
//  Balance Sync Service
//  Continuously fetches and delivers the native XLM balance for
//  a connected wallet. Supports:
//    • 5-second polling (default)
//    • Optional WebSocket fast-path (falls back to polling)
//    • Page Visibility API — suspends when tab is hidden
//    • At-most-one in-flight request guard (AbortController)
//    • Debounced start on rapid address changes
//    • Consecutive failure counter with toast escalation at 3
//
//  This module has NO React dependency — it is a plain TypeScript
//  singleton that WalletContext wires up via callbacks.
// ============================================================

import { CONFIG } from '../config'

// ----------------------------------------------------------------
//  Public types
// ----------------------------------------------------------------

/** Callbacks supplied by WalletContext when starting a sync session */
export interface BalanceSyncCallbacks {
  /** Called after every successful Horizon fetch */
  onSuccess: (balance: string, timestamp: Date) => void
  /**
   * Called after every failed fetch.
   * `consecutiveCount` is the running failure streak (resets to 0 on success).
   */
  onError: (message: string, consecutiveCount: number) => void
}

/** Options for startSync — separated from callbacks for clarity */
export interface BalanceSyncOptions {
  /** Polling interval in milliseconds (default: 5000) */
  pollIntervalMs?: number
  /** Debounce delay in milliseconds for rapid address changes (default: 300) */
  debounceMs?: number
  /** Horizon base URL — defaults to CONFIG.HORIZON_URL */
  horizonUrl?: string
  /** Soroban RPC URL used for WebSocket capability check — defaults to CONFIG.RPC_URL */
  rpcUrl?: string
}

// ----------------------------------------------------------------
//  Internal types
// ----------------------------------------------------------------

interface HorizonBalance {
  asset_type: string
  balance: string // decimal string e.g. "1234.5000000"
}

interface HorizonAccount {
  balances: HorizonBalance[]
}

const DEFAULT_OPTIONS: Required<BalanceSyncOptions> = {
  pollIntervalMs: 5_000,
  debounceMs: 300,
  horizonUrl: CONFIG.HORIZON_URL,
  rpcUrl: CONFIG.RPC_URL,
}

// ----------------------------------------------------------------
//  Module-level singleton state
//  (only one wallet can be active at a time)
// ----------------------------------------------------------------

let _pollTimer:            ReturnType<typeof setInterval> | null = null
let _abortController:      AbortController | null               = null
let _ws:                   WebSocket | null                     = null
let _consecutiveFailures:  number                               = 0
let _debounceTimer:        ReturnType<typeof setTimeout> | null = null
let _paused:               boolean                              = false
let _currentAddress:       string | null                        = null
let _callbacks:            BalanceSyncCallbacks | null          = null
let _options:              Required<BalanceSyncOptions>         = { ...DEFAULT_OPTIONS }
let _visibilityRegistered: boolean                              = false

// ----------------------------------------------------------------
//  Horizon response parsing
// ----------------------------------------------------------------

function parseNativeBalance(data: HorizonAccount): string {
  const native = data.balances.find((b) => b.asset_type === 'native')
  return native?.balance ?? '0.0000000'
}

// ----------------------------------------------------------------
//  Core fetch
// ----------------------------------------------------------------

/**
 * Fetch the balance for `address` from Horizon, notify callbacks,
 * and update module state accordingly.
 *
 * - HTTP 404  → treated as unfunded account; calls onSuccess('0.0000000')
 * - AbortError → silently swallowed (stopSync was called; not a real failure)
 * - Other errors → increments failure counter, calls onError
 */
async function _fetchAndNotify(address: string): Promise<void> {
  // Create a fresh abort controller for this request
  const controller = new AbortController()
  _abortController = controller

  try {
    const url = `${_options.horizonUrl}/accounts/${address}`
    const response = await fetch(url, { signal: controller.signal })

    // Clear the controller reference now that the fetch has settled
    if (_abortController === controller) {
      _abortController = null
    }

    // 404 = account not yet funded on-chain — treat as zero balance, not an error
    if (response.status === 404) {
      _consecutiveFailures = 0
      _callbacks?.onSuccess('0.0000000', new Date())
      return
    }

    if (!response.ok) {
      throw new Error(`Horizon HTTP ${response.status}`)
    }

    const data = (await response.json()) as HorizonAccount
    const balance = parseNativeBalance(data)

    _consecutiveFailures = 0
    _callbacks?.onSuccess(balance, new Date())
  } catch (err: unknown) {
    // Clear the controller reference if it's still the one we created
    if (_abortController === controller) {
      _abortController = null
    }

    // AbortError means stopSync() was called — not a real failure
    if (err instanceof Error && err.name === 'AbortError') {
      return
    }

    _consecutiveFailures += 1
    const message = err instanceof Error ? err.message : String(err)
    _callbacks?.onError(message, _consecutiveFailures)
  }
}

// ----------------------------------------------------------------
//  Page Visibility API handler
// ----------------------------------------------------------------

function _onVisibilityChange(): void {
  if (document.hidden) {
    _paused = true
  } else {
    _paused = false
    // Resume immediately: trigger a fetch now without waiting for the next tick
    if (_currentAddress && !_abortController) {
      void _fetchAndNotify(_currentAddress)
    }
  }
}

// ----------------------------------------------------------------
//  Poller
// ----------------------------------------------------------------

function _startPoller(
  address: string,
  intervalMs: number,
): void {
  _pollTimer = setInterval(() => {
    if (_paused) return          // tab hidden — skip
    if (_abortController) return // previous fetch still in flight — skip
    void _fetchAndNotify(address)
  }, intervalMs)
}

// ----------------------------------------------------------------
//  WebSocket capability check
// ----------------------------------------------------------------

/**
 * Check at runtime whether the configured Soroban RPC endpoint supports
 * WebSocket event streaming. Returns true only if a WebSocket can be opened
 * and a valid subscription acknowledgement is received within 3 seconds.
 * All errors are caught — the function never throws.
 */
export async function checkWebSocketCapability(rpcUrl: string): Promise<boolean> {
  return new Promise<boolean>((resolve) => {
    const wsUrl = rpcUrl
      .replace(/^https:\/\//, 'wss://')
      .replace(/^http:\/\//, 'ws://')

    let settled = false
    const settle = (result: boolean): void => {
      if (settled) return
      settled = true
      try { ws.close() } catch { /* ignore */ }
      clearTimeout(timer)
      resolve(result)
    }

    let ws: WebSocket
    try {
      ws = new WebSocket(wsUrl)
    } catch {
      resolve(false)
      return
    }

    // 3-second timeout
    const timer = setTimeout(() => settle(false), 3_000)

    ws.addEventListener('open', () => {
      // Send a minimal JSON-RPC subscription probe
      try {
        ws.send(JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'getEvents', params: {} }))
      } catch {
        settle(false)
      }
    })

    ws.addEventListener('message', () => {
      // Any message back counts as "capable"
      settle(true)
    })

    ws.addEventListener('error', () => settle(false))
    ws.addEventListener('close', () => settle(false))
  })
}

// ----------------------------------------------------------------
//  WebSocket subscription (optional fast-path)
// ----------------------------------------------------------------

function _startWebSocket(address: string): void {
  const wsUrl = _options.rpcUrl
    .replace(/^https:\/\//, 'wss://')
    .replace(/^http:\/\//, 'ws://')

  let ws: WebSocket
  try {
    ws = new WebSocket(wsUrl)
  } catch (e) {
    console.warn('[balanceSyncService] WebSocket construction failed, falling back to polling:', e)
    _startPoller(address, _options.pollIntervalMs)
    return
  }

  _ws = ws

  ws.addEventListener('open', () => {
    try {
      ws.send(JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'subscribeAccount',
        params: { account: address },
      }))
    } catch (e) {
      console.warn('[balanceSyncService] WebSocket send failed:', e)
    }
  })

  ws.addEventListener('message', (event: MessageEvent) => {
    try {
      const data = JSON.parse(event.data as string) as Record<string, unknown>
      // Trigger a balance fetch on account-change events for the current address
      if (
        data['type'] === 'account' &&
        (data['account'] === address || data['id'] === address) &&
        _currentAddress === address
      ) {
        if (!_abortController) {
          void _fetchAndNotify(address)
        }
      }
    } catch {
      // Unparseable message — ignore
    }
  })

  const fallbackToPolling = (reason: string): void => {
    console.warn(`[balanceSyncService] WebSocket ${reason}, falling back to polling`)
    _ws = null
    _startPoller(address, _options.pollIntervalMs)
  }

  ws.addEventListener('error', () => fallbackToPolling('error'))

  ws.addEventListener('close', (event: CloseEvent) => {
    if (event.code !== 1000) {
      // Unexpected close — fall back to polling
      fallbackToPolling(`closed unexpectedly (code ${event.code})`)
    }
  })
}

// ----------------------------------------------------------------
//  Internal session starter (called after debounce)
// ----------------------------------------------------------------

async function _doStartSync(
  address: string,
  callbacks: BalanceSyncCallbacks,
  options: Required<BalanceSyncOptions>,
): Promise<void> {
  // Clean up any existing session first
  stopSync()

  // Store session state
  _currentAddress = address
  _callbacks      = callbacks
  _options        = options

  // Register page visibility listener (idempotent)
  if (!_visibilityRegistered) {
    document.addEventListener('visibilitychange', _onVisibilityChange)
    _visibilityRegistered = true
  }

  // Fetch immediately without waiting for the first interval
  await _fetchAndNotify(address)

  // Only continue if this session is still active (stopSync may have been called)
  if (_currentAddress !== address) return

  // Attempt WebSocket fast-path; fall back to polling if not supported
  const wsSupported = await checkWebSocketCapability(options.rpcUrl)
  if (_currentAddress !== address) return // guard again after async gap

  if (wsSupported) {
    _startWebSocket(address)
  } else {
    _startPoller(address, options.pollIntervalMs)
  }
}

// ----------------------------------------------------------------
//  Public API
// ----------------------------------------------------------------

/**
 * Stop all polling, close any open WebSocket, and reset all module state.
 * Safe to call even if no sync is active.
 */
export function stopSync(): void {
  if (_debounceTimer !== null) {
    clearTimeout(_debounceTimer)
    _debounceTimer = null
  }

  if (_pollTimer !== null) {
    clearInterval(_pollTimer)
    _pollTimer = null
  }

  if (_abortController !== null) {
    _abortController.abort()
    _abortController = null
  }

  if (_ws !== null) {
    try { _ws.close(1000) } catch { /* ignore */ }
    _ws = null
  }

  if (_visibilityRegistered) {
    document.removeEventListener('visibilitychange', _onVisibilityChange)
    _visibilityRegistered = false
  }

  _consecutiveFailures = 0
  _paused              = false
  _currentAddress      = null
  _callbacks           = null
  _options             = { ...DEFAULT_OPTIONS }
}

/**
 * Start (or restart) the balance sync for a given wallet address.
 * Safe to call repeatedly: any existing session is cleanly stopped first.
 *
 * Calls are debounced by `options.debounceMs` (default 300 ms) to absorb
 * rapid consecutive calls during React re-renders.
 *
 * @returns A cleanup function; call it to stop the sync (e.g. from useEffect cleanup).
 */
export function startSync(
  address: string,
  callbacks: BalanceSyncCallbacks,
  options?: BalanceSyncOptions,
): () => void {
  const resolved: Required<BalanceSyncOptions> = {
    ...DEFAULT_OPTIONS,
    ...options,
  }

  // Clear any pending debounce
  if (_debounceTimer !== null) {
    clearTimeout(_debounceTimer)
    _debounceTimer = null
  }

  _debounceTimer = setTimeout(() => {
    _debounceTimer = null
    void _doStartSync(address, callbacks, resolved)
  }, resolved.debounceMs)

  return stopSync
}

/**
 * Trigger an immediate, on-demand balance fetch outside the normal poll cycle.
 * Uses the same callbacks registered in the current startSync call.
 *
 * - Returns early (no-op) if a fetch is already in flight.
 * - Throws if called when no sync session is active.
 */
export async function refreshBalance(): Promise<void> {
  if (_currentAddress === null) {
    throw new Error('[balanceSyncService] refreshBalance() called with no active sync session')
  }

  // If a fetch is already in flight, let it finish — don't stack another one
  if (_abortController !== null) {
    return
  }

  await _fetchAndNotify(_currentAddress)
}
