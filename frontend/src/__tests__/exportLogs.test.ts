import { describe, expect, it } from 'vitest'
import { createExportBlob } from '../lib/exportLogs'
import type { ContractLogEntry } from '../types/logs'

const log: ContractLogEntry = {
  id: 'log-1',
  operation: 'deposit',
  status: 'error',
  timestamp: '2026-11-20T06:18:54.799Z',
  txHash: 'hash-1',
  initiator: 'wallet-1',
  parameters: { amount: 10, memo: 'a < b' },
  errorMessage: 'Invalid "amount"',
  details: 'Line one\nLine two',
}

describe('contract log exports', () => {
  it('creates formatted JSON with the original nested data', async () => {
    const blob = await createExportBlob([log], 'json')
    const exported = JSON.parse(await blob.text()) as ContractLogEntry[]

    expect(exported).toEqual([log])
    expect(blob.type).toContain('application/json')
  })

  it('exports every field in CSV with correctly quoted values', async () => {
    const blob = await createExportBlob([log], 'csv')
    const csv = await blob.text()

    expect(csv).toContain('Parameters')
    expect(csv).toContain('"{""amount"":10,""memo"":""a < b""}"')
    expect(csv).toContain('"Invalid ""amount"""')
    expect(csv).toContain('"Line one\nLine two"')
  })

  it('exports namespaced XML with a schema reference and escaped values', async () => {
    const blob = await createExportBlob([log], 'xml')
    const xml = await blob.text()

    expect(xml).toContain('xsi:schemaLocation="urn:safe-haven:contract-logs:v1 /schemas/contract-logs-v1.xsd"')
    expect(xml).toContain('<parameters>{&quot;amount&quot;:10,&quot;memo&quot;:&quot;a &lt; b&quot;}</parameters>')
    expect(xml).toContain('<errorMessage>Invalid &quot;amount&quot;</errorMessage>')
    expect(xml).toContain('<details>Line one\nLine two</details>')
  })

  it('creates Excel SpreadsheetML with typed date cells and formatted headers', async () => {
    const blob = await createExportBlob([log], 'excel')
    const workbook = await blob.text()

    expect(blob.type).toContain('application/vnd.ms-excel')
    expect(workbook).toContain('<?mso-application progid="Excel.Sheet"?>')
    expect(workbook).toContain('<Style ss:ID="Header">')
    expect(workbook).toContain('<Data ss:Type="DateTime">2026-11-20T06:18:54.799</Data>')
    expect(workbook).toContain('<Data ss:Type="String">log-1</Data>')
    expect(workbook).toContain('<Data ss:Type="String">{"amount":10,"memo":"a &lt; b"}</Data>')
  })
})
