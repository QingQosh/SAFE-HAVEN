import { useState, useCallback, useEffect } from 'react'

interface ContractLogSearchProps {
  onSearchChange: (query: string) => void
  placeholder?: string
}

export function ContractLogSearch({
  onSearchChange,
  placeholder = 'Search by tx hash, initiator, parameters, or error...',
}: ContractLogSearchProps) {
  const [query, setQuery] = useState('')

  // Debounce the search to avoid excessive updates
  useEffect(() => {
    const timer = setTimeout(() => {
      onSearchChange(query)
    }, 300)

    return () => clearTimeout(timer)
  }, [query, onSearchChange])

  const handleClear = useCallback(() => {
    setQuery('')
  }, [])

  return (
    <div className="relative">
      <div className="flex items-center gap-2 px-4 py-2 rounded-lg bg-slate-800 border border-slate-700 focus-within:border-blue-500 focus-within:ring-2 focus-within:ring-blue-500/50">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          className="w-4 h-4 text-slate-400 flex-shrink-0"
        >
          <circle cx="11" cy="11" r="8" />
          <path d="m21 21-4.35-4.35" />
        </svg>

        <input
          type="text"
          value={query}
          onChange={e => setQuery(e.target.value)}
          placeholder={placeholder}
          className="flex-1 bg-transparent border-none outline-none text-slate-200 placeholder-slate-500 text-sm"
        />

        {query && (
          <button
            onClick={handleClear}
            className="text-slate-400 hover:text-slate-300 transition-colors"
            aria-label="Clear search"
          >
            <svg viewBox="0 0 24 24" fill="currentColor" className="w-4 h-4">
              <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12 19 6.41z" />
            </svg>
          </button>
        )}
      </div>
    </div>
  )
}
