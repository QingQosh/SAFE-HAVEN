// ============================================================
//  Wallet Context — manages multi-wallet connection via stellar-wallets-kit
// ============================================================

import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from 'react'
import toast from 'react-hot-toast'
import { StellarWalletsKit } from '@creit.tech/stellar-wallets-kit'
import {
  FREIGHTER_ID,
  WalletNetwork,
  FreighterModule,
  xBullModule,
  AlbedoModule,
  LobstrModule,
  HanaModule,
} from '@creit.tech/stellar-wallets-kit'
import type { WalletInfo, SigningResult } from '../types'
import { shortAddr } from '../lib/format'
import { CONFIG } from '../config'
import {
  startSync,
  stopSync,
  refreshBalance as balanceSyncRefresh,
} from '../lib/balanceSyncService'

// ----------------------------------------------------------------
//  Context shape
// ----------------------------------------------------------------

interface WalletContextValue {
  // --- wallet connection ---
  wallet: WalletInfo | null
  wallets: WalletInfo[]
  isConnecting: boolean
  isRestoringSession: boolean
  networkMismatch: boolean
  connect: () => Promise<void>
  disconnect: () => void
  signTransaction: (xdr: string) => Promise<SigningResult>

  // --- balance sync (new) ---
  /** Native XLM balance as a decimal string (e.g. "1234.5000000"), or null before first fetch */
  balance: string | null
  /** Human-readable error message when the last fetch failed; null when healthy */
  balanceError: string | null
  /** True when the displayed balance is from a prior successful fetch and the latest attempt failed */
  isBalanceStale: boolean
  /** Wall-clock Date of the most recent successful balance fetch; null before first fetch */
  lastBalanceUpdate: Date | null
  /** Triggers an immediate on-demand fetch */
  refreshBalance: () => Promise<void>
  /** True while a manual refreshBalance() call is in flight */
  isRefreshingBalance: boolean
}

const WalletContext = createContext<WalletContextValue | null>(null)

// ----------------------------------------------------------------
//  Provider
// ----------------------------------------------------------------

export function WalletProvider({ children }: { children: React.ReactNode }) {
  // --- wallet connection state ---
  const [wallet, setWallet]                     = useState<WalletInfo | null>(null)
  const [wallets, setWallets]                   = useState<WalletInfo[]>([])
  const [isConnecting, setConnecting]           = useState(false)
  const [isRestoringSession, setRestoring]      = useState(false)
  const [networkMismatch, setNetworkMismatch]   = useState(false)
  const walletKitRef                            = useRef<StellarWalletsKit | null>(null)

  // --- balance sync state (new) ---
  const [balance, setBalance]                           = useState<string | null>(null)
  const [balanceError, setBalanceError]                 = useState<string | null>(null)
  const [isBalanceStale, setIsBalanceStale]             = useState(false)
  const [lastBalanceUpdate, setLastBalanceUpdate]       = useState<Date | null>(null)
  const [isRefreshingBalance, setIsRefreshingBalance]   = useState(false)

  // Determine network
  const isMainnet = CONFIG.NETWORK_PASSPHRASE === 'Public Global Stellar Network ; September 2015'
  const network   = isMainnet ? WalletNetwork.PUBLIC : WalletNetwork.TESTNET

  // ----------------------------------------------------------------
  //  Initialize wallet kit on mount
  // ----------------------------------------------------------------

  useEffect(() => {
    try {
      walletKitRef.current = new StellarWalletsKit({
        network,
        selectedWalletId: FREIGHTER_ID,
        modules: [
          new FreighterModule(),
          new xBullModule(),
          new AlbedoModule(),
          new LobstrModule(),
          new HanaModule(),
        ],
      })
    } catch (e) {
      console.error('Failed to initialize StellarWalletsKit:', e)
    }
  }, [network])

  // ----------------------------------------------------------------
  //  Restore session on mount — re-validate against the live wallet
  //  address to guard against stale sessions after an account or
  //  network switch.
  // ----------------------------------------------------------------

  useEffect(() => {
    if (!walletKitRef.current) return

    const saved        = localStorage.getItem('tlv_wallet_address')
    const savedWallets = localStorage.getItem('tlv_wallets')

    let restoredWallets: WalletInfo[] = []
    try {
      restoredWallets = savedWallets ? (JSON.parse(savedWallets) as WalletInfo[]) : []
    } catch {
      localStorage.removeItem('tlv_wallets')
    }

    if (saved && !restoredWallets.some((item) => item.address === saved)) {
      restoredWallets = [{ address: saved, displayAddress: shortAddr(saved) }, ...restoredWallets]
    }
    setWallets(restoredWallets)

    if (!saved) return

    setRestoring(true)

    const restore = async () => {
      try {
        const result = await walletKitRef.current!.getAddress()
        if (!result?.address) {
          localStorage.removeItem('tlv_wallet_address')
          return
        }

        const { address } = result
        if (address !== saved) {
          localStorage.removeItem('tlv_wallet_address')
          toast('Wallet account changed — please reconnect.', { icon: '🔄' })
          return
        }

        // Address still valid; restore the session
        setWallet({ address: saved, displayAddress: shortAddr(saved) })
      } catch {
        // Not connected — clear the stale session
        localStorage.removeItem('tlv_wallet_address')
      } finally {
        setRestoring(false)
      }
    }

    void restore()
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []) // run once on mount; walletKitRef is a ref so it's stable

  // ----------------------------------------------------------------
  //  Balance sync lifecycle — keyed on wallet address
  // ----------------------------------------------------------------

  useEffect(() => {
    if (!wallet?.address) {
      stopSync()
      setBalance(null)
      setBalanceError(null)
      setIsBalanceStale(false)
      setLastBalanceUpdate(null)
      setIsRefreshingBalance(false)
      return
    }

    const stop = startSync(wallet.address, {
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
            id: 'balance-sync-error',
          })
        }
      },
    })

    return stop
  }, [wallet?.address])

  // ----------------------------------------------------------------
  //  Actions
  // ----------------------------------------------------------------

  const connect = useCallback(async () => {
    setConnecting(true)
    setNetworkMismatch(false)
    try {
      if (!walletKitRef.current) {
        toast.error('Wallet initialization failed. Please refresh the page.')
        return
      }

      const result = await walletKitRef.current.getAddress()
      if (!result?.address) {
        toast.error('Could not get address from wallet')
        return
      }

      const { address } = result
      const info: WalletInfo = { address, displayAddress: shortAddr(address) }
      setWallet(info)
      localStorage.setItem('tlv_wallet_address', address)
      toast.success(`Connected: ${shortAddr(address)}`)
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Failed to connect wallet'
      if (!msg.toLowerCase().includes('reject') && !msg.toLowerCase().includes('cancel')) {
        toast.error(msg, { duration: 8000 })
      }
    } finally {
      setConnecting(false)
    }
  }, [])

  const disconnect = useCallback(() => {
    setWallet(null)
    setNetworkMismatch(false)
    localStorage.removeItem('tlv_wallet_address')
    toast.success('Wallet disconnected')
  }, [])

  const signTransaction = useCallback(async (txXdr: string): Promise<SigningResult> => {
    if (networkMismatch) {
      const msg = `Network mismatch: Wallet is on ${wallet?.walletNetwork ?? 'unknown'}, but app is on ${CONFIG.NETWORK_PASSPHRASE}`
      toast.error(msg, { duration: 0 })
      return { signed: false, rejected: false, error: msg }
    }

    try {
      if (!walletKitRef.current) {
        toast.error('Wallet not initialized')
        return { signed: false, rejected: false, error: 'Wallet not initialized' }
      }

      const result = await walletKitRef.current.signTransaction(txXdr, {
        networkPassphrase: CONFIG.NETWORK_PASSPHRASE,
      })

      if (!result?.signedTxXdr) {
        toast.error('Failed to sign transaction')
        return { signed: false, rejected: false, error: 'No signed XDR returned' }
      }

      return { signed: true, xdr: result.signedTxXdr }
    } catch (e) {
      const msg           = e instanceof Error ? e.message : 'Signing rejected'
      const isUserReject  = msg.toLowerCase().includes('reject') || msg.toLowerCase().includes('cancel')

      if (isUserReject) {
        return { signed: false, rejected: true }
      }

      toast.error(`Signing error: ${msg}`)
      return { signed: false, rejected: false, error: msg }
    }
  }, [networkMismatch, wallet?.walletNetwork])

  const refreshBalance = useCallback(async () => {
    if (!wallet?.address) return
    setIsRefreshingBalance(true)
    try {
      await balanceSyncRefresh()
    } catch {
      // If refreshBalance throws (no active session), just reset the flag
      setIsRefreshingBalance(false)
    }
    // isRefreshingBalance is reset inside onSuccess / onError callbacks above
  }, [wallet?.address])

  // ----------------------------------------------------------------
  //  Render
  // ----------------------------------------------------------------

  return (
    <WalletContext.Provider
      value={{
        // wallet connection
        wallet,
        wallets,
        isConnecting,
        isRestoringSession,
        networkMismatch,
        connect,
        disconnect,
        signTransaction,
        // balance sync
        balance,
        balanceError,
        isBalanceStale,
        lastBalanceUpdate,
        refreshBalance,
        isRefreshingBalance,
      }}
    >
      {children}
    </WalletContext.Provider>
  )
}

// eslint-disable-next-line react-refresh/only-export-components
export function useWallet(): WalletContextValue {
  const ctx = useContext(WalletContext)
  if (!ctx) throw new Error('useWallet must be used inside WalletProvider')
  return ctx
}
