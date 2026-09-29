import { useEffect, useState } from 'react'
import toast from 'react-hot-toast'

interface GeoLocationData {
  ip: string
  country: string
  city: string
  isp: string
  isVpn: boolean
  isProxy: boolean
  isPublicWifi: boolean
}

/**
 * Detects if user is on public WiFi by analyzing ISP information
 * Uses free ip-api.com service for geolocation data
 */
async function detectPublicWifi(): Promise<GeoLocationData | null> {
  try {
    // Using ip-api.com free tier (no API key required)
    // Note: Free tier has rate limits (45 requests per minute)
    const response = await fetch('https://ip-api.com/json/?fields=query,country,city,isp,mobile')
    
    if (!response.ok) {
      throw new Error('Failed to fetch location data')
    }

    const data = await response.json()

    // Detect public WiFi by analyzing ISP name for common patterns
    const isp = data.isp?.toLowerCase() || ''
    const isPublicWifi = isPublicWifiISP(isp)

    return {
      ip: data.query || 'Unknown',
      country: data.country || 'Unknown',
      city: data.city || 'Unknown',
      isp: data.isp || 'Unknown',
      isVpn: false, // Basic detection; would need premium API for accurate VPN detection
      isProxy: false, // Same as above
      isPublicWifi,
    }
  } catch (error) {
    console.error('Error detecting public WiFi:', error)
    return null
  }
}

/**
 * Heuristic to detect common public WiFi providers based on ISP name
 */
function isPublicWifiISP(isp: string): boolean {
  const publicWifiPatterns = [
    'starbucks',
    'mcdonalds',
    'airport',
    'airline',
    'hotel',
    'library',
    'cafe',
    'coffee',
    'public wifi',
    'free wifi',
    'guest network',
    'open network',
    'wifi hotspot',
    'mobile hotspot',
    'shared network',
    'community wifi',
  ]

  return publicWifiPatterns.some((pattern) => isp.includes(pattern))
}

interface PublicWifiWarningProps {
  enabled?: boolean
}

/**
 * Component that detects and warns about public WiFi usage
 * Uses IP geolocation to identify public networks
 */
export function PublicWifiWarning({ enabled = true }: PublicWifiWarningProps) {
  const [geoData, setGeoData] = useState<GeoLocationData | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isDismissed, setIsDismissed] = useState(false)

  useEffect(() => {
    if (!enabled || isDismissed) return

    const checkNetwork = async () => {
      setIsLoading(true)
      try {
        const data = await detectPublicWifi()
        if (data) {
          setGeoData(data)

          // Show warning if public WiFi detected
          if (data.isPublicWifi) {
            toast.custom(
              (t) => (
                <div
                  className={`${
                    t.visible ? 'animate-in' : 'animate-out'
                  } max-w-md w-full bg-red-950 border border-red-700 rounded-lg shadow-lg p-4 pointer-events-auto`}
                >
                  <div className="flex gap-3">
                    <div className="text-2xl flex-shrink-0">⚠️</div>
                    <div className="flex-1">
                      <p className="font-semibold text-red-100 mb-1">
                        Public WiFi Detected
                      </p>
                      <p className="text-sm text-red-200 mb-2">
                        You appear to be on a public WiFi network ({data.isp}).
                      </p>
                      <p className="text-xs text-red-300">
                        Avoid signing transactions on public networks. Use a VPN or mobile hotspot for security.
                      </p>
                    </div>
                  </div>
                </div>
              ),
              { duration: 10000 }
            )
          }
        }
      } finally {
        setIsLoading(false)
      }
    }

    // Check on mount (with a delay to avoid blocking initial render)
    const timeout = setTimeout(checkNetwork, 1000)
    return () => clearTimeout(timeout)
  }, [enabled, isDismissed])

  // Show banner if public WiFi detected
  if (geoData?.isPublicWifi && !isDismissed) {
    return (
      <div className="w-full bg-red-900/30 border-b border-red-700/40 px-4 py-3">
        <div className="max-w-6xl mx-auto flex items-center gap-3">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth={1.5}
            className="w-5 h-5 text-red-400 flex-shrink-0"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M8.25 3v1.5M4.5 6.75h.008v.008H4.5V6.75zm15 0h.008v.008h-.008V6.75zM12 3v16.5m0 0l-3.75-3.75M12 19.5l3.75-3.75M4.5 19.5H3.75a1.5 1.5 0 01-1.5-1.5V5.25a1.5 1.5 0 011.5-1.5h.75m15 16.5h.75a1.5 1.5 0 001.5-1.5V5.25a1.5 1.5 0 00-1.5-1.5h-.75"
            />
          </svg>
          <div className="flex-1">
            <p className="text-sm font-semibold text-red-400 mb-1">
              Public WiFi Network Detected
            </p>
            <p className="text-xs text-red-300">
              You're connected to {geoData.isp} (ISP) in {geoData.city}, {geoData.country}. 
              Avoid signing transactions on public networks. Consider using a VPN or cellular hotspot.
            </p>
          </div>
          <button
            onClick={() => setIsDismissed(true)}
            className="text-red-400 hover:text-red-300 transition-colors px-3 py-1 text-xs font-medium flex-shrink-0"
          >
            Dismiss
          </button>
        </div>
      </div>
    )
  }

  return null
}
