# get_ledger_vault() Usage Examples

## Overview

`get_ledger_vault()` retrieves ledger-based deposits from the SAFE-HAVEN vault contract. This document shows practical examples of using this query in tests and client code.

---

## Example 1: Basic Query (Existing Deposit)

### Scenario
Depositor wants to check the details of their ledger-locked deposit.

### Code

```rust
#[test]
fn example_retrieve_ledger_deposit() {
    let (env, vault, token, _admin, alice, _) = setup();
    
    // Create a ledger-based deposit
    let current_ledger = env.ledger().sequence();
    let unlock_ledger = current_ledger + 1_000_000;  // ~5 years
    
    let deposit_id = vault.deposit_by_ledger(
        &alice,
        &token,
        &1_000_000_000,  // 1,000 tokens
        &unlock_ledger,
        &500,            // 5% penalty
    );
    
    // Query the deposit
    let entry = vault.get_ledger_vault(&alice, &deposit_id)
        .expect("deposit should exist");
    
    // Verify all fields
    assert_eq!(entry.token, token);
    assert_eq!(entry.amount, 1_000_000_000);
    assert_eq!(entry.unlock_ledger, unlock_ledger);
    assert_eq!(entry.depositor, alice);
    assert_eq!(entry.penalty_bps, 500);
}
```

**Output**:
```
LedgerVaultEntry {
    token: "GBUQWP3BOUZX34ULNQG23RQ6F4YUSXHTQGSH5H7P42SEMXBOJTHXSANK",
    amount: 1000000000,
    unlock_ledger: 1000000,
    depositor: "GBBD47UZQ5XOTRBE5YNPF675EJXSSA5AHNQE6XBMQDOO2KDLVEDV5HH7",
    penalty_bps: 500,
}
```

---

## Example 2: Non-Existent Deposit (Returns None)

### Scenario
User tries to query a deposit that doesn't exist.

### Code

```rust
#[test]
fn example_nonexistent_deposit() {
    let (env, vault, _token, _admin, alice, _) = setup();
    
    // Query deposit ID 999 which was never created
    let result = vault.get_ledger_vault(&alice, &999);
    
    assert!(result.is_none(), "non-existent deposit should return None");
}
```

**Output**:
```
None
```

---

## Example 3: Withdrawing a Ledger-Based Deposit

### Scenario
After unlock time is reached, depositor withdraws their funds.

### Code

```rust
#[test]
fn example_withdraw_after_unlock() {
    let (env, vault, token, _admin, alice, _) = setup();
    
    // Create deposit that unlocks in 100 ledgers
    let current_ledger = env.ledger().sequence();
    let unlock_ledger = current_ledger + 100;
    
    let deposit_id = vault.deposit_by_ledger(
        &alice,
        &token,
        &5_000_000,  // 5 tokens
        &unlock_ledger,
        &1000,       // 10% penalty if early
    );
    
    // Verify deposit exists
    let entry = vault.get_ledger_vault(&alice, &deposit_id)
        .expect("deposit should exist");
    assert_eq!(entry.amount, 5_000_000);
    
    // Advance ledger past unlock time
    env.ledger().with_sequence(unlock_ledger + 1);
    
    // Withdraw
    let amount = vault.withdraw(&alice, &deposit_id);
    assert_eq!(amount, 5_000_000);
    
    // Verify deposit is gone
    assert!(vault.get_ledger_vault(&alice, &deposit_id).is_none());
}
```

---

## Example 4: Early Withdrawal (Cancel with Penalty)

### Scenario
Depositor cancels before unlock time and pays penalty.

### Code

```rust
#[test]
fn example_cancel_with_penalty() {
    let (env, vault, token, _admin, alice, _) = setup();
    
    // Create deposit with 5% penalty
    let current_ledger = env.ledger().sequence();
    let unlock_ledger = current_ledger + 1_000_000;
    
    let deposit_id = vault.deposit_by_ledger(
        &alice,
        &token,
        &1_000_000,  // 1 token
        &unlock_ledger,
        &500,        // 5% penalty
    );
    
    // Verify deposit details
    let entry = vault.get_ledger_vault(&alice, &deposit_id)
        .expect("deposit should exist");
    
    println!("Deposit amount: {}", entry.amount);        // 1_000_000
    println!("Penalty BPS: {}", entry.penalty_bps);      // 500
    println!("Penalty amount: {}", entry.amount * 500 / 10_000);  // 50_000 (5%)
    
    // Cancel early
    let net_amount = vault.cancel_deposit(&alice, &deposit_id);
    
    // Depositor receives: 1_000_000 - 50_000 = 950_000
    assert_eq!(net_amount, 950_000);
    
    // Verify deposit removed
    assert!(vault.get_ledger_vault(&alice, &deposit_id).is_none());
}
```

**Output**:
```
Deposit amount: 1000000
Penalty BPS: 500
Penalty amount: 50000
```

---

## Example 5: Distinguishing Deposit Types

### Scenario
User needs to determine if a deposit is time-based, ledger-based, or multi-token.

### Code

```rust
#[test]
fn example_check_deposit_type() {
    let (env, vault, token, _admin, alice, _) = setup();
    
    // Create a ledger-based deposit
    let current_ledger = env.ledger().sequence();
    let unlock_ledger = current_ledger + 100_000;
    
    let ledger_id = vault.deposit_by_ledger(
        &alice,
        &token,
        &1_000_000,
        &unlock_ledger,
        &0,
    );
    
    // Create a timestamp-based deposit
    let now = env.ledger().timestamp();
    let unlock_time = now + 365 * 24 * 60 * 60;  // 1 year
    
    let time_id = vault.deposit(
        &alice,
        &token,
        &2_000_000,
        &unlock_time,
        &0,
    );
    
    // Check types
    match vault.get_deposit_type(&alice, &ledger_id) {
        Some(DepositType::LedgerBased) => {
            println!("Deposit {} is ledger-based", ledger_id);
            let entry = vault.get_ledger_vault(&alice, &ledger_id).unwrap();
            println!("Unlocks at ledger: {}", entry.unlock_ledger);
        },
        _ => panic!("Should be ledger-based"),
    }
    
    match vault.get_deposit_type(&alice, &time_id) {
        Some(DepositType::TimeBased) => {
            println!("Deposit {} is time-based", time_id);
            let entry = vault.get_vault(&alice, &time_id).unwrap();
            println!("Unlocks at timestamp: {}", entry.unlock_time);
        },
        _ => panic!("Should be time-based"),
    }
}
```

**Output**:
```
Deposit 0 is ledger-based
Unlocks at ledger: 100000
Deposit 1 is time-based
Unlocks at timestamp: 1701000000
```

---

## Example 6: Batch Query Multiple Depositors

### Scenario
Query the same deposit ID across multiple users.

### Code

```rust
#[test]
fn example_query_multiple_users() {
    let (env, vault, token, _admin, alice, _) = setup();
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    
    StellarAssetClient::new(&env, &token).mint(&bob, &10_000_000);
    StellarAssetClient::new(&env, &token).mint(&carol, &10_000_000);
    
    let current_ledger = env.ledger().sequence();
    let unlock_ledger = current_ledger + 100_000;
    
    // Alice creates deposit 0
    vault.deposit_by_ledger(&alice, &token, &1_000_000, &unlock_ledger, &500);
    
    // Bob creates deposit 0
    vault.deposit_by_ledger(&bob, &token, &2_000_000, &unlock_ledger, &500);
    
    // Carol has no deposit 0
    
    // Query all three
    let alice_entry = vault.get_ledger_vault(&alice, &0);
    let bob_entry = vault.get_ledger_vault(&bob, &0);
    let carol_entry = vault.get_ledger_vault(&carol, &0);
    
    assert!(alice_entry.is_some());
    assert!(bob_entry.is_some());
    assert!(carol_entry.is_none());
    
    assert_eq!(alice_entry.unwrap().amount, 1_000_000);
    assert_eq!(bob_entry.unwrap().amount, 2_000_000);
}
```

---

## Example 7: Client Usage (HTTP/JSON-RPC)

### Scenario
External application queries the contract via JSON-RPC.

### Request (using Soroban CLI or SDK)

```bash
# Using soroban-cli
soroban contract invoke \
  --id CONTRACT_ID \
  --source ACCOUNT \
  --network testnet \
  -- \
  get_ledger_vault \
  --depositor "GBBD47UZQ5..." \
  --deposit_id 0
```

### Response

```json
{
  "Ok": {
    "token": "GBUQWP3BOUZX34ULNQG23RQ6F4YUSXHTQGSH5H7P42SEMXBOJTHXSANK",
    "amount": "1000000000",
    "unlock_ledger": 1000000,
    "depositor": "GBBD47UZQ5XOTRBE5YNPF675EJXSSA5AHNQE6XBMQDOO2KDLVEDV5HH7",
    "penalty_bps": 500
  }
}
```

Or if deposit doesn't exist:

```json
{
  "Ok": null
}
```

---

## Example 8: Error Handling in Client Code

### Scenario
Application queries and handles missing deposits gracefully.

### Code

```rust
use soroban_sdk::Address;

fn check_deposit_status(
    vault_client: &VaultContractClient,
    depositor: &Address,
    deposit_id: u32,
) -> Result<String, String> {
    match vault_client.get_ledger_vault(depositor, &deposit_id) {
        Some(entry) => {
            let status = if current_ledger >= entry.unlock_ledger {
                "Ready to withdraw"
            } else {
                "Locked"
            };
            Ok(format!(
                "Deposit {}: {} tokens, {}, penalty: {}%",
                deposit_id,
                entry.amount,
                status,
                entry.penalty_bps / 100
            ))
        }
        None => Err(format!("Deposit {} not found", deposit_id)),
    }
}

// Usage:
match check_deposit_status(&vault, &alice, &0) {
    Ok(status) => println!("{}", status),
    Err(e) => eprintln!("Error: {}", e),
}
```

**Output (Success)**:
```
Deposit 0: 1000000 tokens, Locked, penalty: 5%
```

**Output (Not Found)**:
```
Error: Deposit 0 not found
```

---

## Key Takeaways

✅ **Always check `Option` result** — Deposit may not exist

✅ **Use `get_deposit_type()` first** — Determines which query to call (ledger vs. time)

✅ **Field values are immutable** — Once stored, values don't change until withdrawal/cancellation

✅ **Separate storage namespaces** — Time-based and ledger-based never conflict

✅ **No TTL concern** — Soroban TTL is handled internally (deposits never silently expire)

✅ **Gas efficient** — O(1) persistent storage lookup

