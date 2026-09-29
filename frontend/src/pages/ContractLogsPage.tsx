import { useState, useMemo } from 'react'
import { useContractLogs } from '../context/ContractLogsContext'
import { ContractLogFilters } from '../components/ContractLogFilters'
import { ContractLogSearch } from '../components/ContractLogSearch'
import { exportLogsAsJson, exportLogsAsCsv } from '../lib/exportLogs'
import { shortAddr } from '../lib/format'
import type { LogFilters, ContractLogEntry } from '../types/logs'

const ITEMS_PER_PAGE = 20

export function ContractLogsPage() {
  const { logs, clearLogs, filteredLogs } = useContractLogs()
  const [filtersOpen, setFiltersOpen] = useState(false)
  const [filters, setFilters] = useState<LogFilters>({})
  const [searchQuery, setSearchQuery] = useState('')
  const [currentPage, setCurrentPage] = useState(1)

  // Apply filters and search
  const filtered = useMemo(() => {
    const withSearch = filteredLogs({ ...filters, search: searchQuery })
    // Sort by most recent first
    return withSearch.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
  }, [filters, searchQuery, filteredLogs])

  // Pagination
  const totalPages = Math.ceil(filtered.length / ITEMS_PER_PAGE)
  const startIdx = (currentPage - 1) * ITEMS_PER_PAGE
  const endIdx = startIdx + ITEMS_PER_PAGE
  const currentLogs = filtered.slice(startIdx, endIdx)

  // Reset to page 1 when filters change
  const handleFiltersChange = (newFilters: LogFilters) => {
    setFilters(newFilters)
    setCurrentPage(1)
  }

  const handleSearchChange = (query: string) => {
    setSearchQuery(query)
    setCurrentPage(1)
  }

  const handleExportJson = () => {
    exportLogsAsJson(filtered)
  }

  const handleExportCsv = () => {
    exportLogsAsCsv(filtered)
  }

  const handleClearLogs = () => {
    if (window.confirm('Are you sure you want to clear all logs? This cannot be undone.')) {
      clearLogs()
      setCurrentPage(1)
    }
  }

  return (
    <div className="space-y-6">
      {/* Header Section */}
      <div className="flex flex-col sm:flex-row sm:items-center gap-4 justify-between">
        <div>
          <h2 className="text-lg font-semibold text-slate-200">
            Contract Logs
            <span className="text-slate-400 text-sm ml-2">({filtered.length} total)</span>
          </h2>
          <p className="text-sm text-slate-400 mt-1">
            Track all contract operations and transactions
          </p>
        </div>

        <div className="flex flex-wrap gap-2">
          <button
            onClick={() => handleExportJson()}
            disabled={filtered.length === 0}
            className="px-3 py-2 rounded-lg bg-slate-800 text-slate-200 hover:bg-slate-700 transition-colors text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" className="w-4 h-4">
              <path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5m9-15v7m-4 0v-7" />
            </svg>
            JSON
          </button>
          <button
            onClick={() => handleExportCsv()}
            disabled={filtered.length === 0}
            className="px-3 py-2 rounded-lg bg-slate-800 text-slate-200 hover:bg-slate-700 transition-colors text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" className="w-4 h-4">
              <path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5m9-15v7m-4 0v-7" />
            </svg>
            CSV
          </button>
          <button
            onClick={() => handleClearLogs()}
            disabled={logs.length === 0}
            className="px-3 py-2 rounded-lg bg-red-900/20 text-red-400 hover:bg-red-900/30 transition-colors text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Clear All
          </button>
        </div>
      </div>

      {/* Search and Filters */}
      <div className="space-y-4">
        <ContractLogSearch onSearchChange={handleSearchChange} />
        <ContractLogFilters
          onFiltersChange={handleFiltersChange}
          isOpen={filtersOpen}
          onToggle={() => setFiltersOpen(!filtersOpen)}
        />
      </div>

      {/* Logs Table */}
      {currentLogs.length === 0 ? (
        <div className="border border-slate-700 rounded-lg p-8 text-center">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            className="w-12 h-12 mx-auto text-slate-500 mb-3"
          >
            <path d="M12 2v20m10-10H2" strokeWidth={2} />
          </svg>
          <p className="text-slate-400">
            {logs.length === 0
              ? 'No logs yet. Contract operations will appear here.'
              : 'No logs match the current filters.'}
          </p>
        </div>
      ) : (
        <>
          <div className="overflow-x-auto border border-slate-700 rounded-lg">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700 bg-slate-800/50">
                  <th className="px-4 py-3 text-left font-semibold text-slate-200">Operation</th>
                  <th className="px-4 py-3 text-left font-semibold text-slate-200">Status</th>
                  <th className="px-4 py-3 text-left font-semibold text-slate-200">Timestamp</th>
                  <th className="px-4 py-3 text-left font-semibold text-slate-200">Tx Hash</th>
                  <th className="px-4 py-3 text-left font-semibold text-slate-200">Initiator</th>
                  <th className="px-4 py-3 text-left font-semibold text-slate-200">Details</th>
                </tr>
              </thead>
              <tbody>
                {currentLogs.map(log => (
                  <LogRow key={log.id} log={log} />
                ))}
              </tbody>
            </table>
          </div>

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="flex items-center justify-between">
              <p className="text-sm text-slate-400">
                Page {currentPage} of {totalPages}
              </p>
              <div className="flex gap-2">
                <button
                  onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
                  disabled={currentPage === 1}
                  className="px-3 py-2 rounded-lg bg-slate-800 text-slate-200 hover:bg-slate-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Previous
                </button>
                <button
                  onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
                  disabled={currentPage === totalPages}
                  className="px-3 py-2 rounded-lg bg-slate-800 text-slate-200 hover:bg-slate-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Next
                </button>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  )
}

function LogRow({ log }: { log: ContractLogEntry }) {
  const [expanded, setExpanded] = useState(false)

  const statusColors = {
    pending: 'bg-yellow-900/30 text-yellow-300',
    success: 'bg-green-900/30 text-green-300',
    error: 'bg-red-900/30 text-red-300',
  }

  return (
    <>
      <tr
        className="border-b border-slate-700/50 hover:bg-slate-800/30 cursor-pointer transition-colors"
        onClick={() => setExpanded(!expanded)}
      >
        <td className="px-4 py-3 text-slate-300 font-medium">
          {log.operation.replace(/_/g, ' ')}
        </td>
        <td className="px-4 py-3">
          <span className={`px-2 py-1 rounded text-xs font-semibold ${statusColors[log.status]}`}>
            {log.status.charAt(0).toUpperCase() + log.status.slice(1)}
          </span>
        </td>
        <td className="px-4 py-3 text-slate-400">
          {new Date(log.timestamp).toLocaleString()}
        </td>
        <td className="px-4 py-3 text-slate-400">
          {log.txHash ? (
            <code className="text-xs bg-slate-900 px-2 py-1 rounded">
              {shortAddr(log.txHash)}
            </code>
          ) : (
            <span className="text-slate-500">—</span>
          )}
        </td>
        <td className="px-4 py-3 text-slate-400">
          {log.initiator ? shortAddr(log.initiator) : '—'}
        </td>
        <td className="px-4 py-3 text-right">
          <button
            onClick={e => {
              e.stopPropagation()
              setExpanded(!expanded)
            }}
            className="text-blue-400 hover:text-blue-300 text-xs font-medium"
          >
            {expanded ? 'Hide' : 'Show'}
          </button>
        </td>
      </tr>

      {expanded && (
        <tr className="bg-slate-800/30">
          <td colSpan={6} className="px-4 py-4">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 text-sm">
              {/* Parameters */}
              {Object.keys(log.parameters).length > 0 && (
                <div>
                  <h4 className="font-semibold text-slate-200 mb-2">Parameters</h4>
                  <pre className="bg-slate-900 p-3 rounded text-xs overflow-auto max-h-40 text-slate-300">
                    {JSON.stringify(log.parameters, null, 2)}
                  </pre>
                </div>
              )}

              {/* Error Message */}
              {log.errorMessage && (
                <div>
                  <h4 className="font-semibold text-slate-200 mb-2">Error</h4>
                  <div className="bg-red-900/20 border border-red-800 p-3 rounded text-red-300 text-xs">
                    {log.errorMessage}
                  </div>
                </div>
              )}

              {/* Details */}
              {log.details && (
                <div className="lg:col-span-2">
                  <h4 className="font-semibold text-slate-200 mb-2">Details</h4>
                  <div className="bg-slate-900 p-3 rounded text-slate-300 text-xs">
                    {log.details}
                  </div>
                </div>
              )}

              {/* Log ID */}
              <div className="lg:col-span-2">
                <p className="text-slate-400">
                  <span className="font-semibold text-slate-300">Log ID:</span>{' '}
                  <code className="text-xs bg-slate-900 px-2 py-1 rounded ml-1">{log.id}</code>
                </p>
              </div>
            </div>
          </td>
        </tr>
      )}
    </>
  )
}
