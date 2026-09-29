# SAFE-HAVEN Contract Gas Profile

## Scope and method

Soroban does not assign one permanent gas number to a function. The fee depends on the protocol version, transaction footprint, serialized argument/result size, storage entry sizes, and whether a token transfer is included. The numbers below are therefore structural profiles. Record exact CPU instructions, memory bytes, ledger reads/writes, and fee from RPC simulation for a deployed WASM at each release.

Profile the same scenario at these boundaries:

- empty state;
- one active deposit;
- 25 active deposits;
- 100 active depositors;
- 25 and 100 active deposits for one voter.

For each scenario, build the transaction, call Soroban RPC `simulateTransaction`, and save the response `cost` object with the contract version and protocol version. Compare `cpu_insns`, `mem_bytes`, `ledger_read_bytes`, `ledger_write_bytes`, `ledger_read_entries`, `ledger_write_entries`, `events_size_bytes`, and the returned resource fee. Do not compare fee alone when diagnosing a regression.

## Cost model

- **S**: fixed persistent storage operation. Reads and writes are the dominant ledger I/O cost.
- **L(n)**: a loop over `n` stored elements; CPU, reads, and serialized writes grow with `n`.
- **T**: one SAC/token transfer, including the token contract invocation. This is usually the largest fixed component of a successful deposit or withdrawal.
- **E**: one event publication; event payload and topics add CPU, memory, and transaction footprint.
- **A**: authorization and address validation overhead.

`n` means the number of active IDs, depositors, or input items relevant to that function. The profiles intentionally use relative categories instead of fabricated fixed fee values.

## Function profile

| Function | Profile | Expensive work and scaling |
|---|---:|---|
| `initialize` | S(3-5)+E+A | Writes admin, initialized, fee recipient, optional limits, then reads effective limits. |
| `deposit` | S(7-10)+T+E+A | Paused/allowlist/limit reads, counter write, transfer, deposit write, active-ID read/write, depositor flags/list writes. |
| `deposit_for` | S(7-10)+T+E+A | Same as `deposit`; payer authorization replaces depositor authorization. |
| `deposit_by_ledger` | S(7-10)+T+E+A | Same lifecycle as `deposit`, with ledger unlock validation. |
| `cancel_deposit` | S(4-8)+T(1-2)+E | Reads both deposit forms in the worst case, rewrites active IDs, may transfer penalty and refund separately. |
| `withdraw` | S(3-6)+T+E | Reads both forms in the worst case, removes the deposit and active ID, may remove depositor flag. |
| `withdraw_to` | S(3-6)+T+E | Same as `withdraw`, with a recipient address. |
| `emergency_withdraw` | S(3-6)+T+E+A | Admin authorization plus the same deposit removal path; callable by anyone presenting the admin address but requiring admin auth. |
| `pause` / `unpause` | S(2)+E+A | Admin read, paused write, event. |
| `propose_pause` | S(3)+E+A | Proposal counter read/write and proposal write. |
| `vote` | S(3)+L(n)+E+A | Proposal and vote-marker reads/writes; community mode scans the voter's active IDs and reads each deposit. Admin mode is fixed-cost. |
| `execute_proposal` | S(2)+E | Proposal read/write and paused write for a passed pause proposal. |
| `get_proposal` | S(1) | Single proposal read and result serialization. |
| `proposal_passed` | S(1) | Single proposal read and comparison. |
| `get_voting_power` | S(1)+L(n) | Reads the active-ID vector and up to two deposit entries per ID. This is the main governance scaling hotspot. |
| `add_allowed_token` | S(1)+A | One allowlist write plus admin authorization. |
| `remove_allowed_token` | S(1)+A | One allowlist removal plus admin authorization. |
| `set_strict_mode` / `toggle_strict_mode` | S(1)+A | One mode read for toggle and one mode write. |
| `is_strict_mode` / `is_token_allowed` | S(1) | Single persistent read. |
| `transfer_admin` | S(2)+E+A | Admin read and pending-admin write. |
| `accept_admin` | S(2)+E+A | Pending-admin read, admin write, pending-admin removal. |
| `cancel_transfer_admin` | S(2)+E+A | Admin read; pending-admin removal/event only when pending state exists. |
| `renounce_admin` | S(2-3)+E+A | Admin read, admin/pending-admin removals, event. |
| `get_vault` / `get_ledger_vault` | S(1) | Single read-only entry lookup. |
| `get_vault_batch` | S(min(input,25)) | One read per input address and result serialization; bounded by `MAX_BATCH_SIZE`. |
| `get_deposit_batch` | S(min(input,25)) | One read per requested deposit ID and result serialization. |
| `get_deposit_ids` | S(1) | Returns the stored active-ID vector; cost grows with serialized vector size. |
| `get_time` | fixed | Ledger timestamp only. |
| `time_remaining` | S(1-2) | Reads timestamp form, then ledger form only on a miss. |
| `get_admin`, `get_pending_admin`, `get_fee_recipient`, `get_constants`, `is_initialized`, `get_storage_version` | S(1-2) | Fixed read-only queries; `get_constants` performs two reads. |
| `get_token_vetting` | S(1) | Single vetting-record read and struct serialization. |
| `propose_token` | S(2)+E+A | Allowlist read and vetting-record write. |
| `review_token` | S(2)+E+A | Admin read, vetting read/write, event. |
| `approve_token` | S(3)+E+A | Admin and vetting reads, vetting write, allowlist write, event. |
| `get_depositor_count` | L(d)+S(d) | Scans the append-only depositor list and reads each activity flag. |
| `get_depositors` | L(d)+S(d) | Scans until the requested active page is found, skipping stale entries. |
| `get_deposits_page` | L(d)+L(n)+S(n) | Scans active depositors, then each active-ID vector and timestamp deposit. |
| `migrate` | S(1)+A | Fixed today; future migrations may scale with all deposits if backfills are added. |

## Highest-risk hotspots

1. **`get_voting_power`** is linear in a voter’s active deposits and performs a second lookup for ledger-based deposits on misses. It is invoked inside `vote`, so a large position can make voting expensive or exceed the instruction budget.
2. **`get_depositors` and `get_depositor_count`** scan an append-only list and read one flag per historical address. Stale entries make both functions grow permanently.
3. **`get_deposits_page`** nests the depositor scan and deposit-ID/deposit reads. It is intentionally useful for pagination but is not a cheap global view.
4. **Successful deposit/withdraw/cancel paths** combine persistent storage churn with one or more token contract calls. Avoid adding extra reads after validation in these paths.
5. **Large `Vec` arguments/results** in batch and page functions increase memory and serialization costs even when storage work is bounded.

## Recommendations

### P0: measure before changing behavior

- Add a release benchmark harness that records simulation `cost` for the scenario matrix above.
- Keep a baseline JSON artifact per contract release; flag regressions by resource dimension, not only total fee.
- Profile both timestamp and ledger deposit variants and both cold and warm persistent entries.

### P1: make voting power O(1)

Maintain a per-address `VotingPower` aggregate whenever a deposit is created or removed. Vote then reads one aggregate instead of walking active IDs. If stake must be snapshotted, store the weight with the vote; the current implementation counts weight at vote time and already leaves completed vote totals unchanged after later withdrawals.

A token-aware aggregate is safer than summing raw amounts across unrelated assets. Options are a single governance-denominated token or one aggregate per approved token with an explicit conversion policy.

### P1: replace append-only depositor scans

Use an active depositor index with swap-and-pop removal, or maintain an active count and bounded page cursor. This avoids repeatedly reading stale flags. If preserving append-only history is required for indexing, keep a separate compact active index for contract queries.

### P1: avoid duplicate timestamp/ledger probes

Store a deposit kind alongside the active ID, or use a tagged `DepositRecord` enum under one key. This lets `withdraw`, `cancel_deposit`, `time_remaining`, and voting-power reads perform one lookup rather than probing timestamp storage and then ledger storage.

### P2: reduce repeated configuration reads

Validation in the three deposit entrypoints is duplicated. A private validation helper can consolidate policy reads and make future additions easier to benchmark. Do not cache mutable configuration in contract instance state; persistent storage remains the source of truth.

### P2: keep batch limits explicit

Retain the 25-item cap unless simulation demonstrates safe headroom. For larger views, paginate by cursor and return only the requested page. Avoid increasing `MAX_BATCH_SIZE` as a first response to UI latency.

### P2: separate governance execution from counting

`proposal_passed` is cheap because it reads stored totals. Keep vote counting incremental, as it is now; never recompute all voter weights during execution. Consider a quorum field and a proposal-level total eligible stake if governance requires more than simple strict majority.

## Governance assumptions to verify

- Community weight currently sums raw active deposit amounts, even when deposits use different tokens. This requires a documented denomination policy before production use.
- There is no quorum requirement; a proposal passes when `for_votes > against_votes`.
- Admin-vote weight is one vote for the current admin, and the admin must also be the proposer for an `AdminVote` proposal.
- Proposal execution is permissionless after the voting period and timelock, which avoids making execution depend on the proposer being online.
