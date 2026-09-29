import { useNetwork } from '../context/NetworkContext'
import { getNetworkConfig, getNetworkBadgeColor } from '../lib/networks'

/**
 * Displays the current network name with a color-coded badge
 * Shows warning indicator if network selection differs from environment
 */
export function NetworkDisplay() {
  const { currentNetwork, isMismatched } = useNetwork()
  const network = getNetworkConfig(currentNetwork)
  const badgeColor = getNetworkBadgeColor(currentNetwork)

  return (
    <div className="flex items-center gap-2">
      <span className={`badge ${badgeColor}`}>
        {isMismatched && (
          <span
            className="w-2 h-2 rounded-full bg-yellow-300 animate-pulse"
            title="Network selection differs from environment"
          />
        )}
        {network.displayName}
      </span>
    </div>
  )
}
