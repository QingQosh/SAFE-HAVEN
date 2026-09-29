import type { ContractInfo } from '../App'
import { formatCountdown, shortAddr, stroopsToXlm } from '../lib/format'

interface ContractExplorerProps {
  contractInfo: ContractInfo
}

export function ContractExplorer({ contractInfo }: ContractExplorerProps) {
  return (
    <div className="max-w-3xl space-y-5">
      <section className="card p-6">
        <div className="flex items-start justify-between gap-4 mb-6">
          <div>
            <h2 className="font-semibold text-lg">Contract constants</h2>
            <p className="text-sm text-slate-400 mt-1">Live limits returned by the deployed contract.</p>
          </div>
          <span className="badge-blue">Read-only</span>
        </div>

        <div className="divide-y divide-slate-800">
          <InfoRow
            label="max_deposit"
            value={contractInfo.loading ? 'Loading…' : `${stroopsToXlm(contractInfo.maxDeposit)} XLM`}
            detail={contractInfo.loading ? undefined : `${contractInfo.maxDeposit.toString()} base units`}
          />
          <InfoRow
            label="max_lock_secs"
            value={contractInfo.loading ? 'Loading…' : `${contractInfo.maxLockSecs.toLocaleString()} seconds`}
            detail={contractInfo.loading ? undefined : formatCountdown(contractInfo.maxLockSecs)}
          />
          <InfoRow
            label="version"
            value={contractInfo.loading ? 'Loading…' : contractInfo.version === null ? 'Not set' : `v${contractInfo.version}`}
            detail="Storage schema version"
          />
        </div>
      </section>

      <section className="card p-6">
        <div className="mb-6">
          <h2 className="font-semibold text-lg">Contract state</h2>
          <p className="text-sm text-slate-400 mt-1">Current administrative configuration.</p>
        </div>

        <div className="divide-y divide-slate-800">
          <AddressRow label="admin" address={contractInfo.admin} />
          <AddressRow label="fee_recipient" address={contractInfo.feeRecipient} />
          <div className="flex items-center justify-between gap-4 py-3 first:pt-0 last:pb-0">
            <span className="font-mono text-sm text-slate-400">paused</span>
            <span className={contractInfo.paused ? 'badge-red' : 'badge-green'}>
              {contractInfo.paused ? 'Paused' : 'Active'}
            </span>
          </div>
        </div>
      </section>
    </div>
  )
}

function InfoRow({ label, value, detail }: { label: string; value: string; detail?: string }) {
  return (
    <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-1 py-3 first:pt-0 last:pb-0">
      <span className="font-mono text-sm text-slate-400">{label}</span>
      <span className="text-sm text-right">
        <span className="font-medium text-slate-100">{value}</span>
        {detail && <span className="block text-xs text-slate-500 mt-0.5">{detail}</span>}
      </span>
    </div>
  )
}

function AddressRow({ label, address }: { label: string; address: string | null }) {
  return (
    <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-1 py-3 first:pt-0 last:pb-0">
      <span className="font-mono text-sm text-slate-400">{label}</span>
      {address ? (
        <span className="font-mono text-sm text-slate-200" title={address}>{shortAddr(address)}</span>
      ) : (
        <span className="text-sm text-slate-500">Not set</span>
      )}
    </div>
  )
}
