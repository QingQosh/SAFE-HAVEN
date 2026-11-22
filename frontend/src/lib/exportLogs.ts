import type { ContractLogEntry } from '../types/logs'

export type ExportFormat = 'csv' | 'json' | 'xml' | 'excel'

const LOG_COLUMNS = [
  { key: 'id', label: 'ID' },
  { key: 'operation', label: 'Operation' },
  { key: 'status', label: 'Status' },
  { key: 'timestamp', label: 'Timestamp' },
  { key: 'txHash', label: 'Tx Hash' },
  { key: 'initiator', label: 'Initiator' },
  { key: 'parameters', label: 'Parameters' },
  { key: 'errorMessage', label: 'Error Message' },
  { key: 'details', label: 'Details' },
] as const

interface ExportFormatter {
  extension: string
  format: (logs: ContractLogEntry[]) => Blob | Promise<Blob>
}

const formatters: Record<ExportFormat, ExportFormatter> = {
  json: {
    extension: 'json',
    format: logs => new Blob([JSON.stringify(logs, null, 2)], { type: 'application/json;charset=utf-8' }),
  },
  csv: {
    extension: 'csv',
    format: logs => {
      const rows = logs.map(log => LOG_COLUMNS.map(({ key }) => getLogValue(log, key)))
      const csv = [
        LOG_COLUMNS.map(({ label }) => escapeCsvField(label)).join(','),
        ...rows.map(row => row.map(escapeCsvField).join(',')),
      ].join('\r\n')
      return new Blob([csv], { type: 'text/csv;charset=utf-8' })
    },
  },
  xml: {
    extension: 'xml',
    format: logs => new Blob([formatXml(logs)], { type: 'application/xml;charset=utf-8' }),
  },
  excel: {
    extension: 'xls',
    format: formatExcel,
  },
}

/** Create a downloadable file for the selected export format. */
export async function createExportBlob(logs: ContractLogEntry[], format: ExportFormat): Promise<Blob> {
  return formatters[format].format(logs)
}

/** Export logs using the selected format. */
export async function exportLogs(logs: ContractLogEntry[], format: ExportFormat): Promise<void> {
  const formatter = formatters[format]
  const blob = await formatter.format(logs)
  downloadFile(blob, `contract-logs-${formatDate(new Date())}.${formatter.extension}`)
}

function getLogValue(log: ContractLogEntry, key: typeof LOG_COLUMNS[number]['key']): string {
  const value = log[key]
  if (value === undefined) return ''
  return typeof value === 'string' ? value : JSON.stringify(value) ?? ''
}

function escapeCsvField(field: string): string {
  return /[",\r\n]/.test(field) ? `"${field.replace(/"/g, '""')}"` : field
}

function escapeXml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;')
}

function formatXml(logs: ContractLogEntry[]): string {
  const namespace = 'urn:safe-haven:contract-logs:v1'
  const entries = logs.map(log => {
    const fields = LOG_COLUMNS.map(({ key }) => {
      const value = getLogValue(log, key)
      return `    <${key}>${escapeXml(value)}</${key}>`
    }).join('\n')
    return `  <log>\n${fields}\n  </log>`
  }).join('\n')

  return `<?xml version="1.0" encoding="UTF-8"?>\n<contractLogs xmlns="${namespace}" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="${namespace} /schemas/contract-logs-v1.xsd">\n${entries}\n</contractLogs>`
}

function formatExcel(logs: ContractLogEntry[]): Blob {
  const columns = LOG_COLUMNS.map((_, index) => `<Column ss:Index="${index + 1}" ss:AutoFitWidth="1" ss:Width="${index > 5 ? 220 : 120}"/>`).join('')
  const header = `<Row ss:StyleID="Header">${LOG_COLUMNS.map(({ label }) => `<Cell><Data ss:Type="String">${escapeXml(label)}</Data></Cell>`).join('')}</Row>`
  const rows = logs.map(log => {
    const cells = LOG_COLUMNS.map(({ key }) => {
      if (key === 'timestamp') {
        const timestamp = new Date(log.timestamp).toISOString().replace(/Z$/, '')
        return `<Cell ss:StyleID="Date"><Data ss:Type="DateTime">${timestamp}</Data></Cell>`
      }
      return `<Cell><Data ss:Type="String">${escapeXml(getLogValue(log, key))}</Data></Cell>`
    }).join('')
    return `<Row>${cells}</Row>`
  }).join('')

  const spreadsheet = `<?xml version="1.0" encoding="UTF-8"?><?mso-application progid="Excel.Sheet"?>\n<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet" xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:x="urn:schemas-microsoft-com:office:excel" xmlns:ss="urn:schemas-microsoft-com:office:spreadsheet" xmlns:html="http://www.w3.org/TR/REC-html40"><Styles><Style ss:ID="Header"><Font ss:Bold="1" ss:Color="#FFFFFF"/><Interior ss:Color="#1E293B" ss:Pattern="Solid"/></Style><Style ss:ID="Date"><NumberFormat ss:Format="yyyy-mm-dd hh:mm:ss"/></Style></Styles><Worksheet ss:Name="Contract Logs"><Table>${columns}${header}${rows}</Table><x:WorksheetOptions><x:FreezePanes/><x:FrozenNoSplit/><x:SplitHorizontal>1</x:SplitHorizontal><x:TopRowBottomPane>1</x:TopRowBottomPane><x:ActivePane>2</x:ActivePane></x:WorksheetOptions></Worksheet></Workbook>`
  return new Blob([spreadsheet], { type: 'application/vnd.ms-excel;charset=utf-8' })
}

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

function formatDate(date: Date): string {
  return date.toISOString().split('T')[0]
}
