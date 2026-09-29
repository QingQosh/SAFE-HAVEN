# Upgrade Governance

Contract upgrades are governed on-chain. The process requires a review, a security audit, a community vote, a 14-day timelock, and an open veto window.

## Upgrade Process

1. **Propose**: Anyone may submit a proposal with the old and new version labels, the new WASM hash, a human-readable diff URL, and a security-audit report URL. Missing diff or audit evidence is rejected.
2. **Review**: A different account must submit a code-review URL and confirm that the security audit is complete. Until this succeeds, the proposal cannot receive votes. The review should compare the deployed version with the proposed version and link to the exact source or binary diff.
3. **Vote**: Community members authorize one approval or rejection vote each. A proposal becomes approved after three approval votes. Duplicate votes are rejected.
4. **Timelock**: Approval records the chain timestamp. Execution is rejected until 14 days have elapsed.
5. **Veto**: After approval and before execution, any community voter may cast a veto. A veto immediately changes the proposal to `Vetoed` and permanently prevents execution.
6. **Execute**: Anyone may execute an approved, non-vetoed proposal after the timelock. The contract updates its current WASM to the proposal's hash.

## Contract Calls

The governance entry points are:

- `propose_upgrade(proposer, old_version, new_version, diff_url, audit_url, wasm_hash)`
- `review_upgrade(reviewer, proposal_id, review_url, security_audit_complete)`
- `vote_upgrade(voter, proposal_id, approve)`
- `veto_upgrade(voter, proposal_id)`
- `execute_upgrade(proposal_id)`
- `get_upgrade_proposal(proposal_id)`

The `wasm_hash` must identify the exact reviewed artifact. The contract stores the evidence links and vote/timelock state with the proposal so clients can show the old/new version comparison and audit trail before users vote.

## Operational Review Checklist

- Build the candidate WASM from a pinned commit.
- Publish a source or binary diff between the deployed version and candidate.
- Complete an independent security audit covering contract logic, storage migrations, authorization, and upgrade code.
- Have a reviewer verify that the audited artifact hash matches `wasm_hash`.
- Confirm the proposal status and evidence with `get_upgrade_proposal` before voting.
- Monitor the 14-day timelock and veto window; execute only after the final status check.
