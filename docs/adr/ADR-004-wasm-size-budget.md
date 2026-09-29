# ADR-004 — WASM Size Budget and Optimization Pipeline

| Field | Value |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-07-27 |
| **Deciders** | SAFE-HAVEN core contributors |
| **Issue / PR** | — |
| **Supersedes** | — |
| **Superseded by** | — |

---

## Context

Soroban contracts are deployed on-chain as WASM bytecode. Every byte of the
WASM binary is stored in Stellar's ledger and contributes to:

1. **Upload cost** — the `stellar contract deploy` fee scales with binary
   size. Larger binaries cost more XLM to deploy.
2. **Footprint rent** — Soroban charges TTL-based rent on every storage
   entry. The contract code entry (`ContractCode`) is itself a ledger entry
   subject to TTL expiry and rent. A larger binary increases the minimum-rent
   cost of keeping the contract alive.
3. **Load time** — the runtime loads and deserializes the WASM before
   execution. Larger binaries have higher per-invocation overhead.

Soroban's mainnet instruction budget is tight (~50M instructions per
transaction). A feature-rich contract can easily exceed 64 KB before
optimization, which pushes it into a more expensive fee tier. Keeping the
binary small is a first-class engineering constraint, not an afterthought.

The project needed a clear, enforceable size limit and an automated
pipeline to enforce it in CI.

---

## Decision

### Size limit: 64 KB on the optimized binary

The hard limit is **65,536 bytes (64 KiB)** applied to the output of
`stellar contract optimize`. This limit is enforced in:

1. `make check-wasm-size` — fails the build if the optimized binary exceeds
   the limit.
2. CI (`build` job) — runs `check-wasm-size` on every push and PR so
   regressions are caught before merge.

The raw (unoptimized) WASM is significantly larger — typically 200–500 KB
for a contract of this complexity — and is not subject to a hard limit.
Only the optimized binary is checked.

### Optimization pipeline

```
cargo build --target wasm32-unknown-unknown --release
    ↓
stellar contract optimize --wasm ... --wasm-out ...
    ↓
make check-wasm-size   (65 536 byte limit)
```

The Cargo release profile is tuned for size in `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"        # optimize for size over speed
overflow-checks = true # retain arithmetic safety (no cost in WASM)
debug = false
strip = "symbols"
debug-assertions = false
panic = "abort"        # smaller panic handler than "unwind"
codegen-units = 1      # enables more aggressive inlining/elimination
lto = true             # link-time optimization across all codegen units
```

`opt-level = "z"` is the most aggressive size optimization in the Rust
compiler. `lto = true` with `codegen-units = 1` allows LLVM to eliminate
unused code across the entire crate graph.

### `release-with-logs` profile

For debugging on testnet, the `release-with-logs` profile inherits
`release` but re-enables `debug-assertions`, which activates Soroban's
internal log/event macros. It is not intended for production deployments and
is not subject to the size check.

---

## Consequences

### Positive

- Size regressions are caught in CI before they are merged. A contributor
  adding a large dependency or a bloated feature will see the `build` CI job
  fail with a clear error message.
- The limit is a single constant (`MAX_WASM_BYTES = 65536`) shared between
  the Makefile and CI, ensuring no drift between local and CI checks.
- The `release` profile's size optimizations reduce deployment cost and
  ledger rent for every user of the contract.
- `overflow-checks = true` is retained despite the size optimization because
  arithmetic overflows in a value-custody contract are a critical failure
  mode. The WASM runtime enforces this at near-zero cost.

### Negative / Risks

- `opt-level = "z"` can occasionally produce slower code than `opt-level = 3`
  or `opt-level = "s"`. For a Soroban contract whose hot paths are bounded
  by the instruction budget rather than raw CPU cycles, this tradeoff is
  acceptable.
- If the contract needs to grow significantly (e.g., adding batched
  migrations, complex governance logic), the 64 KB limit may need to be
  revisited. The `MAX_WASM_BYTES` constant can be updated via a PR with an
  accompanying ADR update explaining why the higher limit is justified.
- `stellar contract optimize` (which wraps `wasm-opt`) is an external
  dependency. If the tool version changes, the optimized size may change.
  CI pins the Stellar CLI to `26.0.0` to ensure determinism.

---

## Alternatives Considered

### Alternative A — No hard size limit

Trust contributors not to add excessive code. Run the optimizer but only
report the size, not fail the build.

Rejected because size regressions are invisible without a hard check.
Deployment and rent costs compound over time. A 64 KB limit enforced in CI
costs nothing for features that stay within budget and catches problems
early for those that don't.

### Alternative B — Use `wasm-opt` directly instead of `stellar contract optimize`

`stellar contract optimize` wraps `wasm-opt` (Binaryen) internally.
Calling `wasm-opt` directly would allow finer control over optimization
flags.

Rejected because `stellar contract optimize` ensures the output is
compatible with Soroban's WASM validator. A manually tuned `wasm-opt`
invocation could produce a binary that `wasm-opt` accepts but Soroban
rejects due to unsupported WASM features or custom section stripping. Using
the official CLI is the safer default.

### Alternative C — 128 KB limit

A more permissive limit would give the contract room to grow without
triggering the check.

Rejected at this time because the current contract comfortably fits within
64 KB after optimization. A higher limit would obscure regressions and
increase deployment costs unnecessarily. If the contract legitimately
outgrows 64 KB, the limit should be updated deliberately with a clear
justification, not set permissively from the start.

---

## References

- `Cargo.toml` — `[profile.release]` section
- `Makefile` — `optimize`, `check-wasm-size` targets
- `.github/workflows/ci.yml` — `build` job, `Check WASM size` step
- [Soroban Wasm Size Optimization guide](https://developers.stellar.org/docs/learn/encyclopedia/contract-development/optimization)
- [Binaryen wasm-opt](https://github.com/WebAssembly/binaryen)
