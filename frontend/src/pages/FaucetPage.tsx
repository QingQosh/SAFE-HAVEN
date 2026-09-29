import { useEffect, useState, type FormEvent } from 'react'
import toast from 'react-hot-toast'
import { useWallet } from '../context/WalletContext'
import { buildRequestFaucet, getFaucetLastRequest, getFaucetStatus, submitTx } from '../lib/stellar'
import type { FaucetAsset, FaucetStatus, TxStatus } from '../types'

const ASSETS: { id: FaucetAsset; label: string }[] = [
  { id: 'Usdc', label: 'USDC' },
  { id: 'Eth', label: 'ETH' },
  { id: 'Btc', label: 'BTC' },
]
const DECIMALS = 10_000_000n

function formatUnits(value: bigint): string {
  const whole = value / DECIMALS
  const fraction = (value % DECIMALS).toString().padStart(7, '0').replace(/0+$/, '')
  return fraction ? `${whole}.${fraction}` : whole.toString()
}

export function FaucetPage() {
  const { wallet, signTransaction } = useWallet()
  const [asset, setAsset] = useState<FaucetAsset>('Usdc')
  const [amount, setAmount] = useState('100')
  const [status, setStatus] = useState<FaucetStatus | null>(null)
  const [lastRequest, setLastRequest] = useState<number | null>(null)
  const [txStatus, setTxStatus] = useState<TxStatus>('idle')

  async function refresh() {
    const [nextStatus, nextLastRequest] = await Promise.all([
      getFaucetStatus(asset),
      wallet ? getFaucetLastRequest(wallet.address) : Promise.resolve(null),
    ])
    setStatus(nextStatus)
    setLastRequest(nextLastRequest)
  }

  useEffect(() => { void refresh() }, [asset, wallet?.address])

  const cooldownRemaining = lastRequest === null
    ? 0
    : Math.max(0, lastRequest + 3600 - Math.floor(Date.now() / 1000))
  const pending = txStatus === 'signing' || txStatus === 'submitting' || txStatus === 'confirming'

  async function handleRequest(event: FormEvent) {
    event.preventDefault()
    if (!wallet || pending || cooldownRemaining > 0 || !status) return
    const wholeAmount = Number(amount)
    if (!Number.isFinite(wholeAmount) || wholeAmount <= 0 || !Number.isInteger(wholeAmount)) {
      toast.error('Enter a positive whole-token amount.')
      return
    }
    const requested = BigInt(wholeAmount) * DECIMALS
    if (requested > status.maxAmount) {
      toast.error(`Maximum request is ${formatUnits(status.maxAmount)} ${asset.toUpperCase()}.`)
      return
    }

    setTxStatus('signing')
    try {
      const xdr = await buildRequestFaucet(wallet.address, asset, requested)
      if (!xdr) throw new Error('Could not build faucet request')
      const signed = await signTransaction(xdr)
      if (!signed.signed) {
        setTxStatus('idle')
        return
      }
      setTxStatus('submitting')
      const result = await submitTx(signed.xdr)
      if (!result.success) throw new Error(result.error ?? 'Faucet request failed')
      setTxStatus('success')
      toast.success(`Requested ${formatUnits(requested)} ${asset.toUpperCase()}.`)
      await refresh()
    } catch (error) {
      setTxStatus('error')
      toast.error(error instanceof Error ? error.message : 'Faucet request failed')
    } finally {
      setTimeout(() => setTxStatus('idle'), 1500)
    }
  }

  return (
    <div className="grid gap-6 lg:grid-cols-[1fr_1.2fr]">
      <section className="rounded-2xl border border-slate-700/60 bg-slate-900/50 p-6">
        <p className="text-xs font-semibold uppercase tracking-widest text-stellar-400">Testnet faucet</p>
        <h2 className="mt-2 text-xl font-semibold text-white">Get test tokens</h2>
        <p className="mt-2 text-sm text-slate-400">One request per wallet every hour. Claims are enforced by the contract.</p>
        <form onSubmit={handleRequest} className="mt-6 space-y-4">
          <label className="block text-sm text-slate-300">Asset
            <select value={asset} onChange={(event) => setAsset(event.target.value as FaucetAsset)} className="mt-2 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-white">
              {ASSETS.map((item) => <option key={item.id} value={item.id}>{item.label}</option>)}
            </select>
          </label>
          <label className="block text-sm text-slate-300">Amount
            <input value={amount} onChange={(event) => setAmount(event.target.value)} inputMode="numeric" className="mt-2 w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-white" />
          </label>
          <button type="submit" disabled={!wallet || pending || cooldownRemaining > 0 || !status} className="w-full rounded-lg bg-stellar-600 px-4 py-2.5 font-medium text-white disabled:cursor-not-allowed disabled:opacity-50">
            {!wallet ? 'Connect wallet to request' : cooldownRemaining > 0 ? `Available again in ${Math.ceil(cooldownRemaining / 60)} min` : pending ? 'Processing...' : 'Request tokens'}
          </button>
        </form>
      </section>
      <section className="rounded-2xl border border-slate-700/60 bg-slate-900/50 p-6">
        <div className="flex items-center justify-between"><div><p className="text-xs font-semibold uppercase tracking-widest text-slate-500">Faucet status</p><h2 className="mt-2 text-xl font-semibold text-white">{asset.toUpperCase()} pool</h2></div><span className="rounded-full bg-emerald-500/10 px-3 py-1 text-xs text-emerald-300">Testnet</span></div>
        {status?.token ? <dl className="mt-8 grid grid-cols-2 gap-5"><div><dt className="text-sm text-slate-500">Available balance</dt><dd className="mt-1 text-2xl font-semibold text-white">{formatUnits(status.balance)} <span className="text-sm text-slate-400">{asset.toUpperCase()}</span></dd></div><div><dt className="text-sm text-slate-500">Max per request</dt><dd className="mt-1 text-2xl font-semibold text-white">{formatUnits(status.maxAmount)}</dd></div><div><dt className="text-sm text-slate-500">Requests served</dt><dd className="mt-1 text-2xl font-semibold text-white">{status.requestCount.toLocaleString()}</dd></div><div><dt className="text-sm text-slate-500">Distributed</dt><dd className="mt-1 text-2xl font-semibold text-white">{formatUnits(status.distributed)}</dd></div></dl> : <p className="mt-8 rounded-lg border border-amber-400/20 bg-amber-400/5 p-4 text-sm text-amber-200">This asset has not been configured by the faucet administrator.</p>}
      </section>
    </div>
  )
}