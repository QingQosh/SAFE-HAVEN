import { describe, expect, it } from 'vitest'
import { runTokenVetting } from '../lib/tokenVetting'
import type { TokenMetadata } from '../types'

const metadata: TokenMetadata = {
  assetCode: 'SAFE',
  issuer: 'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF',
  name: null,
  description: null,
  image: null,
  totalSupply: 1_000_000n,
  holders: 42,
  issuerCreatedAt: new Date(Date.now() - 90 * 86_400_000).toISOString(),
  issuerTransactionCount: null,
  stellarExpertUrl: 'https://stellar.expert/explorer/testnet/asset/SAFE-GAAA',
}

describe('runTokenVetting', () => {
  it('passes when every required check passes', () => {
    const result = runTokenVetting(metadata, {
      minimumTvl: 500_000n,
      minimumCreatorAgeDays: 30,
      contractVerified: true,
      noKnownVulnerabilities: true,
    })

    expect(result.passed).toBe(true)
    expect(result.reasons).toEqual([])
  })

  it('reports each failed checklist item', () => {
    const result = runTokenVetting(metadata, {
      minimumTvl: 2_000_000n,
      minimumCreatorAgeDays: 120,
      contractVerified: false,
      noKnownVulnerabilities: false,
      knownReputationIssue: true,
    })

    expect(result.passed).toBe(false)
    expect(result.reasons).toHaveLength(5)
  })
})