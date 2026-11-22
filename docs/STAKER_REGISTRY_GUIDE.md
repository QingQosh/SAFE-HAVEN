# Staker Registry User Guide

This guide describes the staker registry entry points in the SAFE-HAVEN Soroban contract: registration, penalty accounting, reward calculations, and claim transactions.

> **Important implementation limitations:** Registration stores a numeric weight; it does not transfer or lock tokens. A successful reward claim updates the contract's reward-pool accounting and emits an event, but the current `claim_staker_rewards` implementation does **not** transfer tokens to the staker. The pool is also a single numeric balance, not separated by token. Treat these calls as registry/accounting operations, not as a token-backed staking or payout product.

## Before you begin

You need:

- The deployed SAFE-HAVEN contract ID and its network (testnet or mainnet).
- A Stellar account that can authorize transactions for the staker address.
- The Soroban RPC URL and matching network passphrase.
- The token's base-unit convention if you want to interpret numeric values. The registry itself accepts only an `i128`; it does not accept a token address or decimal precision.

The current contract has no public query entry point for an individual stake, total stake, or the staker reward pool. Applications that display these values need a reliable event indexer or other trusted state source; do not assume a wallet UI or contract getter exists.

## What registration means

Call `register_staker(staker, amount)` with the staker's address and a positive `i128`.

- The staker must authorize the call.
- The amount must be greater than zero.
- A later call **replaces** the registered amount; it does not add to it.
- The amount is used as a relative weight in reward calculations.
- No token address is provided, and no tokens move when registering.
- There is no unregister operation. Registering zero or a negative amount fails. A positive lower amount can reduce the weight, but cannot remove it.

### Register with the Soroban CLI

Use a source key that can authorize `STAKER_ADDRESS`:

```bash
export NETWORK=testnet
export RPC_URL=https://soroban-testnet.stellar.org
export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export CONTRACT_ID="CC..."
export STAKER_SECRET="S..."
export STAKER_ADDRESS="G..."

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$STAKER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- register_staker \
  --staker "$STAKER_ADDRESS" \
  --amount 100000000
```

The example records the integer `100000000`. Whether that represents one whole token depends on the convention chosen by your application; the contract does not normalize decimal places.

### Registration walkthrough

1. Confirm that you are interacting with the intended contract and network.
2. Decide on a positive numeric weight. No tokens are deposited by this call.
3. Submit `register_staker` with your address and weight.
4. Approve the transaction from the wallet for that address.
5. Wait for successful transaction confirmation. A successful call emits `StakerRegistered` with the address, amount, and ledger timestamp.
6. To change your weight, repeat the call with the new full amount.

## How penalties enter the pool

For timestamp-based and ledger-based single-token deposits, `cancel_deposit` before unlock calculates a penalty and applies this split:

```text
penalty = floor(deposit_amount × penalty_bps / 10,000)
fee_recipient_share = floor(penalty × 3,000 / 10,000)
staker_pool_addition = penalty - fee_recipient_share
```

So the nominal split is 30% to the fee recipient and 70% added to the staker rewards pool. Integer division is used; any split-rounding remainder goes to the staker pool. For example, a 1,000-unit deposit with a 1,000-bps (10%) penalty has a penalty of 100 units: 30 go to the fee recipient and 70 are recorded in the pool.

The early-cancel path for multi-token deposits currently sends the full penalty to the fee recipient and does not add it to the staker pool. Ordinary withdrawals do not create staker-pool rewards. The pool stores only a number, without token identity, so it does not represent a guaranteed amount of any particular asset.

## Reward calculation and distribution

At each successful claim, the contract calculates:

```text
claim_amount = floor(staker_registered_amount × current_rewards_pool / current_total_staked)
```

The claim fails if the caller is not registered, the pool or total stake is zero, or the result rounds down to zero. The caller must authorize the claim. On success, the contract subtracts the calculated amount from the pool, increments the staker's cumulative claimed-accounting value, and emits `RewardsClaimed`.

For example, if your registered amount is 1,000, total registered amount is 4,000, and the pool is 700, a claim made while the pool is 700 calculates:

```text
floor(1,000 × 700 / 4,000) = 175
```

Claims are calculated against the pool **as it exists at that moment**. The contract does not snapshot each participant's entitlement when penalties arrive. As each claim reduces the shared pool, the order and timing of claims can affect later claim amounts; integer rounding can leave a balance in the pool. Changing a registered amount also changes the total stake immediately. The contract does not impose a reward epoch or waiting period.

> **This is accounting, not an on-chain payout in the current implementation.** `claim_staker_rewards` returns `Result<(), VaultError>` and does not call a token transfer. The event and reduced pool balance do not mean that tokens were delivered to the wallet.

## Claim walkthrough

1. Check the current contract and network.
2. Confirm that the staker address has registered a positive amount.
3. Use an event indexer or another trusted state source to confirm that the pool and total stake should produce a positive integer amount. There is no public contract getter for these values today.
4. Submit `claim_staker_rewards(staker)` and authorize it with that staker's wallet.
5. Wait for transaction confirmation. A successful call emits `RewardsClaimed` with the calculated amount and timestamp and decreases the pool's accounting balance.
6. Do **not** expect a token transfer from this contract from the current claim implementation.

### Claim with the Soroban CLI

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$STAKER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- claim_staker_rewards \
  --staker "$STAKER_ADDRESS"
```

## TypeScript example (`@stellar/stellar-sdk`)

The following builds a Soroban transaction for either registry call, simulates it to obtain Soroban resources, and assembles the transaction for wallet signing. The wallet adapter must sign the returned XDR for the same network; submit the signed XDR and check the RPC transaction status before showing success.

```ts
import {
  Address,
  BASE_FEE,
  Contract,
  nativeToScVal,
  rpc,
  TransactionBuilder,
} from '@stellar/stellar-sdk'

// Replace these with values for the same deployed network.
const RPC_URL = 'https://soroban-testnet.stellar.org'
const CONTRACT_ID = 'CC...'
const NETWORK_PASSPHRASE = 'Test SDF Network ; September 2015'
const stakerAddress = 'G...'

// Implement this adapter with the wallet provider used by your application.
declare function walletSign(xdr: string, networkPassphrase: string): Promise<string>

const server = new rpc.Server(RPC_URL)
const contract = new Contract(CONTRACT_ID)

async function buildStakerTransaction(
  stakerAddress: string,
  method: 'register_staker' | 'claim_staker_rewards',
  amount?: bigint,
): Promise<string> {
  const args = [new Address(stakerAddress).toScVal()]
  if (method === 'register_staker') {
    if (amount === undefined || amount <= 0n) {
      throw new Error('Registration requires a positive i128 amount')
    }
    args.push(nativeToScVal(amount, { type: 'i128' }))
  }

  const account = await server.getAccount(stakerAddress)
  const transaction = new TransactionBuilder(account, {
    fee: (Number(BASE_FEE) * 10).toString(),
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(300)
    .build()

  const simulation = await server.simulateTransaction(transaction)
  if (rpc.Api.isSimulationError(simulation)) {
    throw new Error(`Soroban simulation failed: ${simulation.error}`)
  }

  return rpc.assembleTransaction(transaction, simulation).build().toXDR()
}

async function signSubmitAndConfirm(unsignedXdr: string): Promise<string> {
  const signedXdr = await walletSign(unsignedXdr, NETWORK_PASSPHRASE)
  const submitted = await server.sendTransaction(
    TransactionBuilder.fromXDR(signedXdr, NETWORK_PASSPHRASE),
  )
  if (submitted.status === 'ERROR') {
    throw new Error(`Transaction submission failed: ${submitted.errorResult}`)
  }

  for (let attempt = 0; attempt < 30; attempt++) {
    await new Promise((resolve) => setTimeout(resolve, 2_000))
    const result = await server.getTransaction(submitted.hash)
    if (result.status === rpc.Api.GetTransactionStatus.SUCCESS) {
      return submitted.hash
    }
    if (result.status === rpc.Api.GetTransactionStatus.FAILED) {
      throw new Error(`Transaction failed on-chain: ${submitted.hash}`)
    }
  }
  throw new Error(`Transaction confirmation timed out: ${submitted.hash}`)
}

// Register (or replace) the numeric weight; this does not transfer tokens.
const unsignedRegisterXdr = await buildStakerTransaction(
  stakerAddress,
  'register_staker',
  100_000_000n,
)
const registerTxHash = await signSubmitAndConfirm(unsignedRegisterXdr)

// Later, after a positive accounting amount is available, submit a claim.
// This claim changes contract accounting; it does not transfer tokens.
// const unsignedClaimXdr = await buildStakerTransaction(
//   stakerAddress,
//   'claim_staker_rewards',
// )
// const claimTxHash = await signSubmitAndConfirm(unsignedClaimXdr)
```

`RPC_URL`, `CONTRACT_ID`, `NETWORK_PASSPHRASE`, and `stakerAddress` must all refer to the same deployed network/account context. Simulation is not a substitute for wallet authorization or final transaction confirmation.

## FAQ

### Do I need to send tokens to register?

No tokens are transferred by `register_staker`. It records a positive integer weight only. The amount is not tied to an asset or decimal scale by the contract.

### Are rewards transferred to my wallet when I claim?

No. In the current contract code, a claim reduces the reward-pool accounting value and emits `RewardsClaimed`; it does not transfer tokens. Do not treat a successful claim as a payment.

### Which penalties contribute to the staker pool?

Early cancellation of timestamp-based and ledger-based single-token deposits adds the staker share. Multi-token early cancellation currently routes its full penalty to the fee recipient instead.

### Why did my claim fail?

Common reasons include: the address was not registered, the reward pool is empty, the total registered amount is zero, or integer division produces a zero-unit result. The staker must also authorize the transaction, and the transaction must target the correct network and contract.

### Can I unregister or set my amount to zero?

No. Registration requires an amount greater than zero, and there is no unregister method. You can replace your weight with a smaller positive number, but it remains in the total.

### Can I read my stake and the pool with a contract query?

Not through the current public contract interface. A client needs an indexed event/state source until read-only registry query methods are provided.

### Does each staker receive a fixed percentage of each penalty?

No. Each claim uses the current pool and current total registered weight. Claims reduce the shared pool, so claim timing/order and integer rounding affect later claim calculations.
