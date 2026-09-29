# Smart Wallet Integration

SAFE-HAVEN is a Soroban contract. Smart wallets are represented by Soroban `Address` values and are not treated as externally owned accounts. Deposit ownership is the `depositor` address supplied to the contract, and authorization is checked with Soroban `require_auth()`.

## Safe-style and multisignature wallets

A multisignature wallet should submit `deposit` from its contract address after its own approval threshold has been met. The vault does not bypass the wallet's policy or inspect individual signers. The wallet contract remains responsible for collecting signatures and authorizing the token transfer.

For a relayer-funded deposit, use `deposit_for(payer, depositor, ...)`. The payer must authorize the call and fund the token transfer; the `depositor` receives ownership of the resulting vault position and is the address allowed to withdraw it.

This model also works for account-abstraction wallets: the wallet or its entry-point executes the contract call, while the wallet's Soroban authorization determines whether the operation is valid. There is no EVM `tx.origin` or EOA-only check in the deposit path.

## Session keys

A wallet can authorize a delegated payer for a bounded period:

1. Call `authorize_session_key(wallet, session_key, expires_at)` from the wallet.
2. The session key calls `deposit_with_session_key(session_key, wallet, token, amount, unlock_time, penalty_bps)`.
3. The vault records the deposit under `wallet`, but transfers the tokens from `session_key`.
4. The wallet can call `revoke_session_key(wallet, session_key)` at any time.

The session key authorization is scoped to one wallet and expires according to ledger time. A session key cannot spend tokens held by the wallet; wallet-funded operations must still be executed and authorized by the wallet contract itself. Use withdrawal whitelists when a wallet needs to constrain approved recipients.

## Integration checklist

- Keep the smart wallet address as the `depositor`; do not substitute a relayer or signer address.
- Ensure the wallet authorizes both the vault invocation and any token transfer it originates.
- Use `deposit_for` for third-party funding and verify the payer has sufficient balance.
- Use an explicit expiry for every session key and revoke keys when the session ends.
- Test the wallet's multisignature threshold, rejected signatures, replay protection, and token authorization on the target network.
- Monitor `deposit` events by depositor address rather than by the address that submitted the transaction.

The contract intentionally does not implement a custom wallet, signature aggregation, recovery, or universal support for every smart-wallet protocol.
