# Circuit Breakers

SAFE-HAVEN has one fixed circuit-breaker condition: cumulative emergency withdrawals in a single ledger may not exceed `100_000_000` token units.

An emergency withdrawal that brings the ledger total exactly to the threshold succeeds and immediately pauses the vault. A withdrawal that would exceed the threshold is rejected before any funds move. Pausing blocks normal deposit operations through the existing `ContractPaused` guard; there is no automatic reset. The admin is notified by the `CircuitBreakerTripped` event, and each activation is retained in `get_circuit_breaker_history` with the ledger, observed total, and threshold.