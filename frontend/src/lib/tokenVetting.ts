import { CONFIG } from '../config'
import type { TokenMetadata, TokenVettingChecks, TokenVettingResult } from '../types'

interface HorizonAsset {
  _embedded?: {
    records?: Array<{
      amount?: string
      num_accounts?: number
    }>
  }
}

interface HorizonAccount {
  created_at?: string
  balances?: Array<{ asset_type: string; balance: string }>
}

interface HorizonTransactionPage {
  _embedded?: { records?: unknown[] }
}

export interface VettingOptions {
  minimumTvl: bigint
  minimumCreatorAgeDays: number
  contractVerified: boolean
  noKnownVulnerabilities: boolean
  knownReputationIssue?: boolean
}

function horizonUrl(path: string): string {
  return `${CONFIG.HORIZON_URL.replace(/\/$/, '')}${path}`
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url)
  if (!response.ok) throw new Error(`Stellar API request failed (${response.status})`)
  return response.json() as Promise<T>
}

/** Fetch issuer and supply metadata from Stellar Horizon. */
export async function fetchTokenMetadata(assetCode: string, issuer: string): Promise<TokenMetadata> {
  const encodedCode = encodeURIComponent(assetCode)
  const encodedIssuer = encodeURIComponent(issuer)
  const asset = await fetchJson<HorizonAsset>(
    horizonUrl(`/assets?asset_code=${encodedCode}&asset_issuer=${encodedIssuer}&limit=1`),
  )
  const account = await fetchJson<HorizonAccount>(horizonUrl(`/accounts/${encodedIssuer}`))
  let firstTransaction: HorizonTransactionPage | null = null
  try {
    firstTransaction = await fetchJson<HorizonTransactionPage>(
      horizonUrl(`/accounts/${encodedIssuer}/transactions?order=asc&limit=1`),
    )
  } catch {
    // Account history can be unavailable for newly created or pruned accounts.
  }

  const firstRecord = firstTransaction?._embedded?.records?.[0] as { created_at?: string } | undefined
  const assetRecord = asset._embedded?.records?.[0]
  const totalSupply = BigInt(assetRecord?.amount ?? '0')
  return {
    assetCode,
    issuer,
    name: null,
    description: null,
    image: null,
    totalSupply,
    holders: assetRecord?.num_accounts ?? 0,
    issuerCreatedAt: firstRecord?.created_at ?? account.created_at ?? null,
    issuerTransactionCount: null,
    stellarExpertUrl: `https://stellar.expert/explorer/${CONFIG.NETWORK_PASSPHRASE === 'Public Global Stellar Network ; September 2015' ? 'public' : 'testnet'}/asset/${assetCode}-${issuer}`,
  }
}

/** Evaluate the token checklist using fetched metadata and operator-supplied audit signals. */
export function runTokenVetting(metadata: TokenMetadata, options: VettingOptions): TokenVettingResult {
  const issuerAgeDays = metadata.issuerCreatedAt
    ? (Date.now() - Date.parse(metadata.issuerCreatedAt)) / 86_400_000
    : 0
  const checks: TokenVettingChecks = {
    contractVerified: options.contractVerified,
    tvlAboveThreshold: metadata.totalSupply >= options.minimumTvl,
    noKnownVulnerabilities: options.noKnownVulnerabilities,
    creatorReputable: !options.knownReputationIssue,
    ageAboveThreshold: issuerAgeDays >= options.minimumCreatorAgeDays,
  }
  const reasons: string[] = []
  if (!checks.contractVerified) reasons.push('Contract or token verification is not confirmed.')
  if (!checks.tvlAboveThreshold) reasons.push('Reported token supply is below the TVL threshold.')
  if (!checks.noKnownVulnerabilities) reasons.push('Known vulnerability concerns are present.')
  if (!checks.creatorReputable) reasons.push('Creator reputation check failed.')
  if (!checks.ageAboveThreshold) reasons.push('Issuer account is younger than the required age.')
  return { metadata, checks, passed: reasons.length === 0, reasons }
}