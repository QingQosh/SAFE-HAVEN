import { useEffect, useState } from 'react'
import toast from 'react-hot-toast'

interface BalanceWarningProps {
  walletAddress: string | null | undefined
  threshold: number // USD threshold, e.g., 10 for $10
  horizonUrl: string
}

interface AccountBalance {
  xlmAmount: bigint
  usdValue: number
  xlmPrice: number
}

const STROOPS_PER_XLM = 10_000_000

/**
 * Gets the current XLM price in USD from CoinGecko API
 */
async function getXlmPrice(): Promise<number> {
  try {
    const response = await fetch(
      'https://api.coingecko.com/api/v3/simple/price?ids=stellar&vs_currencies=usd&include_market_cap=false&include_24hr_vol=false&include_24hr_change=false'
    )
    if (!response.ok) throw new Error('Failed to fetch price')
    const data = await response.json()
    return data.stellar?.usd || 0
  } catch (error) {
    console.error('Failed to get XLM price:', error)
    return 0
  }
}

/**
 * Gets the account balance from Horizon
 */
async function getAccountBalance(
  address: string,
  horizonUrl: string
): Promise<bigint | null> {
  try {
    const response = await fetch(`${horizonUrl}/accounts/${address}`)
    if (!response.ok) {
      if (response.status === 404) return 0n // Account doesn't exist yet
      throw new Error(`HTTP ${response.status}`)
    }
    const data = await response.json()
    const nativeBalance = data.balances.find(
      (b: any) => b.asset_type === 'native'
    )
    if (!nativeBalance) return 0n
    
    // Convert balance string to stroops (bigint)
    const xlmAmount = parseFloat(nativeBalance.balance)
    return BigInt(Math.floor(xlmAmount * STROOPS_PER_XLM))
  } catch (error) {
    console.error('Failed to get account balance:', error)
    return null
  }
}

/**
 * Component that warns users if their wallet balance is below a threshold
 * Fetches XLM price and wallet balance to calculate USD value
 */
export function SmallBalanceWarning({
  walletAddress,
  threshold = 10,
  horizonUrl,
}: BalanceWarningProps) {
  const [isLoading, setIsLoading] = useState(false)
  const [isDismissed, setIsDismissed] = useState(false)
  const [balance, setBalance] = useState<AccountBalance | null>(null)

  // Check balance when wallet connects or address changes
  useEffect(() => {
    if (!walletAddress || isDismissed) {
      setBalance(null)
      return
    }

    const checkBalance = async () => {
      setIsLoading(true)
      try {
        // Get XLM price and wallet balance in parallel
        const [price, xlmStroops] = await Promise.all([
          getXlmPrice(),
          getAccountBalance(walletAddress, horizonUrl),
        ])

        if (xlmStroops === null) {
          // Error fetching balance
          return
        }

        const xlmAmount = Number(xlmStroops) / STROOPS_PER_XLM
        const usdValue = xlmAmount * price

        setBalance({
          xlmAmount: xlmStroops,
          usdValue,
          xlmPrice: price,
        })

        // Show warning if below threshold
        if (usdValue > 0 && usdValue < threshold) {
          toast.custom(
            (t) => (
              <div
                className={`${
                  t.visible ? 'animate-in' : 'animate-out'
                } max-w-md w-full bg-amber-950 border border-amber-700 rounded-lg shadow-lg p-4 pointer-events-auto`}
              >
                <div className="flex gap-3">
                  <div className="text-2xl flex-shrink-0">⚠️</div>
                  <div className="flex-1">
                    <p className="font-semibold text-amber-100 mb-1">
                      Low Balance Warning
                    </p>
                    <p className="text-sm text-amber-200 mb-2">
                      Your wallet has ~{xlmAmount.toFixed(2)} XLM
                      (${usdValue.toFixed(2)} USD), which is below the ${threshold} threshold.
                    </p>
                    <p className="text-xs text-amber-300">
                      Consider {usdValue === 0 ? 'funding your wallet' : 'adding more funds'} before depositing.
                    </p>
                  </div>
                </div>
              </div>
            ),
            { duration: 8000 }
          )
        }
      } finally {
        setIsLoading(false)
      }
    }

    // Debounce balance check
    const timeout = setTimeout(checkBalance, 500)
    return () => clearTimeout(timeout)
  }, [walletAddress, threshold, horizonUrl, isDismissed])

  // Show warning banner if balance is below threshold
  if (
    balance &&
    balance.usdValue >= 0 &&
    balance.usdValue < threshold &&
    !isDismissed
  ) {
    return (
      <div className="w-full bg-amber-900/30 border-b border-amber-700/40 px-4 py-3">
        <div className="max-w-6xl mx-auto flex items-center gap-3">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth={1.5}
            className="w-5 h-5 text-amber-400 flex-shrink-0"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z"
            />
          </svg>
          <div className="flex-1">
            <p className="text-sm font-semibold text-amber-400 mb-1">
              Small Wallet Balance
            </p>
            <p className="text-xs text-amber-300">
              You have ~{(Number(balance.xlmAmount) / STROOPS_PER_XLM).toFixed(2)} XLM (~$
              {balance.usdValue.toFixed(2)}). Keep some funds for transaction fees and to avoid
              running out of funds while deposited.
            </p>
          </div>
          <button
            onClick={() => setIsDismissed(true)}
            className="text-amber-400 hover:text-amber-300 transition-colors px-3 py-1 text-xs font-medium flex-shrink-0"
          >
            Dismiss
          </button>
        </div>
      </div>
    )
  }

  return null
}
