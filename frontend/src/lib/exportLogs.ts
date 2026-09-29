import type { ContractLogEntry } from '../types/logs'

/** Export logs as JSON */
export function exportLogsAsJson(logs: ContractLogEntry[]): void {
  const dataStr = JSON.stringify(logs, null, 2)
  const dataBlob = new Blob([dataStr], { type: 'application/json' })
  downloadFile(dataBlob, `contract-logs-${formatDate(new Date())}.json`)
}

/** Export logs as CSV */
export function exportLogsAsCsv(logs: ContractLogEntry[]): void {
  // CSV headers
  const headers = [
    'ID',
    'Operation',
    'Status',
    'Timestamp',
    'Tx Hash',
    'Initiator',
    'Parameters',
    'Error Message',
    'Details',
  ]

  // Prepare rows
  const rows = logs.map(log => [
    log.id,
    log.operation,
    log.status,
    log.timestamp,
    log.txHash || '',
    log.initiator || '',
    JSON.stringify(log.parameters),
    log.errorMessage || '',
    log.details || '',
  ])

  // Create CSV content
  const csvContent = [
    headers.map(escapeCSVField).join(','),
    ...rows.map(row => row.map(escapeCSVField).join(',')),
  ].join('\n')

  const dataBlob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
  downloadFile(dataBlob, `contract-logs-${formatDate(new Date())}.csv`)
}

/** Helper: escape CSV fields */
function escapeCSVField(field: string): string {
  if (field.includes(',') || field.includes('"') || field.includes('\n')) {
    return `"${field.replace(/"/g, '""')}"` // Double quotes and wrap in quotes
  }
  return field
}

/** Helper: download file */
function downloadFile(blob: Blob, filename: string): void {
  const url = window.URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  window.URL.revokeObjectURL(url)
}

/** Helper: format date for filename */
function formatDate(date: Date): string {
  return date.toISOString().split('T')[0] // YYYY-MM-DD
}
