# Create Issues 1-25
$repo = "shortheartone/SAFE-HAVEN"

Write-Host "Creating Issues 1-25..." -ForegroundColor Cyan

# Issue 1
gh issue create --repo $repo --title "Add Input Validation for Token Address in deposit() Function" --body "**Summary:** The deposit() function currently accepts any Address type for the token parameter without validating that it's a legitimate token contract. This could lead to runtime errors when the contract attempts to transfer tokens from an invalid or non-existent token address. Adding validation would improve error handling and provide clearer error messages to users. Early validation prevents wasted gas and confusing transaction failures.

**Scope of Work:**
- Add a validation check in the deposit() function to verify the token address is valid
- Implement a helper function to check if an address corresponds to a deployed contract
- Return a descriptive error (e.g., InvalidTokenAddress) if validation fails
- Add similar validation to deposit_for() and deposit_by_ledger() functions
- Update error.rs to include the new error variant

**Out of Scope:**
- Validating that the token implements the full SAC interface (beyond existence check)
- Adding a whitelist of approved tokens
- Modifying the token transfer logic itself

**Acceptance Criteria:**
- [ ] deposit() rejects invalid token addresses with a clear error message
- [ ] deposit_for() and deposit_by_ledger() include the same validation
- [ ] New error code InvalidTokenAddress is added to VaultError enum
- [ ] Unit tests verify rejection of zero address and non-contract addresses
- [ ] All existing tests continue to pass"

Start-Sleep -Seconds 2
Write-Host "Issue 1 created" -ForegroundColor Green

# Issue 2
gh issue create --repo $repo --title "Implement Batch Withdrawal Function" --body "**Summary:** Users with multiple deposits currently need to call withdraw() separately for each deposit, resulting in multiple transactions and higher fees. A batch withdrawal function would allow users to withdraw from multiple deposits in a single transaction. This improves user experience and reduces transaction costs. The function should maintain the same security checks as individual withdrawals.

**Scope of Work:**
- Create a new withdraw_batch() function accepting a vector of deposit IDs
- Iterate through each deposit ID and perform standard withdrawal checks
- Accumulate successful withdrawals and track failures
- Return a result structure indicating which withdrawals succeeded
- Add comprehensive unit tests for batch withdrawal scenarios
- Update events.rs to emit appropriate events for batch operations

**Out of Scope:**
- Cross-depositor batch withdrawals (only same depositor)
- Automatic retry logic for failed withdrawals within the batch
- Partial refunds for mixed token types

**Acceptance Criteria:**
- [ ] withdraw_batch() successfully processes multiple valid withdrawals
- [ ] Function properly handles mix of locked and unlocked deposits
- [ ] Gas consumption scales linearly with number of deposits
- [ ] Events are emitted for each individual withdrawal in the batch
- [ ] Function respects MAX_BATCH_SIZE constant
- [ ] Tests cover edge cases (empty list, all locked, all unlocked)"

Start-Sleep -Seconds 2
Write-Host "Issue 2 created" -ForegroundColor Green

Write-Host "Batch 1 (Issues 1-2) completed. Run create-batch-2.ps1 for more issues." -ForegroundColor Yellow
