import { useState } from 'react'
import type { LogFilters, ContractOperation } from '../types/logs'

interface ContractLogFiltersProps {
  onFiltersChange: (filters: LogFilters) => void
  isOpen: boolean
  onToggle: () => void
}

const OPERATIONS: ContractOperation[] = [
  'deposit',
  'deposit_for',
  'deposit_by_ledger',
  'withdraw',
  'withdraw_to',
  'cancel_deposit',
  'register_staker',
  'claim_staker_rewards',
  'emergency_withdraw',
  'pause',
  'unpause',
  'transfer_admin',
  'accept_admin',
  'renounce_admin',
  'initialize',
]

export function ContractLogFilters({
  onFiltersChange,
  isOpen,
  onToggle,
}: ContractLogFiltersProps) {
  const [selectedOperation, setSelectedOperation] = useState<ContractOperation | ''>('')
  const [selectedStatus, setSelectedStatus] = useState<'pending' | 'success' | 'error' | ''>('')
  const [dateFrom, setDateFrom] = useState<string>('')
  const [dateTo, setDateTo] = useState<string>('')

  const handleApplyFilters = () => {
    const filters: LogFilters = {}

    if (selectedOperation) filters.operation = selectedOperation as ContractOperation
    if (selectedStatus) filters.status = selectedStatus as 'pending' | 'success' | 'error'
    if (dateFrom) filters.dateFrom = new Date(dateFrom)
    if (dateTo) filters.dateTo = new Date(dateTo)

    onFiltersChange(filters)
  }

  const handleClearFilters = () => {
    setSelectedOperation('')
    setSelectedStatus('')
    setDateFrom('')
    setDateTo('')
    onFiltersChange({})
  }

  if (!isOpen) {
    return (
      <button
        onClick={onToggle}
        className="px-4 py-2 rounded-lg bg-slate-800 text-slate-200 hover:bg-slate-700 transition-colors flex items-center gap-2"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" className="w-4 h-4">
          <line x1="4" y1="6" x2="20" y2="6" />
          <line x1="4" y1="12" x2="20" y2="12" />
          <line x1="4" y1="18" x2="20" y2="18" />
        </svg>
        Filters
      </button>
    )
  }

  return (
    <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-4 mb-6">
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Operation Filter */}
        <div>
          <label className="block text-sm font-medium text-slate-300 mb-2">Operation</label>
          <select
            value={selectedOperation}
            onChange={e => setSelectedOperation(e.target.value as ContractOperation | '')}
            className="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-700 text-slate-200 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="">All Operations</option>
            {OPERATIONS.map(op => (
              <option key={op} value={op}>
                {formatOperationName(op)}
              </option>
            ))}
          </select>
        </div>

        {/* Status Filter */}
        <div>
          <label className="block text-sm font-medium text-slate-300 mb-2">Status</label>
          <select
            value={selectedStatus}
            onChange={e => setSelectedStatus(e.target.value as any)}
            className="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-700 text-slate-200 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="">All Statuses</option>
            <option value="pending">Pending</option>
            <option value="success">Success</option>
            <option value="error">Error</option>
          </select>
        </div>

        {/* Date From */}
        <div>
          <label className="block text-sm font-medium text-slate-300 mb-2">From Date</label>
          <input
            type="date"
            value={dateFrom}
            onChange={e => setDateFrom(e.target.value)}
            className="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-700 text-slate-200 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        {/* Date To */}
        <div>
          <label className="block text-sm font-medium text-slate-300 mb-2">To Date</label>
          <input
            type="date"
            value={dateTo}
            onChange={e => setDateTo(e.target.value)}
            className="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-700 text-slate-200 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex gap-2 mt-4 justify-end">
        <button
          onClick={handleClearFilters}
          className="px-4 py-2 rounded-lg bg-slate-700 text-slate-200 hover:bg-slate-600 transition-colors text-sm font-medium"
        >
          Clear
        </button>
        <button
          onClick={() => {
            handleApplyFilters()
            onToggle()
          }}
          className="px-4 py-2 rounded-lg bg-blue-600 text-white hover:bg-blue-700 transition-colors text-sm font-medium"
        >
          Apply Filters
        </button>
      </div>
    </div>
  )
}

function formatOperationName(op: ContractOperation): string {
  return op
    .split('_')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}
