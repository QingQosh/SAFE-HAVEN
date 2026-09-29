import { useRef, useEffect, useState } from 'react'
import { useNetwork } from '../context/NetworkContext'
import { getNetworkConfig, getNetworkBadgeColor, NETWORK_LIST, NetworkId } from '../lib/networks'

/**
 * Dropdown component for switching between networks
 * Shows all available networks with visual indicators
 */
export function NetworkSwitcher() {
  const { currentNetwork, switchNetwork, isMismatched } = useNetwork()
  const [isOpen, setIsOpen] = useState(false)
  const menuRef = useRef<HTMLDivElement>(null)

  const currentNetworkConfig = getNetworkConfig(currentNetwork)
  const badgeColor = getNetworkBadgeColor(currentNetwork)

  // Close dropdown when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside)
      return () => document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [isOpen])

  const handleNetworkSelect = (networkId: NetworkId) => {
    switchNetwork(networkId)
    setIsOpen(false)
  }

  return (
    <div className="relative" ref={menuRef}>
      {/* Button */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-2 px-3 py-2 rounded-lg border border-slate-700 bg-slate-900/50 hover:bg-slate-800 transition-colors text-sm font-medium text-slate-200"
        title="Switch network"
      >
        <span className={`badge ${badgeColor} flex items-center gap-2`}>
          {isMismatched && (
            <span
              className="w-1.5 h-1.5 rounded-full bg-yellow-300 animate-pulse"
              title="Network mismatch: selection differs from environment"
            />
          )}
          {currentNetworkConfig.displayName}
        </span>
        <svg
          viewBox="0 0 20 20"
          fill="currentColor"
          className={`w-4 h-4 transition-transform ${isOpen ? 'rotate-180' : ''}`}
        >
          <path
            fillRule="evenodd"
            d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
            clipRule="evenodd"
          />
        </svg>
      </button>

      {/* Dropdown Menu */}
      {isOpen && (
        <div className="absolute top-full right-0 mt-2 w-48 bg-slate-900 border border-slate-700 rounded-lg shadow-xl z-40 overflow-hidden">
          <div className="py-1">
            {NETWORK_LIST.map((network) => {
              const isSelected = network.id === currentNetwork
              const isEnvNetwork = network.id !== currentNetwork // Will be shown as env in UI
              const itemBadgeColor = getNetworkBadgeColor(network.id)

              return (
                <button
                  key={network.id}
                  onClick={() => handleNetworkSelect(network.id)}
                  className={`w-full text-left px-4 py-2.5 flex items-center justify-between gap-3 transition-colors ${
                    isSelected
                      ? 'bg-slate-800 border-l-2 border-stellar-500'
                      : 'hover:bg-slate-800/50'
                  }`}
                >
                  <div className="flex items-center gap-2 flex-1">
                    <span className={`badge ${itemBadgeColor} text-xs`}>
                      {network.displayName}
                    </span>
                    {isSelected && (
                      <svg
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        className="w-4 h-4 text-stellar-400 flex-shrink-0"
                      >
                        <path
                          fillRule="evenodd"
                          d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                          clipRule="evenodd"
                        />
                      </svg>
                    )}
                  </div>
                </button>
              )
            })}
          </div>

          {/* Divider */}
          <div className="border-t border-slate-700" />

          {/* Info Section */}
          <div className="px-4 py-3 bg-slate-900/50 text-xs text-slate-400 space-y-1">
            <p className="text-slate-300 font-semibold mb-2">Info</p>
            <p>
              <span className="font-mono text-slate-300">RPC:</span>{' '}
              <span className="break-all">{getNetworkConfig(currentNetwork).rpcUrl}</span>
            </p>
            <p>
              <span className="font-mono text-slate-300">Explorer:</span>{' '}
              <a
                href={getNetworkConfig(currentNetwork).explorerUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="text-stellar-400 hover:text-stellar-300 underline"
              >
                Stellar Expert
              </a>
            </p>
          </div>
        </div>
      )}
    </div>
  )
}
