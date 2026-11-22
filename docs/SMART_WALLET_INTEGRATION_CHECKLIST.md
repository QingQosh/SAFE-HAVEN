# Smart Wallet Integration Checklist

## Overview

This checklist ensures your smart wallet integrates correctly and securely with SAFE-HAVEN vault contracts. Use it as a step-by-step guide during implementation and as a pre-deployment verification tool.

**Document Version**: 1.0  
**Last Updated**: 2024  
**Target Audience**: Wallet developers, DevOps teams, security auditors

---

## Pre-Integration: Planning Phase

### Authorization Model Review

- [ ] **Reviewed Soroban `require_auth()` documentation**
  - Understand how Soroban authorizes contract addresses
  - Confirm your wallet uses Soroban's native authorization framework
  
- [ ] **Identified wallet type**
  - [ ] Safe-style multisig wallet
  - [ ] Account abstraction (AA) wallet
  - [ ] EOA or hybrid wallet
  - [ ] Other (specify): _______________

- [ ] **Mapped wallet approval flow**
  - Document how your wallet validates and executes transactions
  - Identify where SAFE-HAVEN calls fit in the approval sequence
  
- [ ] **Confirmed token support**
  - Verify tokens accepted by SAFE-HAVEN work with your wallet
  - Check token approval patterns (if applicable)

### Architecture Decision Recording

- [ ] **Documented integration approach**
  - Record whether you're using: `deposit()`, `deposit_for()`, or `deposit_with_session_key()`
  - Explain the rationale for each choice
  
- [ ] **Design review completed**
  - Have security team review integration architecture
  - Document any deviations from standard patterns

---

## Core Integration: Wallet Configuration

### Smart Wallet Address Setup

- [ ] **Smart wallet deployed as Soroban contract**
  - Wallet exists as a contract address on the target network
  - NOT implemented as an EOA or externally owned account
  
- [ ] **Wallet address derivable and consistent**
  - Address generation is deterministic
  - Address remains consistent across deployments
  - Document address derivation method
  
- [ ] **Confirmed wallet as `depositor` in vault calls**
  - Use wallet contract address, NOT individual signer addresses
  - Use wallet contract address, NOT relayer addresses
  - Verified in production configuration files

### Authorization Configuration

- [ ] **Wallet implements Soroban `require_auth()` pattern**
  - Wallet correctly handles auth invocations from SAFE-HAVEN
  - Wallet authorization logic is executed before state mutations
  
- [ ] **Authorization logic executed before token transfer**
  - Wallet checks approval threshold/policy BEFORE calling vault
  - Token approval (if needed) is obtained BEFORE vault call
  
- [ ] **Authorization checks are immutable after deployment**
  - Security-critical authorization logic cannot be disabled
  - Admin functions cannot bypass authorization checks

### Signature Verification (Multisig Wallets)

- [ ] **Multisig threshold correctly implemented**
  - Minimum number of signers required is enforced
  - Threshold is configurable but immutable during a transaction
  
- [ ] **Signature aggregation implemented**
  - Contract correctly verifies all signatures
  - Duplicated signatures are rejected
  - Out-of-order signatures do not bypass verification
  
- [ ] **Signature verification uses safe libraries**
  - Cryptographic functions from established libraries (e.g., `soroban-auth`, Stellar SDK)
  - No custom cryptography implementations
  - Code reviewed by security team

### Replay Protection

- [ ] **Transaction nonce or sequence tracking implemented**
  - Each transaction has a unique identifier to prevent replays
  - Nonce is incremented after each successful operation
  
- [ ] **Nonce validation enforced before authorization**
  - Old/replayed transactions are rejected early
  - Nonce state is updated atomically with operation
  
- [ ] **Chain ID included in signed data (if applicable)**
  - Prevents cross-chain replay attacks
  - Current network is verified during authorization

---

## Integration Pattern: Direct Deposit

### Standard Deposit Flow

Use this when the wallet directly funds its own deposit.

- [ ] **Wallet calls `vault.deposit(depositor, token, amount, unlock_time, penalty_bps)`**
  - `depositor` is the wallet contract address
  - `token` is a valid Stellar asset contract address
  - `amount` is positive and wallet has sufficient balance
  - `unlock_time` is in the future (Unix timestamp)
  - `penalty_bps` is 0-10000 (0-100%)
  
- [ ] **Wallet authorizes the deposit call**
  - Authorization context includes this contract call
  - Soroban sees wallet address as `require_auth()` caller
  
- [ ] **Token transfer authorized by wallet**
  - If token requires approval, wallet initiates approval first
  - Token transfer amount matches vault call
  - Token transfer goes directly from wallet to vault
  
- [ ] **Deposit recorded under wallet address**
  - Vault records ownership as `depositor` (wallet address)
  - Transaction signer irrelevant; ownership is wallet address
  
- [ ] **Event monitoring by wallet address**
  - Monitor `deposit` events filtered by wallet `depositor`
  - Do NOT filter by transaction signer or relayer

### Implementation Validation

- [ ] **Test direct deposit on local network**
  - Deposit creates entry with correct ID (starts at 0)
  - Deposit amount matches vault state
  - Token transfer executed correctly
  
- [ ] **Test multiple deposits per wallet**
  - IDs increment correctly (0, 1, 2, ...)
  - Each deposit is independent
  - Withdrawing one does not affect others
  
- [ ] **Test authorization failure scenarios**
  - Unauthorized address cannot create deposit
  - Operation reverted without state mutation
  - No partial state updates

---

## Integration Pattern: Third-Party Funding (Deposit For)

### Relayer-Funded Deposit Flow

Use this when a relayer or payer funds the deposit for another address (beneficiary).

- [ ] **Payer calls `vault.deposit_for(payer, depositor, token, amount, unlock_time, penalty_bps)`**
  - `payer` is the relayer/funder address (wallet or EOA)
  - `depositor` is the beneficiary (wallet or EOA that owns the deposit)
  - `token` is a valid Stellar asset
  - `amount` must be funded by `payer`
  
- [ ] **Payer address is authorized by Soroban**
  - `require_auth()` validates payer is authorized to spend
  - Payer must authorize the vault call
  
- [ ] **Token transfer from payer to vault**
  - Payer has sufficient balance
  - Token transfer amount matches vault call
  - Token approval (if needed) is from payer
  
- [ ] **Deposit ownership is `depositor`, not payer**
  - Only `depositor` can withdraw the deposit
  - Payer has no further claims on the deposit
  - Multiple payers can fund the same beneficiary
  
- [ ] **Verify payer cannot withdraw beneficiary's deposit**
  - Payer tries to withdraw beneficiary's ID → authorization fails
  - Beneficiary withdraws successfully

### Implementation Validation

- [ ] **Test payer-funded workflow on local network**
  - Deposit ID created correctly
  - Ownership is beneficiary (depositor parameter)
  - Tokens transferred from payer account
  
- [ ] **Test payer cannot access beneficiary funds**
  - Payer authorized to pay, not to withdraw
  - Only beneficiary address can withdraw
  
- [ ] **Test multiple payers funding same beneficiary**
  - Each payer creates separate deposit
  - IDs are independent and sequential
  - Beneficiary can manage all deposits

### Security Considerations

- [ ] **Payer balance verified before transfer**
  - Transaction reverts if payer has insufficient balance
  - No partial deposits created
  
- [ ] **Depositor address cannot be zero address**
  - Invalid depositor addresses rejected
  - Prevents accidental fund locks
  
- [ ] **No reward to payer**
  - Payer receives no interest or vault proceeds
  - Payer investment is one-way transfer

---

## Integration Pattern: Session Keys

### Session Key Authorization Workflow

Use this for time-bounded delegated funding (gasless or relayed deposits).

#### Setup Phase

- [ ] **Wallet calls `vault.authorize_session_key(wallet, session_key, expires_at)`**
  - Called by wallet (requires `require_auth()` for wallet address)
  - `session_key` is the delegated address (can be EOA or contract)
  - `expires_at` is a future Unix timestamp
  - Expiry is explicit and immutable
  
- [ ] **Session key scope is single wallet**
  - Each session key is authorized for one wallet only
  - Same session key cannot be used with multiple wallets
  - Prevents cross-wallet delegation attacks

#### Usage Phase

- [ ] **Session key calls `vault.deposit_with_session_key(session_key, wallet, token, amount, unlock_time, penalty_bps)`**
  - `session_key` must match registered key (authorization checked)
  - `wallet` must match wallet session key is scoped to
  - Must be called before expiry timestamp
  
- [ ] **Expiry validation enforced before operation**
  - Check: `expires_at > env.ledger().timestamp()`
  - Expired keys rejected with `SessionKeyExpired` error
  - No state mutation if expired
  
- [ ] **Token transferred from session key, not wallet**
  - Session key must have sufficient balance
  - Wallet does NOT fund the deposit
  - Reduces load on wallet (gasless pattern possible)
  
- [ ] **Deposit ownership is wallet, not session key**
  - Vault records `depositor` as wallet address
  - Session key cannot withdraw this deposit
  - Only wallet can revoke or manage session keys

#### Revocation Phase

- [ ] **Wallet calls `vault.revoke_session_key(wallet, session_key)` when needed**
  - Only wallet that authorized key can revoke
  - Revocation is immediate
  
- [ ] **Revoked keys cannot be reused**
  - After revocation, key is permanently disabled for this wallet
  - Cannot re-register revoked key (if business logic prevents it)
  
- [ ] **Revocation does not affect existing deposits**
  - Deposits created by session key remain owned by wallet
  - Wallet can manage deposits normally
  
- [ ] **Session key expiry acts as automatic revocation**
  - Expired keys are automatically invalid
  - No manual revocation needed if you wait for expiry
  - Explicit revocation recommended for security

### Session Key Expiry Management

- [ ] **Expiry timestamp is explicit and documented**
  - Do NOT use vague durations like "1 hour from now"
  - Calculate: `current_timestamp + session_duration`
  - Document session duration in configuration
  
- [ ] **Multiple session keys per wallet allowed**
  - Wallet can authorize multiple concurrent session keys
  - Each key has independent expiry
  - Useful for: different relayers, seasonal delegation, key rotation
  
- [ ] **Expiry is checked at operation time**
  - Session key authorized at time T with expiry T+3600
  - If current time > expiry, operation fails
  - No grace period
  
- [ ] **Clock skew tolerance considered**
  - Ledger timestamp is authoritative (not wall-clock)
  - Stellar network closure time ≈ 5 seconds
  - Set expiry with buffer: `now + 3600 + buffer`
  - Document buffer (e.g., 60 seconds) in config

### Ledger Time Integration

- [ ] **Ledger timestamp used for all time checks**
  - NOT wall-clock time
  - NOT block number
  - Use `env.ledger().timestamp()` in contracts
  
- [ ] **Expiry semantics well-defined**
  - `expires_at > ledger_timestamp()` means NOT YET EXPIRED
  - `expires_at <= ledger_timestamp()` means EXPIRED
  - Document this explicitly in code comments

### Implementation Validation

- [ ] **Test session key authorization on local network**
  - Key authorized successfully
  - Authorized key can create deposit
  - Deposit recorded under wallet
  
- [ ] **Test session key expiry enforcement**
  - Valid key (before expiry) works
  - Expired key (after expiry) fails
  - Test boundary: `expires_at == current_timestamp`
  
- [ ] **Test session key revocation**
  - After revocation, key cannot create deposits
  - Deposits already created remain valid
  - Can re-authorize same key with new expiry
  
- [ ] **Test session key scope**
  - Key authorized for wallet A works with wallet A
  - Same key with wallet B fails (if scope enforced)
  - Prevents cross-wallet attacks

### Security Best Practices: Session Keys

- [ ] **Every session key has explicit expiry**
  - NEVER use unbounded session keys
  - Expiry must be configurable per use case
  
- [ ] **Session key is NOT the wallet private key**
  - Use distinct keypairs for delegation
  - Session key compromise does not expose wallet
  - Reduces blast radius of key theft
  
- [ ] **Session key has limited balance**
  - Fund session key only with needed amount
  - Wallet retains control of primary funds
  - Session key cannot spend wallet's entire balance
  
- [ ] **Revoke session keys when no longer needed**
  - Revoke immediately after intended session ends
  - Revoke before expiry if key compromised
  - Implement revocation monitoring/alerts
  
- [ ] **Monitor session key usage**
  - Track which keys created which deposits
  - Alert on unexpected session key usage
  - Audit session key lifecycle events
  
- [ ] **Session key credentials stored securely**
  - Use secure key management (e.g., AWS KMS, HashiCorp Vault)
  - Rotate keys periodically
  - Never commit session keys to version control

---

## Integration Pattern: Multisig Wallets

### Multisig Wallet Requirements

- [ ] **Wallet implements Safe-style or similar multisig pattern**
  - Multiple signers required for approvals
  - Threshold of signatures enforced
  - Signature aggregation logic implemented
  
- [ ] **Multisig wallet is deployed as Soroban contract**
  - Wallet address is a contract address
  - NOT deployed as individual signer EOAs
  
- [ ] **Approval threshold enforced before vault call**
  - Multisig checks approval count before calling SAFE-HAVEN
  - Approved transactions produce wallet-signed data
  - Wallet calls vault contract after threshold met

### Multisig Authorization Flow

- [ ] **Individual signers do NOT directly authorize vault call**
  - Signers authorize the multisig wallet operation
  - Multisig wallet (after approval) authorizes vault call
  - Vault sees multisig contract address as `require_auth()` caller
  
- [ ] **SAFE-HAVEN vault does NOT inspect individual signers**
  - Vault trusts multisig contract authorization
  - Vault does NOT validate signature count
  - Vault does NOT reverse-engineer signer identities
  
- [ ] **Deposit is recorded under multisig wallet address**
  - `depositor` is multisig contract address
  - Individual signers have no direct ownership claim
  - Signers can only access via multisig authorization

### Multisig Implementation Validation

- [ ] **Test multisig wallet deposit on local network**
  - Configure M-of-N signers (e.g., 2-of-3)
  - Gather M signatures
  - Multisig wallet calls vault
  - Deposit recorded under multisig address
  
- [ ] **Test multisig rejection below threshold**
  - Configure 2-of-3 threshold
  - Provide only 1 signature
  - Multisig rejects operation
  - Vault call never reached
  
- [ ] **Test individual signer cannot bypass multisig**
  - Signer A tries to call vault directly (not via multisig)
  - Authorization check fails
  - Signer cannot withdraw multisig-owned deposit
  
- [ ] **Test multisig withdrawal**
  - Multisig-owned deposit created
  - Multisig provides approval for withdrawal
  - Withdrawal succeeds
  
- [ ] **Test multisig cannot withdraw single-signer deposit**
  - Single-signer address owns deposit (not multisig)
  - Multisig tries to withdraw
  - Authorization fails

### Multisig Security Considerations

- [ ] **Signature order does not matter**
  - Signatures can be sorted or aggregated in any order
  - Signature verification is order-independent
  
- [ ] **Duplicate signatures rejected**
  - Same signature cannot be used twice
  - Prevents threshold bypass
  
- [ ] **Signature is over the correct operation**
  - Data includes: contract address, method, parameters
  - Signature binds to this specific operation
  - Cannot be reused for different operations
  
- [ ] **Signature algorithm uses established standards**
  - Stellar uses Ed25519
  - Signature verification uses standard Stellar SDK
  - No custom cryptography

---

## Integration Pattern: Account Abstraction (AA) Wallets

### AA Wallet Architecture

- [ ] **Wallet deployed as Soroban contract**
  - AA wallet is a smart contract (not EOA)
  - Custom authorization logic implemented in wallet contract
  
- [ ] **Entry point contract for relay operations (optional)**
  - Entry point can relay calls to AA wallet
  - Entry point calls AA wallet, which authorizes vault call
  - Enables gasless patterns
  
- [ ] **Authorization logic customizable per wallet**
  - SAFE-HAVEN does NOT enforce AA wallet structure
  - Wallet implements its own approval/verification
  - Vault trusts Soroban `require_auth()` mechanism

### AA Wallet Authorization Flow

- [ ] **AA wallet authorizes vault call via Soroban auth**
  - Wallet implements custom authorization logic
  - Authorization is checked by Soroban (not SAFE-HAVEN)
  - Vault sees wallet contract address as caller
  
- [ ] **Entry point (if used) calls AA wallet first**
  - Entry point receives relayed transaction
  - Entry point calls AA wallet to validate
  - AA wallet authorizes entry point call
  - Entry point then calls vault
  - Vault sees entry point address, which is authorized by AA wallet
  
- [ ] **Deposit is recorded under AA wallet address**
  - If AA wallet directly calls vault: `depositor` is AA wallet
  - If entry point relays: `depositor` is still AA wallet (not entry point)
  - Wallet maintains full control and ownership

### AA Wallet Implementation Validation

- [ ] **Test AA wallet direct deposit**
  - AA wallet calls vault directly
  - Deposit recorded under AA wallet address
  - AA wallet can withdraw own deposit
  
- [ ] **Test AA wallet with entry point relay**
  - Entry point receives relayed transaction
  - Entry point calls AA wallet for authorization
  - AA wallet authorizes entry point call
  - Entry point calls vault
  - Deposit recorded under AA wallet address
  
- [ ] **Test AA wallet with session key delegation**
  - AA wallet authorizes session key with `authorize_session_key()`
  - Session key calls `deposit_with_session_key()`
  - Deposit recorded under AA wallet
  - AA wallet can revoke session key anytime
  
- [ ] **Test multi-operation sequence**
  - Create multiple deposits
  - Transfer tokens between deposits (if applicable)
  - AA wallet maintains authorization throughout

### AA Wallet Security Considerations

- [ ] **Entry point does NOT own deposits**
  - Depositor is AA wallet, NOT entry point
  - Entry point is purely relaying mechanism
  - Wallet retains full control
  
- [ ] **Entry point cannot be substituted mid-operation**
  - AA wallet verifies entry point identity
  - Only authorized entry point can relay
  - Different entry points require re-authorization
  
- [ ] **AA wallet maintains recovery procedures**
  - Wallet implements key recovery (if needed)
  - Recovery does NOT affect vault deposits
  - Recovery changes wallet authorization, not depositor
  
- [ ] **AA wallet can implement custom spending limits**
  - Wallet can limit per-transaction amounts
  - Wallet can limit daily/weekly aggregates
  - SAFE-HAVEN respects wallet's authorization decision

---

## Security Audit Checklist

### Authorization & Access Control

- [ ] **`require_auth()` called before state mutation**
  - Authorization check is first operation in state-mutating functions
  - No state changes before auth validation
  
- [ ] **Authorization scope is correct**
  - Wallet address is authorized for wallet-owned operations
  - Session key is authorized for delegated operations
  - Payer is authorized for `deposit_for()` operations
  
- [ ] **Unauthorized addresses cannot execute operations**
  - Non-wallet address cannot create/withdraw wallet deposits
  - Non-session-key address cannot use session key
  - Non-payer address cannot fund `deposit_for()`
  
- [ ] **Authorization is idempotent**
  - Same operation succeeds if authorized, fails if not
  - No state corruption from repeated auth checks
  
- [ ] **Authorization cannot be bypassed**
  - No alternate code paths skip auth
  - No admin override of authorization
  - No emergency mode that disables auth

### Session Key Security

- [ ] **Session key expiry enforced every use**
  - Expiry checked before ANY operation with session key
  - No state mutation if expired
  - Expiry value is immutable after authorization
  
- [ ] **Session key scope is enforced**
  - Key authorized for wallet A cannot be used with wallet B
  - Wallet information is part of the authorization check
  - Cross-wallet usage fails fast
  
- [ ] **Session key revocation is immediate**
  - After revocation, key cannot create any operations
  - Revocation is effective immediately (no grace period)
  - Revoked key cannot be re-used (if business logic prevents it)
  
- [ ] **Session key does NOT inherit wallet privileges**
  - Session key can create deposits but cannot withdraw them
  - Session key cannot transfer wallet funds outside vault
  - Wallet remains authorization point for sensitive operations
  
- [ ] **Multiple session keys do NOT interfere**
  - Each key is tracked independently
  - Revocation of key A does not affect key B
  - Expiry of key A does not affect key B
  
- [ ] **Session key cannot spend wallet's balance**
  - Only session key's own balance is used
  - Wallet funds remain untouched by session key
  - Wallet can be empty while session key operates

### Input Validation

- [ ] **Token address is validated**
  - Token contract exists on the network
  - Token address is not zero address
  - Token implements standard interface (transfer, mint, etc.)
  
- [ ] **Amount is validated**
  - Amount > 0 (no zero deposits)
  - Amount does not overflow (checked against max uint256)
  - Sender has sufficient balance
  
- [ ] **Unlock time is validated**
  - Unlock time is in the future
  - Unlock time is reasonable (e.g., not 1000 years away)
  - Unlock time is before platform end-of-life date
  
- [ ] **Penalty basis points are validated**
  - Penalty is 0-10000 (0-100%)
  - Penalty is not negative
  - Penalty is reasonable for vault use case
  
- [ ] **Session key address is validated**
  - Session key is not zero address
  - Session key is not same as wallet address
  - Session key is not same as another registered key
  
- [ ] **Depositor address is validated**
  - Depositor is not zero address
  - Depositor address format is valid

### State Management

- [ ] **Deposit IDs are assigned sequentially**
  - First deposit for address is ID 0
  - IDs increment by 1
  - IDs never reused even after withdrawal
  
- [ ] **Deposit ownership is immutable**
  - Once deposit created under address, owner cannot change
  - Cannot transfer deposit to different owner
  - Cannot delegate ownership to another address
  
- [ ] **Deposits are isolated**
  - Withdrawing deposit X does not affect deposit Y
  - Canceling deposit X does not affect deposit Y
  - State mutations are atomic per operation
  
- [ ] **No duplicate session keys per wallet**
  - Same session key cannot be authorized twice for same wallet
  - Re-authorization of same key is rejected or replaces old entry
  - Prevents confusion and tracking bugs

### Error Handling

- [ ] **Error codes are specific and actionable**
  - `NotAuthorized` = authorization failed (not just missing auth)
  - `SessionKeyExpired` = key expired (not generic error)
  - `FundsStillLocked` = unlock time not reached
  - `SessionKeyNotAuthorized` = key not registered
  
- [ ] **Errors include minimal information**
  - Error message does NOT leak sensitive state
  - Error message does NOT leak timing information
  - Error message is sufficient for debugging

### Replay & Nonce Protection

- [ ] **Wallet implements replay protection**
  - Nonce or sequence tracking prevents old transactions from replaying
  - Nonce is checked before wallet authorization
  
- [ ] **Cross-chain replay is prevented**
  - Chain ID or network identifier included in signed data
  - Different networks require different signatures
  
- [ ] **Session key replays are prevented**
  - Session key operations include timestamp
  - Old operations cannot be replayed on new network state
  - Nonce incremented per operation

### Cryptographic Security

- [ ] **Signatures use Stellar Ed25519 standard**
  - All signatures are Ed25519 (Soroban standard)
  - No custom signature algorithms
  - Signature verification uses established libraries
  
- [ ] **Signature verification is constant-time**
  - Signature check does NOT leak timing information
  - Verification time does NOT depend on signature validity
  
- [ ] **Private key handling is secure**
  - Private keys never logged or exposed
  - Private keys stored in secure key management system
  - Key rotation periodic and documented

### Testing Coverage

- [ ] **Authorization tests exist**
  - Test wallet authorizes operations
  - Test non-wallet cannot authorize operations
  - Test authorization failure reverts state
  
- [ ] **Session key tests exist**
  - Test key authorization and usage
  - Test key expiry enforcement
  - Test key revocation
  - Test scope (wallet-specific)
  
- [ ] **Multisig tests exist**
  - Test threshold enforcement
  - Test signature aggregation
  - Test duplicate signature rejection
  - Test invalid signature rejection
  
- [ ] **Error case tests exist**
  - Test each error condition
  - Test error specificity
  - Test no state mutation on error
  
- [ ] **Edge case tests exist**
  - Test boundary conditions (expiry at exact time)
  - Test maximum values (large amounts, max penalty)
  - Test zero values (zero amount, zero penalty)
  
- [ ] **Integration tests exist**
  - Test full workflow from user perspective
  - Test multiple wallets interacting
  - Test session key lifecycle

---

## Pre-Deployment Verification

### Code Review Checklist

- [ ] **Authorization logic reviewed by security team**
  - Code review completed with sign-off
  - No outstanding security concerns
  - Authorization checks are in correct order
  
- [ ] **Signature verification reviewed by cryptography expert**
  - Signature scheme is appropriate
  - Verification logic is correct
  - No timing side-channels
  
- [ ] **State management reviewed**
  - Deposits are correctly stored and retrieved
  - State mutations are atomic
  - No race conditions
  
- [ ] **Error handling reviewed**
  - All error paths are correct
  - No unhandled exceptions
  - Error messages are appropriate

### Static Analysis

- [ ] **Solidity/Rust linter passes**
  - No compiler warnings
  - Code style is consistent
  - All formatting rules applied
  
- [ ] **Security static analysis tool run**
  - Tool: _________________ (e.g., Slither, Mythril)
  - No high/critical issues
  - Medium issues reviewed and documented
  
- [ ] **Dependency audit completed**
  - No known vulnerabilities in dependencies
  - All dependencies up-to-date
  - Dependency changes documented

### Testnet Deployment

- [ ] **Deployed to public testnet**
  - Network: ________________________ (e.g., Stellar Testnet)
  - Deployment date: _________________
  - Wallet address: ___________________
  
- [ ] **Testnet tests passed**
  - All integration tests pass on testnet
  - No network-specific issues
  - Performance acceptable
  
- [ ] **Testnet monitoring active**
  - Deployed for min. 1 week
  - No critical issues found
  - User feedback reviewed (if applicable)

### Documentation Readiness

- [ ] **Integration guide written**
  - Step-by-step instructions for developers
  - Example code provided
  - Common pitfalls documented
  
- [ ] **Security guide written**
  - Authorization model explained
  - Session key best practices documented
  - Revocation procedures explained
  
- [ ] **API documentation complete**
  - All functions documented
  - Parameters documented
  - Return values documented
  - Error codes documented
  
- [ ] **FAQ updated**
  - Common integration questions answered
  - Troubleshooting section provided
  - Links to security resources

### Operational Readiness

- [ ] **Monitoring configured**
  - Deposit events logged
  - Authorization failures logged
  - Session key operations logged
  - Alerts configured for failures
  
- [ ] **Key management procedures documented**
  - Session key generation procedure
  - Key storage location
  - Key rotation schedule
  - Key revocation procedure
  
- [ ] **Incident response plan drafted**
  - Process for revoking compromised keys
  - Process for suspending wallet (if applicable)
  - Escalation contacts
  - Communication templates
  
- [ ] **Runbook prepared**
  - Step-by-step deployment instructions
  - Rollback procedures
  - Verification steps
  - Contact list for escalation

---

## Post-Deployment: Operations

### Day-1 Operations

- [ ] **Deployed to mainnet**
  - Network: ________________________
  - Deployment date: _________________
  - Wallet address: ___________________
  - Deployment confirmed by 2 team members
  
- [ ] **Monitoring verified**
  - Event logs being collected
  - Alerts configured and tested
  - Dashboard accessible
  
- [ ] **On-call contact established**
  - Primary contact: ___________________
  - Backup contact: ____________________
  - Escalation chain documented

### Ongoing Monitoring

- [ ] **Daily monitoring (first week)**
  - Review logs for errors
  - Confirm expected transaction patterns
  - Check for anomalies
  
- [ ] **Weekly monitoring (first month)**
  - Review key rotation schedule
  - Check for unused session keys (revoke if needed)
  - Review user feedback
  
- [ ] **Monthly monitoring**
  - Aggregate statistics (deposits, withdrawals, etc.)
  - Review security logs for unauthorized attempts
  - Check for deprecation notices or upstream changes
  
- [ ] **Quarterly review**
  - Full security audit of logs
  - Update incident response plan
  - Plan for upgrades or improvements

### Maintenance

- [ ] **Security patches applied promptly**
  - Upstream security updates reviewed
  - Patches tested on testnet
  - Patches deployed to mainnet with change control
  
- [ ] **Dependencies updated**
  - Quarterly dependency audit
  - Updates tested before deployment
  - Changelog reviewed for breaking changes
  
- [ ] **Key rotation on schedule**
  - Session keys rotated per policy
  - Old keys revoked
  - New keys authorized and deployed
  
- [ ] **Procedures updated as needed**
  - Runbooks kept current
  - Incident response plan updated
  - Documentation updated for new features

---

## Sign-Off

### Development Team

- [ ] Developer Name: _________________ Date: _________
- [ ] Developer Name: _________________ Date: _________

### Security Review

- [ ] Security Reviewer: _______________ Date: _________
- [ ] Security Reviewer: _______________ Date: _________

### DevOps / Deployment

- [ ] DevOps Engineer: ________________ Date: _________
- [ ] Operations Contact: ______________ Date: _________

### Management Approval

- [ ] Manager: ______________________ Date: _________
- [ ] Project Lead: __________________ Date: _________

---

## Appendix A: Common Integration Patterns

### Pattern 1: Simple Multisig Wallet

```
User/Signer → Multisig Contract (M-of-N approval)
           ↓
        [Signatures collected and verified]
           ↓
    Multisig calls vault.deposit(wallet_address, token, amount, unlock_time, penalty)
           ↓
    Soroban require_auth() validates multisig contract address
           ↓
    Deposit recorded under multisig contract address
           ↓
    Only multisig can withdraw (via subsequent M-of-N approval)
```

**When to use**: Traditional multisig wallets, organizations, joint accounts

### Pattern 2: Relayer-Funded Deposit

```
User (Beneficiary) → Relayer (Payer)
                  ↓
          Relayer has tokens
                  ↓
    Relayer calls vault.deposit_for(relayer, beneficiary, token, amount, unlock_time, penalty)
                  ↓
    Soroban require_auth() validates relayer address
                  ↓
    Tokens transferred from relayer to vault
                  ↓
    Deposit recorded under beneficiary address
                  ↓
    Only beneficiary can withdraw
```

**When to use**: Onboarding flows, gifting, subsidy programs

### Pattern 3: Session Key Gasless Deposits

```
Wallet → Wallet authorizes Session Key
      ↓
   Wallet calls vault.authorize_session_key(wallet, session_key, expires_at)
      ↓
   [Session key active for bounded period]
      ↓
   Session Key calls vault.deposit_with_session_key(session_key, wallet, token, amount, unlock_time, penalty)
      ↓
   Soroban checks: session_key authorized for wallet? Yes. Expired? No.
      ↓
   Tokens transferred from session key to vault
      ↓
   Deposit recorded under wallet address
      ↓
   Wallet can later revoke: vault.revoke_session_key(wallet, session_key)
```

**When to use**: Gasless deposits, relayed operations, delegated funding

### Pattern 4: Account Abstraction with Entry Point

```
User → AA Entry Point → AA Wallet → AA Wallet authorizes entry point call
    ↓                  ↓              ↓
    └──────────────────┴──────────────┘
                       ↓
          Entry point calls vault.deposit(aa_wallet_address, token, amount, unlock_time, penalty)
                       ↓
          Soroban require_auth() validates AA wallet address (entry point is authorized by AA)
                       ↓
          Tokens transferred from AA wallet to vault
                       ↓
          Deposit recorded under AA wallet address
                       ↓
          AA wallet maintains full control and can authorize future operations
```

**When to use**: Account abstraction architectures, custom authorization logic, advanced wallets

---

## Appendix B: Security Incident Response

### Compromised Session Key

**Symptoms**: Unauthorized deposits appearing under wallet

**Response**:
1. [ ] Immediately revoke the compromised session key
2. [ ] Review all deposits created by the key (identify unauthorized ones)
3. [ ] If pending unlock: mark deposit as locked pending investigation
4. [ ] If unlockable: consider early withdrawal with penalty
5. [ ] Rotate affected wallet's session keys
6. [ ] Investigate root cause (key leak, unauthorized delegation, etc.)
7. [ ] Notify affected users
8. [ ] Update key management procedures

### Unauthorized Withdrawal Attempt

**Symptoms**: Failed authorization logs from unauthorized address attempting withdrawal

**Response**:
1. [ ] Verify no unauthorized withdrawal occurred (check deposit state)
2. [ ] If withdrawal succeeded: investigate authorization bypass (potential critical bug)
3. [ ] If withdrawal failed: log and monitor (normal behavior)
4. [ ] If attempted by known wallet address: contact wallet owner
5. [ ] Document incident and pattern
6. [ ] Review code for authorization bypass vectors

### Authorization Bypass Detection

**Symptoms**: State mutation without proper authorization

**Response**:
1. [ ] **PAUSE deployment immediately** (if possible)
2. [ ] Investigate authorization logic
3. [ ] Check logs for other exploits
4. [ ] Identify affected deposits/users
5. [ ] Notify security team and leadership
6. [ ] Deploy patch/hotfix
7. [ ] Conduct post-mortem
8. [ ] Update testing to prevent recurrence

---

## Appendix C: Useful Resources

### Soroban Documentation
- [Soroban Authorization](https://developers.stellar.org/docs/build/smart-contracts/authorization)
- [Soroban Core Concepts](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar Network Guide](https://developers.stellar.org/docs/learn/fundamentals)

### Smart Wallet Resources
- [Safe Smart Account](https://safe.global/)
- [EIP-4337 Account Abstraction](https://eips.ethereum.org/EIPS/eip-4337) (for reference)
- [SAFE-HAVEN Smart Wallet Integration Guide](./SMART_WALLETS.md)
- [SAFE-HAVEN Smart Wallet Test Suite](./SMART_WALLET_TESTING.md)

### Security Resources
- [OWASP Smart Contract Security](https://cheatsheetseries.owasp.org/cheatsheets/Cryptocurrency_Developers_Cheatsheet.html)
- [Stellar Security Best Practices](https://developers.stellar.org/docs/learn/fundamentals/security)

### Testing Resources
- [Soroban Test Framework](https://developers.stellar.org/docs/build/smart-contracts/testing)
- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2024 | SAFE-HAVEN Team | Initial checklist document |
| | | | Includes authorization, session keys, multisig, AA patterns |
| | | | Comprehensive security audit section |
| | | | Pre-deployment and operations guidance |

---

## Questions or Feedback?

For questions about this checklist:
1. Review the [SMART_WALLETS.md](./SMART_WALLETS.md) integration guide
2. Check the [SMART_WALLET_TESTING.md](./SMART_WALLET_TESTING.md) test examples
3. Open an issue on the SAFE-HAVEN repository
4. Contact the SAFE-HAVEN security team
