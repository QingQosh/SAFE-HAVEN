# SAFE-HAVEN User Onboarding

SAFE-HAVEN is a Stellar Soroban vault. You deposit a supported token, choose when it unlocks, and later withdraw the full amount after the lock expires. You can cancel early, but the penalty selected at deposit time is deducted.

## Before you start

- Install the [Freighter wallet](https://www.freighter.app/) and create or import the account you intend to use.
- Set Freighter to the same network shown by the SAFE-HAVEN app. Testnet and mainnet are separate.
- Fund the account with the token you plan to deposit and enough native XLM for network fees.
- Confirm the token contract address. A token address is not the same thing as a wallet address.

Never share your secret key or recovery phrase. SAFE-HAVEN cannot recover them.

## First deposit

1. Open the published SAFE-HAVEN app and connect Freighter.
2. Check the network warning, wallet address, and contract status. Deposits are unavailable while the contract is paused.
3. Open **Deposit**.
4. Enter the token contract address. Native XLM is selected by default; custom tokens must be compatible with Stellar's token interface.
5. Enter the amount. The app reads token decimals and validates the configured maximum.
6. Choose an unlock date and time in your local timezone. The unlock must be in the future, at least 60 seconds away, and within the deployment's maximum lock duration (five years by default).
7. Choose an early-exit penalty from 0 to 10,000 basis points. `100 bps = 1%`; `0 bps` means no penalty.
8. Review the summary and approve the transaction in Freighter.
9. Wait for confirmation. The new deposit ID appears on the dashboard. Keep that ID and the transaction hash for support.

The transfer and vault record are created in one contract call. A failed transaction does not create a usable deposit.

## Worked example

Assume a deposit of **10 XLM**, using XLM's 7 decimal places, with a **250 bps (2.5%)** early-exit penalty:

- Amount in token base units: `10 * 10^7 = 100,000,000` stroops.
- Early-exit penalty: `10 * 2.5% = 0.25 XLM`.
- Early-cancellation refund: `10 - 0.25 = 9.75 XLM`.
- Normal withdrawal after unlock: `10 XLM` (network fee is separate).

For a custom token, calculate using that token's decimals and display unit. The contract stores integer base units.

## Lock time, withdrawal, and cancellation

**Lock time** is the interval between the current ledger time and the selected unlock time. The contract, not the browser clock, decides whether the vault is unlocked. The app counts down locally and rechecks the chain before enabling withdrawal.

After the unlock time, use the dashboard's **Withdraw** action or enter the deposit ID on the **Withdraw** tab. The full stored amount is sent to the depositor. `withdraw_to` is an SDK/contract API option for sending to another recipient; the current frontend's normal flow withdraws to the depositor.

Before unlock, use **Cancel**. The contract removes the deposit, sends the penalty to the configured fee recipient, and returns the remainder to the depositor. Cancellation after unlock is rejected; withdraw instead.

Ledger-based deposits use a ledger sequence rather than a timestamp. The contract supports them, but the current frontend creates timestamp-based deposits. Ledger time estimates use roughly 5 seconds per ledger and are not exact.

## FAQ

**Can I change the unlock date?**  No. Cancel the existing vault while it is locked, accept the configured penalty, and create a new deposit.

**Can I withdraw before unlock without a penalty?**  No. Early exit is cancellation and applies the penalty chosen when the deposit was created.

**What happens if I lose access to my wallet?**  SAFE-HAVEN cannot bypass wallet authorization. Recover the wallet through Freighter's supported recovery process.

**Does the admin control my tokens?**  The admin cannot withdraw funds to itself. Emergency withdrawal returns a deposit to its depositor. The admin can pause new deposits and may eventually renounce admin rights.

**Why is my custom token amount wrong?**  Token decimals determine base units. Verify the token contract address and metadata before signing.

**Why is my deposit ID not zero?**  IDs are monotonic per depositor and are not reused after withdrawal or cancellation.

**Are network fees refundable?**  No. Stellar transaction fees are separate from vault principal and penalties.

## Troubleshooting

| Message or symptom | What to check |
|---|---|
| `Network Mismatch` | Switch Freighter to the network configured by the app, then disconnect and reconnect. |
| `ContractPaused` or "Deposits are disabled" | Wait for the operator to unpause the contract; existing vaults remain withdrawable. |
| `InvalidAmount` / `AmountTooLarge` | Enter a positive amount within the deployment limit and account for token decimals. |
| `UnlockTimeNotInFuture` / `LockDurationTooShort` | Pick a time at least 60 seconds in the future; the chain timestamp is authoritative. |
| `LockDurationTooLong` | Choose an unlock within the configured maximum, five years by default. |
| `FundsStillLocked` | Wait until the chain confirms the unlock. Browser time reaching zero is not sufficient by itself. |
| `NoDepositFound` | Check the connected wallet, network, token/deposit ID, and that the vault was not already withdrawn or cancelled. |
| `MissingFeeRecipient` | A non-zero penalty requires a configured fee recipient; choose zero only if the deployment explicitly allows it. |
| Transaction rejected in Freighter | Review the transaction details and approve again. Never approve an unexpected token or recipient. |
| Confirmation timeout | Check the transaction hash in the configured explorer before retrying, to avoid submitting a duplicate action. |

## Checklist

- [ ] Freighter is installed and on the app's configured network.
- [ ] The wallet has the intended token and native XLM for fees.
- [ ] The token contract address is verified.
- [ ] Amount and token decimals are correct.
- [ ] Unlock time is at least 60 seconds away and within the maximum.
- [ ] Penalty and estimated refund are understood.
- [ ] The transaction details were reviewed in Freighter.
- [ ] Deposit ID and transaction hash were recorded.
- [ ] The dashboard shows the active vault.
- [ ] Before acting later, the wallet and network are checked again.

## Support

Report reproducible problems in [GitHub Issues](https://github.com/shortheartone/SAFE-HAVEN/issues). For a useful report, include the network, wallet public address, deposit ID, transaction hash, exact error, and steps to reproduce. Do not include secret keys, recovery phrases, or other credentials.
