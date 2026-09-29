import { useState, useEffect, useCallback } from 'react'
import { useWallet } from '../context/WalletContext'
import { useRampOnramp } from '../hooks/useRampOnramp'
import { CONFIG } from '../config'
import toast from 'react-hot-toast'

interface BuyTokensModalProps {
  isOpen: boolean
  onClose: () => void
}

/**
 * Modal component for fiat on-ramp via Ramp Network
 * Allows users to purchase cryptocurrency using fiat currency
 */
export function BuyTokensModal({ isOpen, onClose }: BuyTokensModalProps) {
  const { wallet } = useWallet()
  const { isSDKLoaded, isSDKError, openRampWidget, closeRampWidget } = useRampOnramp()
  const [isInitializing, setIsInitializing] = useState(false)

  // Open Ramp widget when modal opens
  useEffect(() => {
    if (!isOpen) {
      return
    }

    if (!wallet?.address) {
      toast.error('Please connect your wallet first')
      onClose()
      return
    }

    if (isSDKError) {
      toast.error('Failed to load Ramp SDK. Please try again.')
      onClose()
      return
    }

    if (!isSDKLoaded) {
      setIsInitializing(true)
      // Wait a bit for SDK to load
      const timeout = setTimeout(() => {
        if (!isSDKLoaded) {
          toast.error('Ramp SDK is still loading. Please try again.')
          onClose()
        }
        setIsInitializing(false)
      }, 2000)

      return () => clearTimeout(timeout)
    }

    // SDK is loaded and wallet is connected — open the widget
    setIsInitializing(false)
    openRampWidget(wallet.address)
  }, [isOpen, wallet, isSDKLoaded, isSDKError, openRampWidget, onClose])

  const handleClose = useCallback(() => {
    closeRampWidget()
    onClose()
  }, [closeRampWidget, onClose])

  if (!CONFIG.RAMP_ENABLED) {
    return null
  }

  if (!isOpen) {
    return null
  }

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm"
        onClick={handleClose}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => e.key === 'Escape' && handleClose()}
      />

      {/* Modal */}
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div
          className="relative w-full max-w-2xl max-h-[90vh] bg-slate-900 border border-slate-700 rounded-xl shadow-2xl flex flex-col"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-slate-700">
            <h2 className="text-lg font-semibold text-slate-100">Buy Tokens</h2>
            <button
              onClick={handleClose}
              className="text-slate-400 hover:text-slate-200 transition-colors p-1 -mr-2"
              title="Close"
            >
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth={2}
                className="w-5 h-5"
              >
                <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* Content - Ramp widget will render here */}
          <div
            id="ramp-widget-container"
            className="flex-1 overflow-auto bg-slate-950"
          >
            {isInitializing && (
              <div className="flex items-center justify-center h-96">
                <div className="text-center">
                  <div className="w-10 h-10 border-2 border-stellar-600 border-t-transparent rounded-full animate-spin mx-auto mb-3" />
                  <p className="text-slate-400 text-sm">Loading Ramp Network...</p>
                </div>
              </div>
            )}
          </div>

          {/* Footer */}
          <div className="border-t border-slate-700 p-4 bg-slate-900/50 flex justify-end">
            <button
              onClick={handleClose}
              className="btn-secondary text-sm"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </>
  )
}
