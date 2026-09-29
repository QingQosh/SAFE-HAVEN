# SAFE-HAVEN Performance Guide for Operators

This guide covers the contract and its RPC-facing workloads. It does not prescribe infrastructure or server configuration and does not modify contract behavior.

## Performance model

A deposit or withdrawal combines authorization, persistent storage, a token transfer, and event publication. Cost grows with the number of storage reads/writes and the size of serialized vectors. Read-only simulations avoid signing but still consume RPC and simulation capacity.

The Soroban instruction budget is finite. Treat the repository's approximate planning figures (about 100M instructions on testnet and 50M on mainnet) as operational estimates, not protocol guarantees. The network and protocol version are authoritative.

## Bounded access patterns

- Use `get_deposit_ids(depositor)` to enumerate one account's active IDs. It reads the maintained active-ID list instead of scanning every historical counter value.
- Use `get_depositors(offset, limit)` for depositor pages. Keep pages small enough for the target RPC and simulation budget.
- Use `get_deposit_batch(depositor, ids)` for up to **25** IDs per call (`MAX_BATCH_SIZE`). Split larger requests into pages.
- `get_deposits_page(offset, limit)` is a timestamp-deposit view and walks depositor indexes up to the requested range. Keep production limits at **50 or fewer**, as recommended by the contract comments.
- Ledger deposits are not returned by `get_deposits_page`; enumerate depositors and active IDs, then query `get_ledger_vault` or `get_deposit_type`.
- Avoid fetching every depositor or deposit in a single request. Cache immutable display data such as token metadata, but recheck unlock state and transaction status on-chain.

Before relying on `get_vault_batch`, verify the deployed WASM and ABI against the source and CI. The current source signature accepts a depositor list plus one ID, while its implementation references `pairs`; this is a release-blocking consistency check until compiled and tested.

## Storage and TTL

Persistent entries are bumped on writes. The code derives:

- `LEDGER_SECONDS = 5` (an estimate of average ledger spacing).
- `BUMP_TARGET = ceil(157,788,000 / 5) = 31,557,600` ledgers, covering the default maximum lock duration.
- `BUMP_THRESHOLD = 15,778,800` ledgers, half the target.

Mutable reads used by later state changes can extend TTL; read-only query variants intentionally do not, avoiding a write cost. Operators should not treat a successful read-only query as a TTL renewal.

Monitor long-lived deposits and migration health. A deposit that outlives its storage TTL can read as missing, so keep the deployed limits, TTL policy, and upgrade process aligned. The admin-only `migrate` entry point is currently an idempotent schema-version hook; future schema changes must add and test real migration steps before deployment.

## Gas and cost estimation

For a representative operation:

1. Build the transaction with the actual caller and arguments.
2. Simulate it through the target Soroban RPC.
3. Assemble the transaction using the returned resource data.
4. Record CPU instructions, memory, ledger reads/writes, and transaction fee from the simulation/result.
5. Repeat at the expected page or batch sizes and with both success and error paths.

The frontend follows this build, simulate, assemble, sign, submit, and poll sequence. Do not estimate a production operation from a read-only query alone. Token contract behavior and vector size can materially change resource use.

## Benchmarking method

Run benchmarks against the same network and contract version used in production. Record the contract `version()`, network passphrase, RPC endpoint, ledger range, account count, active deposits per account, token type, and page/batch size.

At minimum measure:

- one deposit and one withdrawal;
- early cancellation with zero and non-zero penalties;
- one account with 1, 25, and 50 active deposits;
- depositor pages at 25 and 50 items;
- mixed timestamp and ledger deposits;
- cold and repeated read-only simulations;
- successful calls and expected failures such as `FundsStillLocked`.

Capture p50/p95 latency, instruction count, read/write counts, fee, RPC errors, and confirmation time. A benchmark is actionable only when the workload and contract version are reproducible.

## Scaling guidance

Scale reads horizontally at the client/indexer layer: cache token metadata, queue RPC work, use bounded concurrency, and checkpoint pagination. Preserve ordering and deduplicate by `(depositor, deposit_id)`.

For writes, serialize actions per deposit ID, avoid retrying an unknown-status transaction blindly, and inspect the transaction hash before resubmitting. Keep batch sizes at or below contract limits even if an RPC accepts larger payloads.

The append-only depositor index can contain stale addresses after all of an address's vaults are removed; reads skip inactive flags. This avoids expensive list compaction, but enumeration still has historical work. Monitor page latency as the index grows.

## Slow or failed transactions

1. Confirm the network and contract ID.
2. Check simulation output for an instruction, authorization, footprint, or contract error.
3. Reduce page/batch size and retry a read-only query.
4. Check whether the call is walking a large historical depositor index.
5. For writes, verify the wallet signature and token authorization, then inspect the submitted hash.
6. Distinguish pending confirmation from failed execution; the frontend polls for up to 30 attempts at two-second intervals.
7. Escalate with contract version, network, method, arguments excluding secrets, simulation output, transaction hash, and timing metrics.

Do not increase limits or retry an unknown-status withdrawal without checking chain state first. That can create confusing duplicate UX even when the contract correctly prevents a second withdrawal.

## Operational checklist

- [ ] Contract `version()` and network are recorded for every deployment.
- [ ] CI, contract tests, WASM build, size check, and local smoke test pass.
- [ ] Batch and pagination callers enforce 25/50 operational limits.
- [ ] Benchmarks include both lock types and expected error paths.
- [ ] TTL assumptions match the configured maximum lock duration.
- [ ] Migration is tested against a representative deployed state.
- [ ] RPC latency, failures, instruction use, and confirmation times are monitored.
- [ ] Incident reports omit secret keys and recovery phrases.
