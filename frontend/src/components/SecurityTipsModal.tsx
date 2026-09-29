import { useState } from 'react'
import { SECURITY_TIPS, getTipsByCategory, getHighPriorityTips, SecurityTip } from '../lib/security'

interface SecurityTipsModalProps {
  isOpen: boolean
  onClose: () => void
}

type CategoryFilter = 'all' | 'wallet' | 'freighter' | 'network' | 'operational'

/**
 * Modal component displaying comprehensive security tips and best practices
 */
export function SecurityTipsModal({ isOpen, onClose }: SecurityTipsModalProps) {
  const [selectedCategory, setSelectedCategory] = useState<CategoryFilter>('all')
  const [expandedTip, setExpandedTip] = useState<string | null>(null)

  // Filter tips based on selected category
  let filteredTips: SecurityTip[]
  if (selectedCategory === 'all') {
    filteredTips = SECURITY_TIPS
  } else {
    filteredTips = getTipsByCategory(selectedCategory as any)
  }

  // Count high-priority tips
  const highPriorityCount = getHighPriorityTips().length

  if (!isOpen) {
    return null
  }

  const getPriorityIcon = (priority: string) => {
    switch (priority) {
      case 'high':
        return '🔴'
      case 'medium':
        return '🟡'
      case 'low':
        return '🟢'
      default:
        return '•'
    }
  }

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'wallet':
        return '🔐'
      case 'freighter':
        return '👛'
      case 'network':
        return '🌐'
      case 'operational':
        return '⚙️'
      default:
        return '💡'
    }
  }

  const getCategoryLabel = (category: string) => {
    switch (category) {
      case 'wallet':
        return 'Wallet Security'
      case 'freighter':
        return 'Freighter Wallet'
      case 'network':
        return 'Network Security'
      case 'operational':
        return 'Operational Security'
      default:
        return 'Security'
    }
  }

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm"
        onClick={onClose}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => e.key === 'Escape' && onClose()}
      />

      {/* Modal */}
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4 overflow-y-auto">
        <div
          className="relative w-full max-w-3xl bg-slate-900 border border-slate-700 rounded-xl shadow-2xl flex flex-col max-h-[90vh]"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between p-6 border-b border-slate-700 flex-shrink-0">
            <div>
              <h2 className="text-xl font-semibold text-slate-100 flex items-center gap-2">
                🛡️ Security Tips & Best Practices
              </h2>
              <p className="text-xs text-slate-400 mt-1">
                {highPriorityCount} high-priority recommendations
              </p>
            </div>
            <button
              onClick={onClose}
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

          {/* Category Filter */}
          <div className="flex flex-wrap gap-2 p-4 border-b border-slate-700 bg-slate-900/50 flex-shrink-0">
            {[
              { id: 'all', label: 'All Tips' },
              { id: 'wallet', label: '🔐 Wallet' },
              { id: 'freighter', label: '👛 Freighter' },
              { id: 'network', label: '🌐 Network' },
              { id: 'operational', label: '⚙️ Operational' },
            ].map((category) => (
              <button
                key={category.id}
                onClick={() => setSelectedCategory(category.id as CategoryFilter)}
                className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                  selectedCategory === category.id
                    ? 'bg-stellar-600 text-white'
                    : 'bg-slate-800 text-slate-300 hover:bg-slate-700'
                }`}
              >
                {category.label}
              </button>
            ))}
          </div>

          {/* Tips Content */}
          <div className="flex-1 overflow-y-auto">
            <div className="p-4 space-y-3">
              {filteredTips.map((tip) => (
                <div
                  key={tip.id}
                  className="bg-slate-800/50 border border-slate-700 rounded-lg hover:border-slate-600 transition-colors"
                >
                  {/* Tip Header - Always Visible */}
                  <button
                    onClick={() =>
                      setExpandedTip(expandedTip === tip.id ? null : tip.id)
                    }
                    className="w-full text-left p-4 flex items-start justify-between gap-3 hover:bg-slate-800/80 transition-colors"
                  >
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="text-lg flex-shrink-0">
                          {getCategoryIcon(tip.category)}
                        </span>
                        <span className="text-sm font-semibold text-slate-200 flex-shrink-0">
                          {getCategoryLabel(tip.category)}
                        </span>
                        <span className="text-xl flex-shrink-0" title={tip.priority}>
                          {getPriorityIcon(tip.priority)}
                        </span>
                      </div>
                      <h3 className="font-semibold text-slate-100 break-words">
                        {tip.title}
                      </h3>
                    </div>
                    <svg
                      viewBox="0 0 20 20"
                      fill="currentColor"
                      className={`w-5 h-5 flex-shrink-0 text-slate-400 transition-transform ${
                        expandedTip === tip.id ? 'rotate-180' : ''
                      }`}
                    >
                      <path
                        fillRule="evenodd"
                        d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
                        clipRule="evenodd"
                      />
                    </svg>
                  </button>

                  {/* Tip Details - Expandable */}
                  {expandedTip === tip.id && (
                    <div className="px-4 pb-4 pt-0 border-t border-slate-700 bg-slate-900/50">
                      <p className="text-sm text-slate-300 leading-relaxed mb-3">
                        {tip.description}
                      </p>
                      {tip.actionable && (
                        <div className="flex items-center gap-2 text-xs text-stellar-300 bg-stellar-900/30 border border-stellar-800/50 rounded px-2 py-1.5">
                          <span>✓</span>
                          <span>This is an actionable step you can take now</span>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>

          {/* Footer */}
          <div className="border-t border-slate-700 p-4 bg-slate-900/50 flex justify-between items-center flex-shrink-0">
            <p className="text-xs text-slate-400">
              Showing {filteredTips.length} of {SECURITY_TIPS.length} tips
            </p>
            <button
              onClick={onClose}
              className="btn-primary text-sm"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </>
  )
}
