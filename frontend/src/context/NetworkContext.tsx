/**
 * Network Context — manages current network selection and persistence
 * Provides network switching with localStorage persistence and environment validation
 */

import React, { createContext, useCallback, useContext, useEffect, useState } from 'react'
import toast from 'react-hot-toast'
import {
  NetworkId,
  detectNetworkFromEnv,
  getNetworkConfig,
  getAlternateNetwork,
  TESTNET,
  MAINNET,
} from '../lib/networks'

interface NetworkContextValue {
  /** Current selected network */
  currentNetwork: NetworkId
  /** Environment-configured network (from VITE_NETWORK_PASSPHRASE) */
  envNetwork: NetworkId
  /** Whether current selection differs from environment configuration */
  isMismatched: boolean
  /** Switch to a different network */
  switchNetwork: (networkId: NetworkId) => void
  /** Reset to environment-configured network */
  resetToEnvNetwork: () => void
}

const NetworkContext = createContext<NetworkContextValue | null>(null)

const NETWORK_STORAGE_KEY = 'safe-haven_selected_network'

/**
 * Initialize network state from localStorage and environment
 * Returns the initially selected network (localStorage or env, in that order)
 */
function initializeNetworkFromStorage(envNetwork: NetworkId): NetworkId {
  if (typeof window === 'undefined' || typeof localStorage === 'undefined') {
    return envNetwork
  }

  const saved = localStorage.getItem(NETWORK_STORAGE_KEY) as NetworkId | null
  if (saved && (saved === NetworkId.TESTNET || saved === NetworkId.MAINNET)) {
    return saved
  }

  return envNetwork
}

export function NetworkProvider({ children }: { children: React.ReactNode }) {
  const envNetwork = detectNetworkFromEnv()
  const [currentNetwork, setCurrentNetwork] = useState<NetworkId>(
    initializeNetworkFromStorage(envNetwork)
  )

  const isMismatched = currentNetwork !== envNetwork

  const switchNetwork = useCallback(
    (networkId: NetworkId) => {
      if (networkId === currentNetwork) {
        return
      }

      // Show warning when switching away from environment network
      if (currentNetwork === envNetwork && networkId !== envNetwork) {
        toast.custom(
          (t) => (
            <div
              className={`${
                t.visible ? 'animate-in' : 'animate-out'
              } max-w-md w-full bg-amber-950 border border-amber-700 rounded-lg shadow-lg p-4 pointer-events-auto flex flex-col gap-2`}
            >
              <p className="font-semibold text-amber-100">Network Switch Warning</p>
              <p className="text-sm text-amber-200">
                You're switching from{' '}
                <span className="font-mono font-semibold">
                  {getNetworkConfig(envNetwork).displayName}
                </span>{' '}
                (environment network) to{' '}
                <span className="font-mono font-semibold">
                  {getNetworkConfig(networkId).displayName}
                </span>
                . This may cause contract interactions to fail.
              </p>
              <p className="text-xs text-amber-300">
                Ensure your wallet is on the selected network.
              </p>
            </div>
          ),
          { duration: 6000 }
        )
      }

      // Switch network and persist
      setCurrentNetwork(networkId)
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem(NETWORK_STORAGE_KEY, networkId)
      }

      toast.success(
        `Switched to ${getNetworkConfig(networkId).displayName}`,
        { duration: 3000 }
      )
    },
    [currentNetwork, envNetwork]
  )

  const resetToEnvNetwork = useCallback(() => {
    if (currentNetwork !== envNetwork) {
      setCurrentNetwork(envNetwork)
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem(NETWORK_STORAGE_KEY, envNetwork)
      }
      toast.success(`Reset to ${getNetworkConfig(envNetwork).displayName}`)
    }
  }, [currentNetwork, envNetwork])

  return (
    <NetworkContext.Provider
      value={{
        currentNetwork,
        envNetwork,
        isMismatched,
        switchNetwork,
        resetToEnvNetwork,
      }}
    >
      {children}
    </NetworkContext.Provider>
  )
}

/**
 * Hook to access network context
 * Must be used inside NetworkProvider
 */
export function useNetwork(): NetworkContextValue {
  const ctx = useContext(NetworkContext)
  if (!ctx) {
    throw new Error('useNetwork must be used inside NetworkProvider')
  }
  return ctx
}
