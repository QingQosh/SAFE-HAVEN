# Deposit Contract

## Overview

The deposit contract holds user funds for a fixed lock duration. Because Soroban
storage entries have a time-to-live (TTL), every write must extend the TTL so
that a deposit entry survives for the entire lock period plus a safety buffer.

## TTL Configuration

Storage TTL is managed with `extend_ttl()` on every `set_*` storage operation.
The relevant constants are:

| Constant         | Purpose                                                        |
| ---------------- | -------------------------------------------------------------- |
| `BUMP_TARGET`    | Target TTL (in ledgers) applied on each write. Set to ~5 years plus buffer so it covers the maximum lock duration. |
| `BUMP_THRESHOLD` | Minimum remaining TTL before a bump is triggered. Provides a buffer so entries are renewed well before expiry. |

### Why these values

- **Maximum lock duration** is ~5 years. `BUMP_TARGET` is set to ~5 years plus
  a buffer so a freshly created deposit never expires before its lock ends.
- **`BUMP_THRESHOLD`** is sized so that entries approaching expiry are renewed
  on the next write, leaving ample headroom before the actual expiry ledger.

### Rules

- `extend_ttl()` MUST be called on every `set_*` storage operation (deposit
  creation and any subsequent update).
- TTL constants are intentionally fixed; changing them is out of scope for
  routine work and requires a dedicated review.

## Monitoring

Entries approaching expiry should be observable so operators can act before a
bump is needed. When reading a deposit, compare the entry's remaining TTL
against `BUMP_THRESHOLD`; entries below the threshold are candidates for
renewal and should be surfaced in monitoring.

## Testing

Tests should verify that:

- A deposit created via `set_*` has its TTL extended to at least `BUMP_TARGET`.
- TTL is extended on every storage write, not just creation.
- Expired entries behave correctly (reads fail / are treated as absent).
