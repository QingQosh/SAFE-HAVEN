# SAFE-HAVEN Knowledge Base and FAQ

## Purpose

This document serves as a centralized repository of frequently asked questions, troubleshooting guides, and reference material for SAFE-HAVEN users, developers, and operators. It is organized by category, maintained based on support tickets and community discussions, and regularly updated as new patterns emerge.

---

## Table of Contents

- [Getting Started](#getting-started)
- [Deposits](#deposits)
- [Withdrawals and Cancellations](#withdrawals-and-cancellations)
- [Staker Registry and Rewards](#staker-registry-and-rewards)
- [Advanced Topics](#advanced-topics)
- [Troubleshooting](#troubleshooting)
- [Security and Safety](#security-and-safety)
- [Contributing to the FAQ](#contributing-to-the-faq)

---

## Getting Started

### Q1. What is SAFE-HAVEN?

**A:** SAFE-HAVEN is a decentralized smart contract on the Stellar blockchain (Soroban) that lets you lock tokens until a future date. You set:
- **How long** to lock (specific date or Stellar ledger sequence)
- **Early exit penalty** (0–100% of the deposit)

If you exit early, you pay the penalty. If you wait until unlock time, you get all your tokens back.

**Why use it?**
- Enforce your own discipline (can't panic-sell if funds are locked)
- Transparent vesting schedules (no intermediaries)
- Trustless (the contract has no admin power to steal funds)

**Key difference from other vaults:**
- Your tokens go to **you**, not the admin
- You set the penalty; the contract enforces it
- Admin can be permanently renounced

---

### Q2. Is SAFE-HAVEN safe to use?

**A:** SAFE-HAVEN has been designed with security first, but no smart contract is 100% risk-free. Here's what we've done:

**Security practices:**
- Open-source code (you can audit it yourself)
- Comprehensive unit tests (48+ tests)
- Storage safety measures (TTL protection, checked-effects-interactions pattern)
- Disaster recovery playbook (funds can be recovered if needed)
- Security contact (report vulnerabilities privately: see SECURITY.md)

**Risks to be aware of:**
- Smart contract bugs (though we test extensively)
- Soroban platform issues (rare but possible)
- User error (e.g., wrong network, losing recovery phrase)
- Your wallet being compromised

**What we recommend:**
1. Start small (test with a small amount first)
2. Use testnet to practice
3. Never share your seed phrase
4. Verify the contract ID before depositing (it's in the URL)
5. Keep your recovery phrase safe

---

### Q3. How do I get started?

**A:** Follow these steps:

1. **Get a wallet**
   - Download Freighter browser extension (Freighter.app)
   - Create or import your wallet

2. **Get tokens**
   - Testnet: Use the faucet (link in SAFE-HAVEN UI) to get test XLM or USDC
   - Mainnet: Buy tokens via an exchange or DEX (Ramp Network has a built-in option)

3. **Go to SAFE-HAVEN**
   - Testnet: [testnet URL]
   - Mainnet: [mainnet URL]

4. **Connect your wallet**
   - Click "Connect Wallet" in the top-right
   - Approve the connection in Freighter

5. **Make your first deposit**
   - Click "Deposit" tab
   - Choose token, amount, unlock time, penalty
   - Confirm the transaction

---

### Q4. What networks does SAFE-HAVEN support?

**A:** SAFE-HAVEN supports:

- **Stellar Mainnet** (production, real XLM and other tokens)
- **Stellar Testnet** (for testing, fake tokens only)

You can switch networks using the network selector in the top-right corner of the frontend.

**Be careful:** If you're on testnet and send tokens to an address, those tokens only exist on testnet. Mainnet and testnet are separate blockchains.

---

### Q5. Which tokens does SAFE-HAVEN support?

**A:** SAFE-HAVEN supports any **SEP-41 token** on Stellar. That includes:

- **XLM** (native Stellar asset)
- **USDC** (Circle, stablecoin)
- **SLQ** (Soroban Liquid Staking)
- **Wrapped BTC** (Bitcoin on Stellar)
- **Wrapped ETH** (Ethereum on Stellar)
- Custom tokens issued by projects

**How to use a custom token:**
1. Know its contract address (e.g., CABC...)
2. On the Deposit page, paste the address in the token field
3. The UI will verify the token and show the symbol

**Note:** If a token is flagged as high-risk, the UI will show a warning. Use your judgment.

---

## Deposits

### Q6. What does "unlock time" mean?

**A:** Unlock time is when your tokens become available to withdraw. It's either:

- **Timestamp:** A specific date and time (e.g., "December 31, 2026 at 3:00 PM UTC")
- **Ledger sequence:** A specific Stellar ledger block number (e.g., "Ledger 50,000,000")

**Timestamp-based deposits** are most common. You pick a calendar date, and when that time arrives, you can withdraw.

**Ledger-based deposits** are for advanced users who want to lock until a specific block is reached. Ledgers close roughly every 5 seconds, so you can calculate:
```
Time ≈ (target_ledger - current_ledger) × 5 seconds
```

---

### Q7. What does "penalty" mean?

**A:** The penalty is what you forfeit if you exit early. It's expressed as a **basis points** (bps), where 10,000 bps = 100%.

**Examples:**
- **Penalty = 0 bps** → No penalty for early exit (same as no lock)
- **Penalty = 1,000 bps** → 10% penalty (you lose 10% of the deposit if you exit early)
- **Penalty = 5,000 bps** → 50% penalty (you lose half of the deposit if you exit early)
- **Penalty = 10,000 bps** → 100% penalty (you forfeit the entire deposit if you exit early)

**How penalties are split:**
When you exit early:
- 30% goes to the **fee recipient** (usually the protocol or a DAO treasury)
- 70% goes to the **staker rewards pool** (shared among registered stakers)
- The remainder goes back to you

**Example:**
You deposit 1,000 tokens with a 1,000 bps (10%) penalty. You exit early.
- Penalty amount: 100 tokens
- Fee recipient gets: 30 tokens
- Staker pool gets: 70 tokens
- You get: 900 tokens back

---

### Q8. How long can I lock tokens?

**A:** Lock duration limits depend on the deposit type:

**Timestamp-based deposits:**
- **Minimum:** 60 seconds (1 minute)
- **Maximum:** ~5 years (configurable)

**Ledger-based deposits:**
- **Minimum:** 12 ledgers (~60 seconds)
- **Maximum:** Unlimited (you can lock until ledger 4 billion if you want)

**In practice:**
- Most deposits are days to months
- Very long locks (years) are less common
- You can always exit early if needed (paying the penalty)

---

### Q9. Can I deposit tokens for someone else?

**A:** Yes, with the `deposit_for` function. You fund the deposit, but the tokens go to the beneficiary's address. This is useful for:
- **Vesting for contractors:** You deposit, contractor receives tokens only after vesting period
- **Gift with a lock:** You lock tokens as a gift; recipient claims them later
- **Fiduciary deposits:** You hold tokens on behalf of another party

**How it works:**
1. You sign the transaction (you're the payer)
2. You specify the beneficiary address
3. Tokens are locked in the contract
4. Only the beneficiary can unlock and withdraw

---

### Q10. Can I deposit multiple tokens at once?

**A:** Yes (if enabled). SAFE-HAVEN supports multi-token deposits where you can lock up to 5 different tokens in a single deposit. All tokens share the same unlock time and penalty.

**Example:**
You lock 100 XLM + 50 USDC + 10 wrapped BTC, all unlocking on the same date with the same penalty. If you exit early, the penalty applies to the whole bundle.

**Note:** This feature must be enabled on the contract you're using. Check the contract's `get_constants()` to see if multi-token is supported.

---

## Withdrawals and Cancellations

### Q11. How do I withdraw my tokens?

**A:** Once the unlock time has passed, go to the Withdraw tab:

1. Click "Withdraw" in the tab bar
2. Choose the deposit you want to withdraw
3. Click "Withdraw"
4. Confirm the transaction in Freighter
5. Tokens return to your wallet

**That's it.** No fees, no delays, no permissions needed.

---

### Q12. What if I want to exit early?

**A:** Use the "Cancel Deposit" button to exit early. You'll get your tokens back immediately, but you'll lose the penalty (which is split between fee recipients and stakers).

**Process:**
1. Go to the Withdraw tab
2. Find the deposit you want to exit
3. Click "Cancel Deposit" (not "Withdraw")
4. Confirm the transaction
5. You receive your tokens minus the penalty

**Important:** Once you cancel, the deposit is gone. You can't undo it.

---

### Q13. Can I withdraw to a different address?

**A:** Yes, with the `withdraw_to` function. Instead of withdrawing to your own wallet, you can send the unlocked tokens to any address.

**Use case:**
- Sending vested tokens directly to a recipient
- Payouts to multiple addresses from a single deposit
- Vault management on behalf of others

**Note:** This must be explicitly supported by the contract. Check if `withdraw_to` is available.

---

### Q14. What happens if I lose my wallet access?

**A:** If you lose access to your wallet:

1. **If you have your recovery phrase:** Import your wallet into a new Freighter instance and withdraw normally
2. **If you lost your recovery phrase:** Your tokens are locked in the contract and cannot be accessed unless:
   - The admin uses emergency withdrawal (tokens go back to your original address)
   - Or you recover your wallet's private key somehow

**Prevention:**
- Save your recovery phrase in a safe place (not on your computer)
- Consider a hardware wallet (Ledger, Trezor) for large amounts
- Test your recovery phrase on testnet before using mainnet

---

### Q15. Can the admin take my tokens?

**A:** No. The SAFE-HAVEN contract is designed so that:
- The admin **cannot** withdraw your tokens
- The admin **can only** return tokens to you in emergencies (e.g., contract bug)
- The admin can be **permanently renounced**, making the contract fully decentralized

If there's a critical bug or attack, the admin can call `emergency_withdraw(your_address, deposit_id)`, which sends your tokens **directly to you**, not the admin.

---

## Staker Registry and Rewards

### Q16. How do I become a staker?

**A:** Register with an amount of tokens:

1. Go to the Admin or Staking page (if available)
2. Click "Register as Staker"
3. Enter the amount of tokens you want to stake
4. Confirm the transaction
5. You're now registered and earning rewards

**What does staking do?**
When other users exit their deposits early, the penalty is split:
- 30% to the fee recipient
- 70% to the staker rewards pool

Your share of the staker pool is proportional to your stake relative to total staked.

**Example:**
- You stake 1,000 tokens
- Total all stakers: 10,000 tokens
- Your share: 10%

If the staker pool accumulates 1,000 tokens from penalties, you can claim 100 tokens.

---

### Q17. How do I claim staker rewards?

**A:** Go to the Admin or Staking page:

1. Click "Claim Rewards"
2. Confirm the transaction
3. Your rewards are transferred to your wallet

**Requirements:**
- You must be a registered staker
- There must be rewards in the pool
- Your proportional reward must be > 0 tokens

**Note:** Rewards are calculated at claim time, not continuously. The more often users exit early, the more the pool grows.

---

### Q18. Can I increase or decrease my stake?

**A:** Yes, re-register with a new amount. If you want to:
- **Increase stake:** Register with a higher amount
- **Decrease stake:** Register with a lower amount

Your stake is updated immediately. Your proportional reward share is recalculated based on the new stake and total staked.

---

### Q19. What happens if I unregister as a staker?

**A:** You can register with an amount of 0 to effectively unregister, but there's no explicit `unregister` function. Once you register with 0:
- You stop earning new rewards
- You can still claim any rewards you've already earned
- Your stake is removed from the total

---

## Advanced Topics

### Q20. What's the difference between timestamp and ledger deposits?

**A:** Both lock your tokens until a condition is met, but the condition is different:

| Feature | Timestamp | Ledger |
|---|---|---|
| **Unlock condition** | Specific date/time | Specific ledger number |
| **Use case** | "Release on 2026-12-31" | "Release at block 50,000,000" |
| **Precision** | Exact to the second | Exact to the ledger |
| **Time estimate** | Known (wall-clock) | Estimated (~5 sec/ledger) |
| **Best for** | Vesting schedules, events | Block-based logic, advanced |
| **Frontend support** | Full | Limited (CLI/SDK only) |

**Practical difference:**
- Timestamp: you know exactly when you get your tokens back (it's a calendar date)
- Ledger: you know the unlock condition but the exact time varies slightly (network conditions affect ledger close times)

**Use ledger if:**
- You need block-based logic (e.g., "release after this specific block")
- You're building smart contract automation that reads block height

**Use timestamp if:**
- You want a simple calendar date (most users)
- You're on the frontend UI (easier to use)

---

### Q21. Can I automate my deposits?

**A:** For smart contracts or programmatic access:

1. **Use the Stellar SDK:**
   ```javascript
   import { Contract } from '@stellar/stellar-sdk';
   
   const contract = new Contract(contractId, networkPassphrase);
   const tx = contract.call(
     'deposit',
     depositor,
     token,
     amount,
     unlockTime,
     penaltyBps
   );
   ```

2. **For batch operations:**
   - Use `get_vault_batch()` to fetch multiple deposits
   - Use `get_deposit_batch()` to fetch multiple deposits per depositor

3. **For event tracking:**
   - Listen to contract events via RPC subscription
   - Index events in your own database

---

### Q22. How do I query my deposits on-chain?

**A:** Use read-only contract functions:

```bash
# Get all your deposit IDs
soroban contract invoke --id CONTRACT_ID --network public \
  -- get_deposit_ids --depositor YOUR_ADDRESS

# Get a specific deposit
soroban contract invoke --id CONTRACT_ID --network public \
  -- get_vault --depositor YOUR_ADDRESS --id DEPOSIT_ID

# Get time remaining (in seconds)
soroban contract invoke --id CONTRACT_ID --network public \
  -- time_remaining --depositor YOUR_ADDRESS --id DEPOSIT_ID
```

---

### Q23. Can I use SAFE-HAVEN in a DAO?

**A:** Yes! SAFE-HAVEN works well for DAOs to:

- **Vesting team tokens:** Deposit team allocation locked for N months
- **Incentive programs:** Lock user deposits and share penalties with stakers (governance participants)
- **Treasury management:** Lock seasonal or event-specific treasuries
- **Contributor rewards:** Vesting for contributors with a transparent on-chain schedule

**DAO setup:**
1. Make your DAO the `fee_recipient` (receives 30% of penalties)
2. Register DAO governance token holders as stakers (earn 70% of penalties)
3. Create governance proposals for large deposits or early releases

---

## Troubleshooting

### Q24. My transaction is stuck or slow. What should I do?

**A:** Stellar transactions usually complete in seconds, but if you're waiting:

1. **Wait 1–2 more minutes**
   - Soroban RPC sometimes takes a moment to process

2. **Check the transaction status**
   - Go to Stellar Expert (link in the UI)
   - Paste your transaction hash (shown in the UI)
   - Look for "Confirmed" or "Error"

3. **If it fails:**
   - Read the error message (e.g., "ContractPaused", "FundsStillLocked")
   - See the Troubleshooting section below

4. **If it's genuinely stuck (>10 minutes):**
   - Refresh the page
   - Try the transaction again (it may have already gone through)
   - Ask in Discord with your transaction hash

**Common reasons for failures:**
- **"ContractPaused"** — The contract is paused for maintenance. Wait and try again.
- **"FundsStillLocked"** — Unlock time hasn't passed yet. You can exit early with a penalty.
- **"NoDepositFound"** — Deposit ID doesn't exist. Check you selected the right deposit.

---

### Q25. I see "Network Mismatch" error. What does that mean?

**A:** Your wallet is on a different network than the contract.

**Fix:**
1. Look at the top-right of the SAFE-HAVEN UI (network indicator)
2. Note which network is selected (e.g., "Testnet" or "Mainnet")
3. Click the network switcher and select the **same network**
4. In Freighter, also switch to the same network
5. Refresh the page and try again

**Example:**
- SAFE-HAVEN is on Testnet
- Your Freighter is on Mainnet
- → Change Freighter to Testnet
- → Retry the transaction

---

### Q26. I'm getting "Unauthorized" error. What's wrong?

**A:** This usually means:

1. **Your wallet isn't connected**
   - Click "Connect Wallet" and approve in Freighter

2. **You're using a different address than the one that created the deposit**
   - Deposits are tied to the address that created them
   - If you need to access from a different address, ask the original depositor to use `withdraw_to` to send tokens to your address

3. **The contract requires you to sign (and you're not)**
   - All mutating operations (deposit, withdraw, cancel) require a signature
   - Approve the transaction in Freighter when prompted

---

### Q27. My deposit shows 0 tokens remaining. What happened?

**A:** Either:

1. **The lock has expired** — If the unlock time has passed, the contract shows 0 until you withdraw. Withdrawing will return your full balance.

2. **You've already withdrawn** — If you already claimed the tokens, the deposit no longer exists.

3. **The deposit was cancelled** — If you exited early with a penalty, some tokens went to penalties and the deposit is gone.

To check, go to the Dashboard and look for the deposit status. If it's not listed, it's been withdrawn or cancelled.

---

### Q28. How do I report a bug?

**A:** If you find a bug (e.g., transaction fails unexpectedly, UI freezes):

1. **Gather info:**
   - Transaction hash (if applicable)
   - Network (testnet or mainnet)
   - Steps to reproduce
   - Error message (if any)
   - Browser and wallet version

2. **Report it:**
   - GitHub: Open an issue at [repo]
   - Discord: Post in #support with details
   - Email: security@example.com (if security-related)

3. **Do NOT:**
   - Share your private key or recovery phrase
   - Share other users' addresses or transactions (privacy)
   - Make false claims about fund loss without evidence

---

### Q29. My wallet won't connect. Help!

**A:** Try these steps:

1. **Check if Freighter is installed**
   - Visit freighter.app and install the extension
   - Refresh the page

2. **Check if Freighter is enabled in browser extensions**
   - Extension menu → Freighter → Allow it to run on this site

3. **Try incognito mode**
   - Some ad blockers interfere; test in private browsing

4. **Check the Soroban RPC**
   - If the RPC is down, nothing will work
   - Check Stellar status page for outages

5. **Try a different RPC endpoint**
   - Some RPC servers are slower than others
   - Mainnet: Try soroban-mainnet.stellar.org
   - Testnet: Try soroban-testnet.stellar.org

6. **Clear cache and refresh**
   - Press Ctrl+Shift+Delete (or Cmd+Shift+Delete on Mac)
   - Clear browser cache
   - Refresh the page

---

## Security and Safety

### Q30. How can I verify I'm on the real SAFE-HAVEN?

**A:** Check these signs:

1. **Contract ID is correct**
   - Mainnet: [official contract ID]
   - Testnet: [official testnet contract ID]
   - Find this in the README or docs

2. **URL is correct**
   - Mainnet: [official URL]
   - Testnet: [official testnet URL]
   - Check for typos or lookalike domains

3. **Certificate is valid**
   - Click the lock icon in the address bar
   - Verify the domain is legitimate

4. **No warnings**
   - Browser doesn't show "Not Secure" or certificate errors
   - Browser security features don't flag the site

**If unsure:**
- Go directly from the official README.md
- Ask in Discord first
- Never click random links in chat

---

### Q31. What if I think my wallet is compromised?

**A:** Move quickly:

1. **Immediately withdraw all funds from SAFE-HAVEN**
   - Go to the Withdraw tab
   - Withdraw any active deposits
   - Transfer tokens to a safe wallet

2. **Create a new wallet**
   - Use Freighter to create a brand-new wallet
   - Write down the recovery phrase (in a safe place)

3. **Do NOT use the compromised wallet again**
   - Your seed phrase is exposed
   - Any funds there can be stolen

4. **Report suspicious activity**
   - If you see unauthorized withdrawals, note the transaction hash
   - Report to Discord #support with hash
   - Email security@example.com

---

### Q32. What's the best way to secure my funds?

**A:** Multi-layered approach:

**Recovery phrase:**
- Write it on paper (never digital)
- Store in a safe, safe deposit box, or fireproof safe
- Never photograph it or email it
- Never give it to anyone

**Wallet software:**
- Use Freighter (well-maintained, trusted)
- Keep it updated
- Don't install suspicious browser extensions

**On public Wi-Fi:**
- Avoid signing transactions on public Wi-Fi
- If you must, use a VPN
- Better: use a hardware wallet (Ledger, Trezor)

**Amount:**
- Only keep on hot wallet (browser) what you can afford to lose
- Large amounts go on hardware wallet or cold storage

**Backup:**
- Test your recovery phrase on testnet before mainnet
- Verify you can recover before putting significant funds at risk

---

### Q33. Is SAFE-HAVEN a bank? Do you hold my funds?

**A:** No. SAFE-HAVEN is a smart contract (code running on the blockchain). You always control your funds:

- **You hold the deposit** — It sits in the contract, only you can unlock it
- **You sign every transaction** — No one can move your tokens without your signature
- **The contract has no admin power to steal** — By design
- **If SAFE-HAVEN team disappears** — Your deposits still work; the code never changes

SAFE-HAVEN is not a custodian. You're the custodian of your own funds.

---

## Contributing to the FAQ

### How to suggest an FAQ entry

Found a common question that should be here? Suggest it:

1. **Check if it's already here** — Use Ctrl+F to search
2. **Open a GitHub issue** — Label it "docs" or "faq"
3. **Post in Discord** — Ask in #general or #support
4. **Email** — Community lead email

**Include:**
- The question
- A clear, helpful answer
- Use case or example (if relevant)
- Any code snippets or links

### Maintaining the FAQ

The FAQ is reviewed and updated:
- **Monthly:** Check support tickets for new patterns
- **Quarterly:** Review community discussions for trending questions
- **Yearly:** Major reorganization and archival

Last updated: [Date]

---

## Quick Reference

### Important Links

| Resource | Link |
|---|---|
| **Frontend (Mainnet)** | [URL] |
| **Frontend (Testnet)** | [URL] |
| **GitHub Repository** | [URL] |
| **Documentation** | [URL] |
| **Discord Community** | [URL] |
| **Security Contact** | security@example.com |
| **Support Email** | support@example.com |
| **Bug Reports** | [GitHub Issues] |

### Key Contract Addresses

| Network | Address |
|---|---|
| **Stellar Mainnet** | CABC... |
| **Stellar Testnet** | CTEST... |

### Common Error Codes

| Code | Error | Meaning |
|---|---|---|
| 1 | InvalidAmount | Amount <= 0 |
| 2 | UnlockTimeNotInFuture | Unlock time is in the past |
| 3 | NoDepositFound | Deposit doesn't exist |
| 4 | FundsStillLocked | Unlock time hasn't passed |
| 6 | LockDurationTooLong | Duration > 5 years |
| 7 | Unauthorized | Not authorized (not admin) |
| 9 | InvalidPenaltyBps | Penalty > 10,000 bps |
| 11 | LockDurationTooShort | Duration < 60 seconds |
| 12 | ContractPaused | Contract is paused for maintenance |
| 18 | RecipientNotWhitelisted | Recipient not on withdrawal whitelist |

---

## Related Documents

- [README.md](../README.md) — Technical overview
- [USER_ONBOARDING.md](./USER_ONBOARDING.md) — Getting started guide
- [SECURITY.md](../SECURITY.md) — Security policy
- [CONTRIBUTING.md](../CONTRIBUTING.md) — Development guidelines
