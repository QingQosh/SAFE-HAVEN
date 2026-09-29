# Legal Documentation

> **IMPORTANT:** This document is a template drafted for the SAFE-HAVEN project and has
> **not** been reviewed by a qualified legal professional. It does not constitute legal
> advice. Before deploying to production or making this document publicly accessible,
> engage qualified legal counsel in every jurisdiction where the software is operated
> or used.

---

## Table of Contents

1. [Terms of Service](#1-terms-of-service)
2. [Privacy Policy](#2-privacy-policy)
3. [Disclaimers](#3-disclaimers)
4. [Regulatory Considerations](#4-regulatory-considerations)
5. [Data Retention Policy](#5-data-retention-policy)
6. [Customer Data Handling](#6-customer-data-handling)
7. [GDPR Compliance](#7-gdpr-compliance)
8. [Audit Trail](#8-audit-trail)

---

## 1. Terms of Service

**Effective date:** [INSERT DATE BEFORE PUBLISHING]
**Last revised:** [INSERT DATE BEFORE PUBLISHING]

### 1.1 Acceptance

By accessing or using the SAFE-HAVEN smart contract interface, website, or any
associated tooling (collectively, the "Service"), you agree to be bound by these
Terms of Service ("Terms"). If you do not agree, do not use the Service.

### 1.2 Eligibility

You must be:

- At least 18 years of age, or the age of legal majority in your jurisdiction
  (whichever is higher).
- Legally permitted to use decentralized financial applications in your jurisdiction.
- Not located in, or ordinarily resident in, a jurisdiction subject to comprehensive
  trade sanctions, including but not limited to jurisdictions listed on sanctions
  lists maintained by OFAC, the EU, or the UN.

By using the Service you represent and warrant that all of the above are true.

### 1.3 Description of Service

SAFE-HAVEN is a non-custodial, decentralized token-locking vault deployed as a
Soroban smart contract on the Stellar blockchain. The Service allows users to:

- Lock Stellar-compatible tokens for a defined period.
- Withdraw tokens after the lock period expires.
- Cancel a deposit early subject to a user-defined penalty.
- Register as a staker and earn proportional rewards from early-exit penalties.

The Service is provided on an "as-is" and "as-available" basis. The operators do not
take custody of your tokens at any time. Tokens are held exclusively by the deployed
smart contract on the Stellar blockchain.

### 1.4 User Responsibilities

You are solely responsible for:

- Maintaining the security of your private keys, seed phrases, and signing devices.
- Verifying the contract ID and network before signing any transaction.
- Understanding the lock period, penalty basis points, and unlock conditions you set
  at deposit time. These parameters are immutable after deposit.
- Ensuring that your use of the Service complies with applicable laws and regulations
  in your jurisdiction.
- Paying all applicable transaction fees (Stellar network fees, Soroban resource fees).

### 1.5 Prohibited Uses

You may not use the Service to:

- Violate any applicable law or regulation.
- Launder money, finance terrorism, or evade sanctions.
- Circumvent any security measure or access control.
- Reverse-engineer, decompile, or extract proprietary components of the Service.
- Conduct any activity that would impose an unreasonable load on the Stellar network
  or the Service's infrastructure.

### 1.6 No Warranties

THE SERVICE IS PROVIDED "AS IS" AND "AS AVAILABLE" WITHOUT WARRANTIES OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE, TITLE, AND NON-INFRINGEMENT. THE OPERATORS DO NOT
WARRANT THAT THE SERVICE WILL BE UNINTERRUPTED, ERROR-FREE, OR FREE FROM HARMFUL
COMPONENTS.

SMART CONTRACTS MAY CONTAIN BUGS. BLOCKCHAIN TRANSACTIONS ARE IRREVERSIBLE. YOU
ASSUME ALL RISK ASSOCIATED WITH YOUR USE OF THE SERVICE.

### 1.7 Limitation of Liability

TO THE MAXIMUM EXTENT PERMITTED BY LAW, THE OPERATORS AND CONTRIBUTORS SHALL NOT BE
LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, CONSEQUENTIAL, OR PUNITIVE DAMAGES,
OR ANY LOSS OF PROFITS, REVENUE, DATA, OR TOKENS, ARISING OUT OF OR IN CONNECTION
WITH YOUR USE OF THE SERVICE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGES.

IN NO EVENT SHALL THE AGGREGATE LIABILITY OF THE OPERATORS EXCEED THE GREATER OF
(A) ONE HUNDRED USD ($100) OR (B) THE AMOUNT OF FEES YOU PAID TO THE SERVICE IN
THE TWELVE MONTHS PRECEDING THE CLAIM.

### 1.8 Indemnification

You agree to indemnify, defend, and hold harmless the operators, contributors, and
their respective affiliates from any claims, damages, losses, liabilities, and
expenses (including attorneys' fees) arising out of your use of the Service or your
violation of these Terms.

### 1.9 Modifications

The operators reserve the right to modify these Terms at any time. Material changes
will be announced via the project's GitHub repository and, where feasible, the
frontend UI. Continued use of the Service after changes take effect constitutes
acceptance of the revised Terms.

### 1.10 Governing Law and Jurisdiction

[INSERT GOVERNING LAW JURISDICTION — requires legal review]

### 1.11 Dispute Resolution

[INSERT ARBITRATION OR LITIGATION CLAUSE — requires legal review]

### 1.12 Contact

Legal inquiries: [INSERT LEGAL CONTACT EMAIL]
Security issues: security@example.com *(update before production)*

---

## 2. Privacy Policy

**Effective date:** [INSERT DATE BEFORE PUBLISHING]
**Last revised:** [INSERT DATE BEFORE PUBLISHING]

### 2.1 Scope

This Privacy Policy applies to data collected through the SAFE-HAVEN frontend
application hosted at [INSERT URL] and any associated infrastructure operated by
the project.

It does not apply to data written to the Stellar blockchain, which is immutable,
public, and not controlled by the operators.

### 2.2 Information We Collect

**On-chain data (public, permanent):**
- Stellar public keys (addresses) submitted in contract transactions.
- Token amounts, lock durations, and penalty parameters set at deposit time.
- All contract events emitted during deposit, withdrawal, and admin operations.

This data is stored permanently on the Stellar blockchain and cannot be deleted by
anyone, including the operators.

**Off-chain data collected by the frontend:**
- Browser type, OS, and viewport size (via server access logs or analytics, if enabled).
- IP address (by inference from network requests to the RPC endpoint).
- Wallet connection events (via Freighter wallet API — no private keys or seeds are
  accessible to the frontend).
- Frontend error logs (if a crash-reporting service is configured).

**We do not collect:**
- Private keys, seed phrases, or wallet passwords.
- Personally identifiable information (name, email, phone) unless voluntarily provided
  through a support channel.
- Payment card information.

### 2.3 How We Use Information

Off-chain data is used solely for:

- Diagnosing errors and improving the reliability of the frontend.
- Understanding aggregate usage patterns to inform feature development.
- Complying with applicable legal obligations.

We do not sell, rent, or share personal data with third parties for advertising
purposes.

### 2.4 Cookies and Local Storage

The frontend may use browser local storage to persist:

- Wallet connection state (connected/disconnected flag, last-used network).
- User interface preferences (selected network, dismissed banners).
- Cached contract data to reduce RPC calls.

No tracking cookies or cross-site advertising identifiers are used.

If analytics are integrated in the future, this policy will be updated and a
cookie consent banner will be displayed in jurisdictions that require one.

### 2.5 Third-Party Services

The frontend communicates with:

- **Stellar RPC endpoint** (`VITE_RPC_URL`): required to query and submit contract
  transactions. Your IP address is visible to the RPC operator.
- **Stellar Horizon API** (`VITE_HORIZON_URL`): used for account balance queries.
- **Freighter wallet** (browser extension): handles transaction signing. The
  extension's own privacy policy governs data handled within the extension.
- **On-ramp provider** (if configured via `VITE_RAMP_*`): governed by that
  provider's own terms and privacy policy.

### 2.6 Data Retention — Off-Chain

| Data Type | Retention Period | Basis |
|---|---|---|
| Server access logs | 90 days | Operational necessity |
| Frontend error logs | 30 days | Debugging |
| Analytics aggregates | 24 months | Product improvement |
| Support communications | 3 years | Legal / dispute resolution |

### 2.7 Your Rights

Depending on your jurisdiction, you may have the right to:

- Access the personal data we hold about you.
- Correct inaccurate data.
- Request deletion of data (note: on-chain data cannot be deleted).
- Restrict or object to certain processing.
- Data portability.

To exercise these rights, contact: [INSERT PRIVACY CONTACT EMAIL]

We will respond within 30 days (or the period required by applicable law).

### 2.8 Children

The Service is not intended for individuals under 18. We do not knowingly collect
data from minors. If you believe we have inadvertently collected data from a minor,
contact us immediately.

### 2.9 Changes to This Policy

Material changes will be published on the project website and GitHub repository
with at least 14 days' notice before taking effect.

---

## 3. Disclaimers

### 3.1 Not Financial Advice

Nothing in this project, its documentation, its frontend, or any communication from
its contributors constitutes financial, investment, legal, or tax advice. The
SAFE-HAVEN vault is a technical tool. You are solely responsible for all financial
decisions you make using it, including the decision to lock tokens, the duration of
the lock, and the penalty parameters you select.

**You should consult a qualified financial and legal advisor before using this Service
with funds of material value.**

### 3.2 No Guarantee of Returns

The staker rewards mechanism distributes early-exit penalties to registered stakers.
The operators make no representation or guarantee that:

- Any rewards will be available at any given time.
- The rewards pool will contain a non-zero balance.
- Staker rewards will meet any particular yield or rate of return.
- The protocol will continue to operate indefinitely.

Reward amounts depend entirely on the behaviour of other users (early exits) and the
total stake registered.

### 3.3 Smart Contract Risk

Soroban smart contracts are software programs that execute on a decentralized network.
They may contain bugs, vulnerabilities, or design flaws that are not discovered until
after deployment. Because the contract is **immutable**, a critical bug cannot be
patched in place — mitigation requires deploying a new contract and coordinating user
migration.

The operators have taken reasonable steps to reduce risk (formal testing, security
review, CI checks, auditability of the open-source code), but no software is
provably free of all defects.

**You should not lock funds you cannot afford to lose.**

### 3.4 Blockchain Network Risk

The Stellar network is operated by an independent set of validators not controlled by
this project. Network outages, consensus failures, protocol upgrades, or validator
behaviour changes are outside the operators' control and may affect the availability
of the Service.

### 3.5 Regulatory Risk

The legal status of decentralized finance protocols, token locking, and staking
reward mechanisms varies by jurisdiction and is evolving. You are responsible for
determining whether your use of the Service complies with applicable laws in your
jurisdiction. The operators make no representation as to the regulatory status of
the Service in any jurisdiction.

### 3.6 Irreversibility of Blockchain Transactions

Transactions submitted to the Stellar blockchain are final and cannot be reversed or
cancelled. Tokens locked in the SAFE-HAVEN contract are subject to the lock duration
and penalty parameters set at deposit time. The operators cannot unlock, refund, or
reverse any on-chain transaction.

---

## 4. Regulatory Considerations

> This section identifies regulatory areas relevant to SAFE-HAVEN and the questions
> that legal counsel should address before production launch. It is not a legal
> opinion.

### 4.1 Applicability Criteria

SAFE-HAVEN provides non-custodial token locking with an optional staking rewards
mechanism. Regulatory treatment depends on:

- Whether tokens locked or staked constitute "securities" or "financial instruments"
  under applicable law.
- Whether early-exit penalty distribution constitutes an "interest payment",
  "yield", or "dividend" requiring licensing.
- Whether the staker rewards pool constitutes "collective investment scheme" operation.
- Whether the Service qualifies as a "virtual asset service provider" (VASP) under
  FATF guidance, EU MiCA, or equivalent frameworks.

### 4.2 Jurisdiction Checklist

Obtain qualified legal opinion for each of the following questions before launch:

| Jurisdiction | Question |
|---|---|
| United States | Does the staking rewards mechanism require SEC registration or FinCEN MSB registration? Does the lock-and-release model fall under the Commodity Exchange Act? |
| European Union | Does SAFE-HAVEN qualify as a Crypto-Asset Service Provider (CASP) under MiCA (Regulation 2023/1114)? Are travel-rule obligations (FATF Recommendation 16) triggered? |
| United Kingdom | Does the penalty-distribution mechanism constitute a "collective investment scheme" under FSMA 2000? |
| Singapore | Does the staking mechanism require a licence under the Payment Services Act 2019? |
| Other jurisdictions | Assess on a case-by-case basis for any jurisdiction where users are permitted to access the Service. |

### 4.3 AML/KYC Considerations

SAFE-HAVEN is non-custodial — the smart contract holds tokens, not the operators.
However, depending on jurisdiction and volume, the frontend may be required to
implement:

- Identity verification (KYC) for users above transaction thresholds.
- Transaction monitoring and suspicious activity reporting.
- Sanctions screening against OFAC/EU/UN lists.

Until legal review is complete, geo-blocking is recommended for jurisdictions with
clear prohibition on such services.

### 4.4 Consumer Protection

Consider including:

- Prominent risk warnings at onboarding ("you may lose all deposited funds").
- A cool-off or confirmation step before finalizing a deposit (the frontend
  already includes a withdrawal confirmation UI; a similar pattern should apply
  to deposits above a threshold).
- Clear display of penalty parameters before a user confirms a deposit.

---

## 5. Data Retention Policy

### 5.1 On-Chain Data

All data written to the Stellar blockchain is **permanent** and cannot be deleted by
any party, including the operators. This includes deposit records, withdrawal events,
admin actions, and staker registrations.

Soroban persistent storage entries are subject to a TTL (time-to-live) mechanism at
the ledger level. The contract extends TTL on every write and read to cover a period
of approximately 5.2 years per the `BUMP_TARGET` constant in `storage.rs`. Expired
storage entries are pruned by the network and cannot be recovered.

### 5.2 Off-Chain Data Retention Schedule

| Category | Data Types | Retention | Deletion Method |
|---|---|---|---|
| Server access logs | IP, user agent, request path, timestamps | 90 days | Automated log rotation |
| Frontend error logs | Error message, stack trace, browser info | 30 days | Automated purge |
| Analytics aggregates | Page views, feature usage counts (no PII) | 24 months | Aggregated; individual events purged at 90 days |
| Support tickets | Email, issue description, wallet address (if provided) | 3 years after close | Manual deletion on request |
| Incident records | Timeline, root-cause, actions (no private keys) | Indefinite | Retained for audit and learning |
| Contract deployment artifacts | WASM, contract ID, manifest, checksum | Indefinite | Retained for reproducibility and emergency recovery |

### 5.3 Minimum Retention Requirements

The following records must be retained for at least the stated period regardless of
deletion requests, to satisfy legal, audit, and regulatory obligations:

| Record | Minimum Retention | Reason |
|---|---|---|
| Incident post-mortems | 3 years | Audit trail, regulatory inquiry |
| Security vulnerability reports (resolved) | 5 years | Potential liability |
| Deployment manifests | Indefinite | Reproducibility, emergency recovery |
| Financial records (if applicable) | 7 years | Tax and accounting |

### 5.4 Deletion Requests

Users may request deletion of off-chain personal data by contacting
[INSERT PRIVACY CONTACT]. Requests are processed within 30 days. Note that
on-chain data cannot be deleted and is excluded from all deletion requests.

---

## 6. Customer Data Handling

### 6.1 Data Minimization

The SAFE-HAVEN frontend is designed to collect the minimum data necessary to
operate the Service:

- No registration or account creation is required.
- No email address, name, or personal identifier is collected unless voluntarily
  provided via a support channel.
- Wallet public keys are handled by Freighter and are only transmitted to the
  Stellar network for transaction purposes.

### 6.2 Data Storage

Off-chain operational data (logs, analytics) is stored in:

- [INSERT STORAGE PROVIDER AND REGION]
- Access is restricted to authorized operators via role-based access control.
- Data at rest is encrypted using [INSERT ENCRYPTION STANDARD].
- Data in transit uses TLS 1.2 or higher.

### 6.3 Access Controls

Access to off-chain user data is subject to:

- Need-to-know basis: only operators with a specific reason access data.
- Audit logging of all data access events.
- Multi-factor authentication for all accounts with data access.
- Quarterly access review to remove stale permissions.

### 6.4 Incident Response for Data Breaches

If a data breach affecting personal data is identified:

1. Contain the breach within 24 hours of discovery.
2. Assess scope: what data was accessed, by whom, for how long.
3. Notify affected individuals within 72 hours where required by applicable law
   (e.g., GDPR Article 33/34).
4. File a report with the relevant data protection authority if required.
5. Document the incident per the process in [POSTMORTEM.md](./POSTMORTEM.md).

For security incidents involving the smart contract itself, follow
[INCIDENT_RESPONSE.md](./INCIDENT_RESPONSE.md).

### 6.5 Third-Party Data Processors

| Processor | Purpose | Data Shared | Agreement |
|---|---|---|---|
| Stellar RPC provider | Transaction submission and query | IP address, transaction XDR | [INSERT DPA REFERENCE] |
| Freighter | Transaction signing | Transaction data only (no private keys) | Browser extension; user controls |
| On-ramp provider | Fiat-to-crypto conversion | Varies by provider | [INSERT DPA REFERENCE] |
| Analytics provider (if used) | Usage metrics | Anonymized / aggregated | [INSERT DPA REFERENCE] |

---

## 7. GDPR Compliance

> This section applies to users in the European Economic Area (EEA), United Kingdom,
> and other jurisdictions with equivalent data protection laws.

### 7.1 Legal Basis for Processing

| Processing Activity | Legal Basis (GDPR Art. 6) |
|---|---|
| Serving the frontend application | Art. 6(1)(b) — Performance of a contract |
| Server access logs | Art. 6(1)(f) — Legitimate interests (security, abuse prevention) |
| Frontend error reporting | Art. 6(1)(f) — Legitimate interests (service reliability) |
| Responding to support inquiries | Art. 6(1)(b) — Performance of a contract |
| Analytics (if used) | Art. 6(1)(a) — Consent (cookie banner) |

### 7.2 Data Subject Rights

Users in the EEA and UK have the following rights under GDPR / UK GDPR:

| Right | How to Exercise | Response Time |
|---|---|---|
| Access (Art. 15) | Email [INSERT PRIVACY CONTACT] with subject "Data Access Request" | 30 days |
| Rectification (Art. 16) | Email [INSERT PRIVACY CONTACT] | 30 days |
| Erasure / "Right to be Forgotten" (Art. 17) | Email [INSERT PRIVACY CONTACT] — note: on-chain data cannot be deleted | 30 days |
| Restriction of Processing (Art. 18) | Email [INSERT PRIVACY CONTACT] | 30 days |
| Data Portability (Art. 20) | Email [INSERT PRIVACY CONTACT] | 30 days |
| Object to Processing (Art. 21) | Email [INSERT PRIVACY CONTACT] | 30 days |

### 7.3 Data Controller and DPO

- **Data Controller:** [INSERT LEGAL ENTITY NAME AND ADDRESS]
- **Data Protection Officer (DPO):** [INSERT NAME OR "not required under Art. 37 — document justification"]
- **EU Representative (if required):** [INSERT NAME AND ADDRESS under Art. 27]

### 7.4 Cross-Border Data Transfers

If personal data is transferred outside the EEA, the following safeguards apply:

- Transfers to the United States: Standard Contractual Clauses (SCCs) issued by the
  European Commission (Decision 2021/914) or an adequacy decision.
- Other jurisdictions: adequacy decision, SCCs, or binding corporate rules as
  applicable.

[INSERT SPECIFIC TRANSFER MECHANISMS FOR EACH THIRD-PARTY PROCESSOR]

### 7.5 Data Protection Impact Assessment (DPIA)

A DPIA is recommended prior to production launch given that the Service processes
financial data and blockchain addresses that may be linkable to natural persons.
The DPIA should assess:

- Systematic or large-scale processing of sensitive financial data.
- Automated decision-making or profiling (currently: none).
- Use of blockchain addresses as unique identifiers that may be pseudonymous but not
  anonymous.

### 7.6 Supervisory Authority

If you believe your data protection rights have been violated, you have the right to
lodge a complaint with the supervisory authority in your EU member state, or with the
ICO in the United Kingdom.

---

## 8. Audit Trail

The SAFE-HAVEN system maintains audit records at multiple layers to satisfy
regulatory compliance and operational accountability requirements.

### 8.1 On-Chain Audit Trail

Every mutating operation on the SAFE-HAVEN smart contract emits a structured event
to the Stellar ledger. These events are permanent, immutable, and publicly verifiable.

| Event | Topics | Data Payload | Significance |
|---|---|---|---|
| `deposit` | `(depositor, token)` | `(amount, unlock_time, deposit_id)` | New deposit created |
| `dep_by_ledger` | `(depositor, token)` | `(amount, unlock_ledger, deposit_id)` | Ledger-based deposit created |
| `withdraw` | `(depositor, token)` | `(amount, deposit_id)` | Normal withdrawal after lock expiry |
| `withdraw_to` | `(depositor, token)` | `(recipient, amount)` | Withdrawal redirected to different address |
| `dep_cancel` | `(depositor, token)` | `(amount, penalty, deposit_id)` | Early exit with penalty applied |
| `emrg_wdraw` | `(depositor)` | `(admin, token, amount, deposit_id)` | Admin-initiated emergency withdrawal |
| `adm_xfr_init` | `(current_admin)` | `pending_admin` | Admin transfer initiated |
| `adm_xfr_done` | `(new_admin)` | — | Admin transfer completed |
| `adm_xfr_cancel` | `(current_admin)` | — | Admin transfer cancelled |
| `adm_renounce` | `(former_admin)` | — | Admin permanently renounced |
| `paused` | — | `admin` | Contract deposits paused |
| `unpaused` | — | `admin` | Contract deposits resumed |
| `contract_initialized` | — | `(admin, fee_recipient, max_deposit, max_lock_secs)` | Contract initialized |
| `staker_registered` | `(staker)` | `stake_amount` | Staker registered or updated |
| `rewards_claimed` | `(staker)` | `reward_amount` | Staker claimed rewards |

All events are indexed by `transaction_hash`, `ledger_sequence`, and `contract_id`
and are queryable via the Stellar Horizon API and any compatible blockchain explorer.

### 8.2 How to Query the Audit Trail

```bash
# Fetch all events for the contract using Stellar CLI
stellar contract events \
  --id <CONTRACT_ID> \
  --network testnet \
  --start-ledger <LEDGER_NUMBER>

# Using Horizon REST API
curl "https://horizon.stellar.org/accounts/<ADDRESS>/transactions"

# Query a specific transaction
curl "https://horizon.stellar.org/transactions/<TRANSACTION_HASH>"
```

The `scripts/smoke_test_local.sh` script demonstrates programmatic event querying
for verification purposes.

### 8.3 Off-Chain Audit Records

In addition to on-chain events, the following off-chain records are maintained:

| Record Type | Location | Retention | Access |
|---|---|---|---|
| Deployment manifests | `deployments/<network>/<timestamp>/` | Indefinite | Repo + operator |
| CI build logs | GitHub Actions | Per GitHub retention policy | Repo contributors |
| Security vulnerability reports | Private channel | 5 years | Security leads |
| Incident post-mortems | POSTMORTEM.md + linked issues | 3 years minimum | Repo (internal) |
| Admin key operation log | Private operational records | Indefinite | Incident commander |

### 8.4 Storage Schema Versioning

The on-chain `StorageVersion` key records the schema version applied by the most
recent `migrate(admin)` call. This provides a verifiable audit point for any
storage layout change:

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <KEY> \
  --network testnet \
  -- get_storage_version
```

Current schema version: `1` (see `types.rs::STORAGE_VERSION`).

### 8.5 Audit Trail Integrity Guarantees

- **Immutability:** All on-chain events are written to the Stellar ledger, which is
  cryptographically chained and append-only. No party — including the contract admin —
  can delete or alter ledger history.
- **Non-repudiation:** Every transaction is signed by the originating Stellar keypair.
  Event topics include the signer address, providing non-repudiation for all actions.
- **Completeness:** The contract emits events for every state-changing operation with
  no silent state transitions.
- **Admin action visibility:** Admin functions (`emergency_withdraw`, `pause`, admin
  transfer) emit specific events allowing external monitoring to detect all privileged
  operations. See [MONITORING.md](./MONITORING.md) for alert configuration.

### 8.6 Audit Log Access for Regulatory Requests

In the event of a regulatory inquiry or legal order requiring audit records:

1. Identify the relevant contract ID, ledger range, and depositor addresses.
2. Export on-chain events using the Stellar Horizon API or a compatible explorer.
3. Export off-chain deployment manifests and incident records.
4. Route the request through qualified legal counsel before disclosure.
5. Do not provide private keys, seed phrases, or unredacted user data in any response.

Contact: [INSERT LEGAL CONTACT EMAIL]

---

*Last reviewed: [INSERT DATE] | Next review due: [INSERT DATE + 12 months]*
*Reviewer: [INSERT NAME / LEGAL COUNSEL FIRM]*
