import { useEffect, useState, useCallback } from 'react'
import { CONFIG } from '../config'

/**
 * Ramp SDK window type declaration
 */
declare global {
  interface Window {
    RampInstances?: {
      [key: string]: RampInstance
    }
  }
}

interface RampInstance {
  show: () => void
  hide: () => void
}

interface RampWidgetConfig {
  hostAppName: string
  hostLogoUrl?: string
  variant: 'embedded' | 'hosted'
  containerNodeSelector?: string
  userAddress?: string
  defaultAsset?: string
  assetFilter?: string
  fiatCurrency?: string
  fiatValue?: number
  webhookStatusUrl?: string
  onSuccess?: (purchase: RampPurchase) => void
  onError?: (error: Error) => void
  onClose?: () => void
}

interface RampPurchase {
  id: string
  status: string
  assetId: string
  assetExchangeRate: number
  fiatCurrencyCode: string
  fiatValue: number
  assetAmount: number
  receiverAddress: string
}

interface UseRampOnrampReturn {
  isSDKLoaded: boolean
  isSDKError: boolean
  openRampWidget: (address: string) => void
  closeRampWidget: () => void
}

/**
 * Hook to manage Ramp Network on-ramp widget
 * Handles SDK loading, initialization, and widget lifecycle
 */
export function useRampOnramp(): UseRampOnrampReturn {
  const [isSDKLoaded, setSDKLoaded] = useState(false)
  const [isSDKError, setSDKError] = useState(false)

  // Load Ramp SDK on mount
  useEffect(() => {
    if (!CONFIG.RAMP_ENABLED) {
      return
    }

    // Check if SDK is already loaded
    if (window.RampInstances) {
      setSDKLoaded(true)
      return
    }

    // Check if script is already in DOM
    if (document.querySelector('script[src*="ramp.network"]')) {
      setSDKLoaded(true)
      return
    }

    // Load Ramp SDK script
    const script = document.createElement('script')
    script.src = 'https://ri-widget-staging.firebaseapp.com/iframe.js'
    script.async = true
    script.onload = () => {
      setSDKLoaded(true)
    }
    script.onerror = () => {
      console.error('Failed to load Ramp SDK')
      setSDKError(true)
    }

    document.head.appendChild(script)

    return () => {
      // Cleanup: remove script on unmount if needed
      // Note: We keep the script loaded for the lifetime of the app
    }
  }, [])

  const openRampWidget = useCallback(
    (address: string) => {
      if (!CONFIG.RAMP_ENABLED || !isSDKLoaded) {
        console.warn('Ramp SDK not available')
        return
      }

      try {
        // Initialize Ramp widget with user's wallet address
        const rampConfig: RampWidgetConfig = {
          hostAppName: 'SAFE-HAVEN',
          hostLogoUrl: undefined,
          variant: 'embedded',
          userAddress: address,
          defaultAsset: `stellar_native`, // XLM
          assetFilter: `stellar_native`, // Only show XLM
          fiatCurrency: 'USD',
          onSuccess: (purchase) => {
            console.log('Ramp purchase successful:', purchase)
          },
          onError: (error) => {
            console.error('Ramp error:', error)
          },
          onClose: () => {
            console.log('Ramp widget closed')
          },
        }

        // Create widget instance
        const rampInstance = new (window as any).Ramp(rampConfig)
        
        // Store instance for later access
        if (!window.RampInstances) {
          window.RampInstances = {}
        }
        window.RampInstances.default = rampInstance

        // Show the widget
        rampInstance.show()
      } catch (error) {
        console.error('Failed to open Ramp widget:', error)
        setSDKError(true)
      }
    },
    [isSDKLoaded]
  )

  const closeRampWidget = useCallback(() => {
    try {
      if (window.RampInstances?.default) {
        window.RampInstances.default.hide()
      }
    } catch (error) {
      console.error('Failed to close Ramp widget:', error)
    }
  }, [])

  return {
    isSDKLoaded,
    isSDKError,
    openRampWidget,
    closeRampWidget,
  }
}
