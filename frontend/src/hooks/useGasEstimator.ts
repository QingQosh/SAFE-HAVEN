import { useCallback } from 'react'
import { Contract, TransactionBuilder, BASE_FEE, nativeToScVal } from '@stellar/stellar-sdk'
import { getRpc } from '../lib/stellar'
import { CONFIG } from '../config'

export interface GasCostBreakdown {
  baseFee: number
  executionCost: number
  storageCost: number
  totalCost: number
  totalCostInUsd: number
}

export interface GasEstimate {
  success: boolean
  breakdown?: GasCostBreakdown
  error?: string
  /** Stroops per unit (from network) */
  stroopsPerUnit?: number
}

/**
 * Simulate a transaction and extract gas cost information.
 * Returns null if simulation fails.
 */
export function useGasEstimator() {
  const estimateGas = useCallback(async (
    walletAddress: string,
    contractMethod: string,
    args: any[],
  ): Promise<GasEstimate> => {
    try {
      const rpc = getRpc()
      const contract = new Contract(CONFIG.CONTRACT_ID)

      // Get the account for building the transaction
      let account
      try {
        account = await rpc.getAccount(walletAddress)
      } catch {
        // Fallback for simulation if account not found
        const { Account } = await import('@stellar/stellar-sdk')
        account = new Account(walletAddress, '0') as any
      }

      // Build unsigned transaction
      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: CONFIG.NETWORK_PASSPHRASE,
      })
        .addOperation(contract.call(contractMethod, ...args))
        .setTimeout(30)
        .build()

      // Simulate to get resource requirements
      const sim = await rpc.simulateTransaction(tx)

      // Check for errors
      const { Api: StellarRpcApi } = await import('@stellar/stellar-sdk/rpc')
      if (StellarRpcApi.isSimulationError(sim)) {
        return {
          success: false,
          error: sim.error ?? 'Simulation failed',
        }
      }

      if (!sim.result) {
        return {
          success: false,
          error: 'No simulation result',
        }
      }

      // Extract fees from the simulation result
      const simResult = sim.result
      if (!simResult) {
        return {
          success: false,
          error: 'Simulation returned no result',
        }
      }

      // The fee is the total cost in stroops
      // Base fee is typically 100 stroops per operation
      const baseFee = Number(BASE_FEE)

      // Soroban execution and storage fees are encoded in the simulation
      // For now, we estimate based on the resource requirements
      // The actual fee would be extracted from sim.result.minResourceFee

      // Try to extract detailed cost from the simulation
      // This is a conservative estimate based on Stellar documentation
      const totalFeeStroops = sim.minResourceFee ? Number(sim.minResourceFee) : baseFee * 10

      // Split estimated costs (these are heuristics):
      // Typically: 10% base, 50% execution, 40% storage
      const executionCost = Math.round(totalFeeStroops * 0.5)
      const storageCost = Math.round(totalFeeStroops * 0.4)
      const estimatedBaseFee = totalFeeStroops - executionCost - storageCost

      // Get current XLM price for USD conversion
      // For now, use a fixed rate or fetch from an oracle
      const stroopsPerUnit = 100 // placeholder; ideally fetch from market data
      const usdPerStroops = 0.000000099 // approximately 1 XLM = 0.15 USD = 1.5e-8 USD per stroops

      return {
        success: true,
        breakdown: {
          baseFee: estimatedBaseFee,
          executionCost,
          storageCost,
          totalCost: totalFeeStroops,
          totalCostInUsd: totalFeeStroops * usdPerStroops,
        },
        stroopsPerUnit,
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Unknown error'
      return {
        success: false,
        error: `Gas estimation failed: ${msg}`,
      }
    }
  }, [])

  return { estimateGas }
}
