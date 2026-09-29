import { useState } from 'react'
import { searchGlossary, getTermsByCategory } from '../lib/glossary'
import { Tooltip } from './Tooltip'

interface HelpModalProps {
  isOpen: boolean
  onClose: () => void
}

type HelpTab = 'faq' | 'glossary'

interface FAQItem {
  question: string
  answer: string | React.ReactNode
  category: 'general' | 'deposits' | 'withdrawals' | 'fees' | 'security'
}

const FAQ_ITEMS: FAQItem[] = [
  {
    question: 'What is SAFE-HAVEN?',
    answer:
      'SAFE-HAVEN is a decentralized vault on the Stellar blockchain where you can lock tokens until a future date. Your funds are managed by an immutable smart contract, not by any central authority.',
    category: 'general',
  },
  {
    question: 'How long can I lock tokens?',
    answer:
      'You can lock tokens for 60 seconds to 5 years. The exact duration is determined by the Unlock Time you choose.',
    category: 'general',
  },
  {
    question: 'What tokens can I deposit?',
    answer:
      'You can deposit any SEP-41 compliant token on the Stellar blockchain, including XLM, USDC, and others.',
    category: 'deposits',
  },
  {
    question: 'What are basis points?',
    answer:
      '1 basis point = 0.01%. So 500 basis points = 5%. This is used for the early-exit penalty percentage.',
    category: 'fees',
  },
  {
    question: 'What happens if I exit early?',
    answer:
      'If you withdraw before the unlock time, your tokens are returned minus the penalty percentage you specified. The penalty goes to the fee recipient address.',
    category: 'withdrawals',
  },
  {
    question: 'Is my vault automatically unlocked?',
    answer:
      'No, your vault automatically becomes eligible for withdrawal when the unlock time is reached. You must manually initiate the withdrawal transaction.',
    category: 'withdrawals',
  },
  {
    question: 'Can the admin steal my funds?',
    answer:
      'No. Funds are managed by an immutable smart contract. The admin can only perform emergency withdrawals (which return funds to you, not to the admin). The admin can renounce their privileges to make SAFE-HAVEN fully trustless.',
    category: 'security',
  },
  {
    question: 'What if I forgot my unlock time?',
    answer:
      'Your unlock time is stored on the blockchain and displayed on your dashboard. You can check any time by viewing your deposits.',
    category: 'general',
  },
  {
    question: 'Can I change the unlock time?',
    answer:
      'No, unlock times are immutable. If you need to access funds sooner, you can cancel the deposit early (paying the penalty).',
    category: 'deposits',
  },
  {
    question: 'What are transaction fees?',
    answer:
      'Standard Stellar network fees apply (~0.00001 XLM). SAFE-HAVEN does not charge fees for deposits or withdrawals, only the early-exit penalty you specify.',
    category: 'fees',
  },
]

/**
 * Help modal with FAQ and glossary
 */
export function HelpModal({ isOpen, onClose }: HelpModalProps) {
  const [activeTab, setActiveTab] = useState<HelpTab>('faq')
  const [selectedFaqCategory, setSelectedFaqCategory] = useState<string>('general')
  const [glossarySearch, setGlossarySearch] = useState('')
  const [expandedFaqIndex, setExpandedFaqIndex] = useState<number | null>(null)

  if (!isOpen) return null

  // Filter FAQ items by selected category
  const filteredFaq = FAQ_ITEMS.filter(
    (item) => item.category === selectedFaqCategory
  )

  // Filter glossary by search
  const glossaryResults =
    glossarySearch.trim() === ''
      ? getTermsByCategory('contract')
      : searchGlossary(glossarySearch)

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4 overflow-y-auto">
        <div className="relative w-full max-w-3xl bg-slate-900 border border-slate-700 rounded-xl shadow-2xl flex flex-col max-h-[90vh]">
          {/* Header */}
          <div className="flex items-center justify-between p-6 border-b border-slate-700 flex-shrink-0">
            <h2 className="text-xl font-semibold text-slate-100 flex items-center gap-2">
              ❓ Help & FAQ
            </h2>
            <button
              onClick={onClose}
              className="text-slate-400 hover:text-slate-200 transition-colors p-1 -mr-2"
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

          {/* Tabs */}
          <div className="flex border-b border-slate-700 bg-slate-900/50 flex-shrink-0">
            {[
              { id: 'faq', label: '❓ FAQ' },
              { id: 'glossary', label: '📖 Glossary' },
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as HelpTab)}
                className={`px-6 py-3 text-sm font-medium transition-colors border-b-2 ${
                  activeTab === tab.id
                    ? 'text-stellar-400 border-stellar-600'
                    : 'text-slate-400 border-transparent hover:text-slate-300'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto">
            {activeTab === 'faq' && (
              <div className="p-6">
                {/* Category Filter */}
                <div className="flex flex-wrap gap-2 mb-4">
                  {['general', 'deposits', 'withdrawals', 'fees', 'security'].map(
                    (cat) => (
                      <button
                        key={cat}
                        onClick={() => setSelectedFaqCategory(cat)}
                        className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                          selectedFaqCategory === cat
                            ? 'bg-stellar-600 text-white'
                            : 'bg-slate-800 text-slate-300 hover:bg-slate-700'
                        }`}
                      >
                        {cat.charAt(0).toUpperCase() + cat.slice(1)}
                      </button>
                    )
                  )}
                </div>

                {/* FAQ Items */}
                <div className="space-y-2">
                  {filteredFaq.map((item, idx) => (
                    <div key={idx} className="border border-slate-700 rounded-lg">
                      <button
                        onClick={() =>
                          setExpandedFaqIndex(
                            expandedFaqIndex === idx ? null : idx
                          )
                        }
                        className="w-full text-left p-4 hover:bg-slate-800/50 transition-colors flex items-center justify-between"
                      >
                        <p className="font-semibold text-slate-100">{item.question}</p>
                        <svg
                          viewBox="0 0 20 20"
                          fill="currentColor"
                          className={`w-5 h-5 text-slate-400 transition-transform flex-shrink-0 ${
                            expandedFaqIndex === idx ? 'rotate-180' : ''
                          }`}
                        >
                          <path
                            fillRule="evenodd"
                            d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
                            clipRule="evenodd"
                          />
                        </svg>
                      </button>
                      {expandedFaqIndex === idx && (
                        <div className="px-4 pb-4 text-sm text-slate-300 border-t border-slate-700">
                          {item.answer}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {activeTab === 'glossary' && (
              <div className="p-6">
                {/* Search */}
                <input
                  type="text"
                  placeholder="Search glossary..."
                  value={glossarySearch}
                  onChange={(e) => setGlossarySearch(e.target.value)}
                  className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-slate-100 placeholder-slate-500 mb-4 focus:outline-none focus:border-stellar-600"
                />

                {/* Terms */}
                <div className="space-y-3">
                  {glossaryResults.map((term) => (
                    <div key={term.term} className="bg-slate-800/50 rounded-lg p-3 border border-slate-700">
                      <div className="flex items-start justify-between gap-2 mb-1">
                        <h3 className="font-semibold text-slate-100">{term.term}</h3>
                        <span className="text-xs px-2 py-1 bg-slate-700 text-slate-300 rounded">
                          {term.category}
                        </span>
                      </div>
                      <p className="text-sm text-slate-300 mb-2">{term.definition}</p>
                      {term.example && (
                        <p className="text-xs text-slate-400 font-mono bg-slate-900 px-2 py-1 rounded">
                          📌 {term.example}
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Footer */}
          <div className="border-t border-slate-700 p-4 bg-slate-900/50 flex justify-end flex-shrink-0">
            <button onClick={onClose} className="btn-primary text-sm">
              Close
            </button>
          </div>
        </div>
      </div>
    </>
  )
}
