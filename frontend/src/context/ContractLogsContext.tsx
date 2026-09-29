import { createContext, useContext, useState, useCallback, useEffect } from 'react'
import type { ContractLogEntry, LogFilters } from '../types/logs'

const LOGS_STORAGE_KEY = 'safe-haven-contract-logs'
const MAX_STORED_LOGS = 500 // Limit stored logs to prevent localStorage overflow

interface ContractLogsContextType {
  logs: ContractLogEntry[]
  addLog: (log: Omit<ContractLogEntry, 'id' | 'timestamp'>) => string
  updateLog: (id: string, updates: Partial<ContractLogEntry>) => void
  clearLogs: () => void
  filteredLogs: (filters: LogFilters) => ContractLogEntry[]
}

const ContractLogsContext = createContext<ContractLogsContextType | undefined>(undefined)

function generateLogId(): string {
  return `log-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}

function loadLogsFromStorage(): ContractLogEntry[] {
  try {
    const stored = localStorage.getItem(LOGS_STORAGE_KEY)
    if (stored) {
      return JSON.parse(stored) as ContractLogEntry[]
    }
  } catch {
    console.error('Failed to load logs from localStorage')
  }
  return []
}

function saveLogsToStorage(logs: ContractLogEntry[]): void {
  try {
    // Keep only the most recent logs if we exceed the limit
    const logsToSave = logs.slice(-MAX_STORED_LOGS)
    localStorage.setItem(LOGS_STORAGE_KEY, JSON.stringify(logsToSave))
  } catch {
    console.error('Failed to save logs to localStorage')
  }
}

export function ContractLogsProvider({ children }: { children: React.ReactNode }) {
  const [logs, setLogs] = useState<ContractLogEntry[]>(() => loadLogsFromStorage())

  // Save logs to localStorage whenever they change
  useEffect(() => {
    saveLogsToStorage(logs)
  }, [logs])

  const addLog = useCallback((log: Omit<ContractLogEntry, 'id' | 'timestamp'>) => {
    const id = generateLogId()
    const newLog: ContractLogEntry = {
      ...log,
      id,
      timestamp: new Date().toISOString(),
    }
    setLogs(prevLogs => [newLog, ...prevLogs])
    return id
  }, [])

  const updateLog = useCallback((id: string, updates: Partial<ContractLogEntry>) => {
    setLogs(prevLogs =>
      prevLogs.map(log =>
        log.id === id ? { ...log, ...updates } : log
      )
    )
  }, [])

  const clearLogs = useCallback(() => {
    setLogs([])
  }, [])

  const filteredLogs = useCallback((filters: LogFilters): ContractLogEntry[] => {
    return logs.filter(log => {
      // Filter by operation
      if (filters.operation && log.operation !== filters.operation) {
        return false
      }

      // Filter by status
      if (filters.status && log.status !== filters.status) {
        return false
      }

      // Filter by search term (searches txHash, initiator, errorMessage, and parameters)
      if (filters.search) {
        const searchLower = filters.search.toLowerCase()
        const txHashMatch = log.txHash?.toLowerCase().includes(searchLower)
        const initiatorMatch = log.initiator?.toLowerCase().includes(searchLower)
        const errorMatch = log.errorMessage?.toLowerCase().includes(searchLower)
        const paramsMatch = JSON.stringify(log.parameters)
          .toLowerCase()
          .includes(searchLower)

        if (!txHashMatch && !initiatorMatch && !errorMatch && !paramsMatch) {
          return false
        }
      }

      // Filter by date range
      if (filters.dateFrom || filters.dateTo) {
        const logDate = new Date(log.timestamp)

        if (filters.dateFrom && logDate < filters.dateFrom) {
          return false
        }

        if (filters.dateTo) {
          // Include the entire dateTo day by setting to end of day
          const endOfDay = new Date(filters.dateTo)
          endOfDay.setHours(23, 59, 59, 999)
          if (logDate > endOfDay) {
            return false
          }
        }
      }

      return true
    })
  }, [logs])

  const value: ContractLogsContextType = {
    logs,
    addLog,
    updateLog,
    clearLogs,
    filteredLogs,
  }

  return (
    <ContractLogsContext.Provider value={value}>
      {children}
    </ContractLogsContext.Provider>
  )
}

export function useContractLogs(): ContractLogsContextType {
  const context = useContext(ContractLogsContext)
  if (!context) {
    throw new Error('useContractLogs must be used within a ContractLogsProvider')
  }
  return context
}
