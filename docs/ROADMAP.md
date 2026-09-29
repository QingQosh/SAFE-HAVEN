# SAFE-HAVEN Development Roadmap

**Planning window:** Q3 2026 through Q2 2027 (four quarters, reviewed quarterly)

This roadmap describes intended priorities, not delivery commitments. Dates may change as security findings, Soroban changes, and community feedback change the order of work.

## Status

- **Completed:** shipped in the current contract or frontend.
- **In progress:** present in the repository or actively being hardened, tested, or documented.
- **Planned:** intended work that is not yet implemented.

## Q3 2026: Reliability and public readiness

| Status | Work | Related issue |
|---|---|---|
| Completed | Timestamp deposits, withdrawals, early cancellation, configurable penalties, admin recovery, pause controls, two-step admin transfer, and admin renunciation | [#411](https://github.com/shortheartone/SAFE-HAVEN/issues/411) |
| Completed | Ledger-sequence deposits and transparent withdrawal/cancellation support | [#88](https://github.com/shortheartone/SAFE-HAVEN/issues/88) |
| Completed | Public vault queries, countdown support, pagination primitives, and bounded batch APIs | [#21](https://github.com/shortheartone/SAFE-HAVEN/issues/21), [#44](https://github.com/shortheartone/SAFE-HAVEN/issues/44), [#81](https://github.com/shortheartone/SAFE-HAVEN/issues/81) |
| In progress | Reconcile contract compilation, CI, WASM-size checks, and smoke tests before the next deployment; verify every batch-query path against the shipped ABI | [#411](https://github.com/shortheartone/SAFE-HAVEN/issues/411) |
| In progress | Publish user onboarding, operator performance, and roadmap documentation | [#412](https://github.com/shortheartone/SAFE-HAVEN/issues/412), [#414](https://github.com/shortheartone/SAFE-HAVEN/issues/414) |

**Why first:** users need a dependable and understandable deposit/withdrawal path before new features add surface area. Release verification also protects funds and keeps the public documentation aligned with the deployed contract.

## Q4 2026: Safer operations and better discovery

| Status | Work | Related issue |
|---|---|---|
| Planned | Add automated contract/API compatibility checks for timestamp and ledger deposit types | [#411](https://github.com/shortheartone/SAFE-HAVEN/issues/411) |
| Planned | Improve frontend discovery for ledger-based deposits and explain estimated time remaining | [#88](https://github.com/shortheartone/SAFE-HAVEN/issues/88) |
| Planned | Add operator dashboards or exportable metrics for active depositors, failed transactions, and storage-health checks | [#414](https://github.com/shortheartone/SAFE-HAVEN/issues/414) |
| Planned | Collect and triage community feedback through GitHub Issues and Discussions | [feedback](https://github.com/shortheartone/SAFE-HAVEN/discussions) |

**Why next:** operational visibility and clear handling of both lock models reduce support load and make failures diagnosable without changing custody behavior.

## Q1 2027: User confidence and maintainability

| Status | Work | Related issue |
|---|---|---|
| Planned | Add stronger transaction-state recovery and clearer network/token validation in the frontend | [#412](https://github.com/shortheartone/SAFE-HAVEN/issues/412) |
| Planned | Expand examples, FAQ coverage, accessibility review, and responsive onboarding flows | [#412](https://github.com/shortheartone/SAFE-HAVEN/issues/412) |
| Planned | Exercise storage migration against representative pre-versioning state and publish the runbook | [#414](https://github.com/shortheartone/SAFE-HAVEN/issues/414) |
| Planned | Review security assumptions and publish the results of the next test/audit cycle | [security policy](../SECURITY.md) |

**Why next:** once the operational baseline is measurable, the highest-value improvements are reducing user error and making upgrades repeatable.

## Q2 2027: Scale-informed improvements

| Status | Work | Related issue |
|---|---|---|
| Planned | Benchmark realistic deposit counts and batch sizes on the target network, then tune documented defaults | [#414](https://github.com/shortheartone/SAFE-HAVEN/issues/414) |
| Planned | Improve large-account and multi-depositor indexing workflows without removing bounded query limits | [#414](https://github.com/shortheartone/SAFE-HAVEN/issues/414) |
| Planned | Review roadmap outcomes, close completed items, and publish the next four-quarter plan | [#411](https://github.com/shortheartone/SAFE-HAVEN/issues/411) |

**Why next:** scaling decisions should follow measured instruction, RPC, and storage behavior rather than optimistic limits.

## Feedback and quarterly updates

- Comment on the related issue or open a new item in [GitHub Issues](https://github.com/shortheartone/SAFE-HAVEN/issues).
- Discuss priorities in [GitHub Discussions](https://github.com/shortheartone/SAFE-HAVEN/discussions).
- Include the quarter, affected workflow, network, transaction hash or reproducible steps, and the user impact when reporting feedback.
- The maintainers review this document at the start of each quarter, update statuses and links, record material changes in `CHANGELOG.md`, and publish the revised document from the README and frontend footer.

This roadmap does not include confidential strategy, investment advice, tax guidance, or delivery guarantees.
