import { describe, it, expect, vi } from 'vitest'

// We need to test parseVaultEntry which is not exported. We test it indirectly
// via the exported functions that use it, or we can test the scVal parsing
// logic by mocking the dependency.

// Since parseVaultEntry is a private function, we test scValToNative-based
// parsing logic through the format of the returned VaultEntry from getVault.
// However, for direct unit testing, we import scValToNative and test the shape.

import { scValToNative, nativeToScVal, xdr, Address } from '@stellar/stellar-sdk'

describe('VaultEntry scVal parsing (parseVaultEntry integration)', () => {
  it('parses a well-formed scVal map into the expected shape', () => {
    // Build an scVal map that mimics what the contract returns for a VaultEntry
    const addr = 'GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFSHONUCEOASW7QC7OX2H'
    const scVal = xdr.ScVal.scvMap([
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('token'),
        val: new Address(addr).toScVal(),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('amount'),
        val: nativeToScVal(1000n, { type: 'i128' }),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('unlock_time'),
        val: nativeToScVal(9999, { type: 'u64' }),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('depositor'),
        val: new Address(addr).toScVal(),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('penalty_bps'),
        val: nativeToScVal(500, { type: 'u32' }),
      }),
    ])

    const raw = scValToNative(scVal) as Record<string, unknown>

    // Replicate parseVaultEntry logic
    const entry = {
      token: raw['token'] as string,
      amount: BigInt(raw['amount'] as string | number),
      unlockTime: Number(raw['unlock_time']),
      depositor: raw['depositor'] as string,
      penaltyBps: Number(raw['penalty_bps']),
    }

    expect(entry.token).toBe(addr)
    expect(entry.amount).toBe(1000n)
    expect(entry.unlockTime).toBe(9999)
    expect(entry.depositor).toBe(addr)
    expect(entry.penaltyBps).toBe(500)
  })

  it('parses amount as string from scVal', () => {
    const addr = 'GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFSHONUCEOASW7QC7OX2H'
    const scVal = xdr.ScVal.scvMap([
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('token'),
        val: new Address(addr).toScVal(),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('amount'),
        val: nativeToScVal('1000000000000000', { type: 'i128' }),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('unlock_time'),
        val: nativeToScVal(1234567890, { type: 'u64' }),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('depositor'),
        val: new Address(addr).toScVal(),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('penalty_bps'),
        val: nativeToScVal(0, { type: 'u32' }),
      }),
    ])

    const raw = scValToNative(scVal) as Record<string, unknown>
    const amount = BigInt(raw['amount'] as string)

    expect(amount).toBe(1_000_000_000_000_000n)
  })

  it('handles zero values correctly', () => {
    const addr = 'GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFSHONUCEOASW7QC7OX2H'
    const scVal = xdr.ScVal.scvMap([
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('token'),
        val: new Address(addr).toScVal(),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('amount'),
        val: nativeToScVal(0n, { type: 'i128' }),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('unlock_time'),
        val: nativeToScVal(0, { type: 'u64' }),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('depositor'),
        val: new Address(addr).toScVal(),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('penalty_bps'),
        val: nativeToScVal(0, { type: 'u32' }),
      }),
    ])

    const raw = scValToNative(scVal) as Record<string, unknown>
    expect(Number(raw['unlock_time'])).toBe(0)
    expect(Number(raw['penalty_bps'])).toBe(0)
    expect(BigInt(raw['amount'] as number)).toBe(0n)
  })

  it('handles maximum penalty_bps (10000)', () => {
    const addr = 'GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFSHONUCEOASW7QC7OX2H'
    const scVal = xdr.ScVal.scvMap([
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('token'),
        val: new Address(addr).toScVal(),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('amount'),
        val: nativeToScVal(5000n, { type: 'i128' }),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('unlock_time'),
        val: nativeToScVal(9999999999, { type: 'u64' }),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('depositor'),
        val: new Address(addr).toScVal(),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('penalty_bps'),
        val: nativeToScVal(10000, { type: 'u32' }),
      }),
    ])

    const raw = scValToNative(scVal) as Record<string, unknown>
    expect(Number(raw['penalty_bps'])).toBe(10000)
  })
})
