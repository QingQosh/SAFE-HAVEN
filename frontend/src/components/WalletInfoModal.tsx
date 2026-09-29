import { useState } from 'react'

export function WalletInfoModal() {
  const [isOpen, setIsOpen] = useState(false)

  return (
    <>
      {/* Help button */}
      <button
        onClick={() => setIsOpen(true)}
        className="fixed bottom-6 right-6 w-10 h-10 rounded-full bg-stellar-600 hover:bg-stellar-700 flex items-center justify-center shadow-lg transition-colors z-40"
        title="Wallet information and help"
        aria-label="Wallet help"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-6 h-6 text-white">
          <circle cx="12" cy="12" r="10" />
          <path d="M12 16v-4M12 8h.01" />
        </svg>
      </button>

      {/* Modal backdrop */}
      {isOpen && (
        <div
          className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
          onClick={() => setIsOpen(false)}
        >
          {/* Modal */}
          <div
            className="bg-slate-900 rounded-xl border border-slate-700 max-w-md w-full shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header */}
            <div className="border-b border-slate-700 px-6 py-4 flex items-center justify-between">
              <h2 className="text-lg font-semibold text-white">Wallet Information</h2>
              <button
                onClick={() => setIsOpen(false)}
                className="text-slate-400 hover:text-slate-200 transition-colors"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-5 h-5">
                  <path d="M18 6L6 18M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Content */}
            <div className="px-6 py-4 space-y-4 max-h-[calc(100vh-200px)] overflow-y-auto">
              {/* Disconnect section */}
              <div>
                <h3 className="font-semibold text-stellar-400 mb-2 flex items-center gap-2">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-4 h-4">
                    <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4M16 3l3 3m0 0l-3 3m3-3h-8" />
                  </svg>
                  Disconnect
                </h3>
                <p className="text-sm text-slate-300 leading-relaxed">
                  Clicking "Disconnect" removes your session from this app, but <span className="font-semibold text-yellow-400">does not revoke Freighter access</span>. You will be automatically reconnected on your next visit.
                </p>
                <p className="text-sm text-slate-400 mt-2">
                  To fully disconnect: Open Freighter → Settings → Connected sites → Remove this site
                </p>
              </div>

              {/* Auto-reconnect section */}
              <div className="border-t border-slate-700 pt-4">
                <h3 className="font-semibold text-stellar-400 mb-2 flex items-center gap-2">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-4 h-4">
                    <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 0 0 4.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 0 0-15.357-2m15.357 2H15" />
                  </svg>
                  Auto-reconnection
                </h3>
                <p className="text-sm text-slate-300 leading-relaxed">
                  For security, we restore your wallet automatically on page load if you've previously connected. This is a technical limitation of browser wallet extensions — not a bug.
                </p>
              </div>

              {/* Why section */}
              <div className="border-t border-slate-700 pt-4">
                <h3 className="font-semibold text-stellar-400 mb-2 flex items-center gap-2">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-4 h-4">
                    <circle cx="12" cy="12" r="10" />
                    <path d="M12 16v-4M12 8h.01" />
                  </svg>
                  Why this design?
                </h3>
                <p className="text-sm text-slate-300 leading-relaxed">
                  Freighter's API doesn't provide a way for dapps to revoke access. Users must manage connections directly in the Freighter extension. This is intentional — it gives users centralized control over wallet permissions.
                </p>
              </div>

              {/* Security note */}
              <div className="border-t border-slate-700 pt-4 bg-blue-900/20 border-blue-700/30 rounded-lg p-3">
                <p className="text-xs text-blue-300 leading-relaxed">
                  💡 <span className="font-semibold">Tip:</span> If concerned about a dapp, revoke access in Freighter instead of relying on the app's disconnect button.
                </p>
              </div>
            </div>

            {/* Footer */}
            <div className="border-t border-slate-700 px-6 py-3 bg-slate-800/50">
              <button
                onClick={() => setIsOpen(false)}
                className="w-full btn-primary text-sm"
              >
                Got it
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  )
}
