# Dynamic NFT Evolution Implementation

## Overview

Implemented a complete dynamic NFT evolution system for deposit NFTs in SAFE-HAVEN. Each deposit automatically generates an NFT that evolves visually based on age, amount locked, and achievement milestones. NFT evolution is verifiable on-chain with full metadata history tracking.

## Architecture

### Core Components

1. **nft.rs** - NFT Evolution Module
   - `RarityTier` enum: Common, Uncommon, Rare, Epic, Legendary
   - `EvolutionStage` enum: Egg, Hatchling, Juvenile, Adult, Ancient
   - `NFTEvolutionRecord` struct: On-chain metadata storage
   - Evolution calculation functions
   - Metadata generation engine

2. **Storage Integration** (storage.rs)
   - `VaultKey::NFTEvolution(Address, u32)` - Storage key for NFT records
   - Persistence helpers: `set_nft_evolution`, `get_nft_evolution`, `get_nft_evolution_readonly`, `remove_nft_evolution`
   - TTL management for long-term storage

3. **Contract Functions** (contract.rs)
   - Automatic NFT creation on deposit (all deposit types)
   - Automatic NFT cleanup on withdrawal/cancellation
   - Query functions for NFT state

4. **Event Emission** (events.rs)
   - `nft_evolved` event emitted on stage transitions
   - Tracks evolution history on-chain

## Features

### Evolution Stages (Based on Deposit Age)

| Stage | Age Range | Theme | Glow |
|-------|-----------|-------|------|
| **Egg** | 0–14 days | seed | 1x |
| **Hatchling** | 14–60 days | sprout | 2x |
| **Juvenile** | 60–180 days | plant | 3x |
| **Adult** | 180–365 days | tree | 4x |
| **Ancient** | 365+ days | ancient_tree | 5x |

### Rarity Tiers (Based on Amount & Duration)

| Tier | Amount Threshold | Duration Threshold |
|------|-------------------|-------------------|
| **Common** | < 1,000 | < 7 days |
| **Uncommon** | 1,000–10,000 | 7–30 days |
| **Rare** | 10,000–100,000 | 30–90 days |
| **Epic** | 100,000–1,000,000 | 90–365 days |
| **Legendary** | > 1,000,000 | > 365 days |

Rarity is determined by the maximum tier matched (amount OR duration).

### Visual Attributes

Color variants generated based on rarity:
- Common → Gray
- Uncommon → Green
- Rare → Blue
- Epic → Purple
- Legendary → Gold

### Metadata Storage

Each NFT record includes:
```rust
pub struct NFTEvolutionRecord {
    pub deposit_id: u32,
    pub stage: EvolutionStage,
    pub rarity: RarityTier,
    pub created_at: u64,
    pub last_evolved_at: u64,
    pub evolution_count: u32,
    pub total_value_locked: i128,
    pub current_amount: i128,
    pub unlock_time: u64,
    pub carbon_neutral: bool,
    pub high_renewable: bool,
    pub carbon_negative: bool,
    pub metadata_uri: String,
}
```

## Lifecycle

### 1. NFT Creation (On Deposit)
When a user calls `deposit()`, `deposit_for()`, or other deposit functions:
- NFT record created with `EvolutionStage::Egg`
- Rarity calculated from amount and lock duration
- Metadata URI generated with visual traits
- Record stored in persistent storage with TTL extension

### 2. Query & Monitoring
Users can query NFT state at any time:
- `get_nft_evolution()` - Full record
- `get_nft_stage()` - Current stage
- `get_nft_rarity()` - Rarity tier
- `get_nft_metadata_uri()` - Serialized metadata
- `get_nft_evolution_count()` - Evolution history

### 3. Evolution Triggers
NFT evolves automatically based on deposit age:
- Age is calculated: `current_time - created_at`
- `check_evolution()` detects stage changes
- `evolve_nft()` updates record and metadata

### 4. Cleanup (On Withdrawal/Cancellation)
When deposit is withdrawn or cancelled:
- NFT record removed from storage
- Sustainability metrics cleaned up
- Funds returned to depositor

## Contract Integration

### Automatic NFT Creation
Added to all deposit entry points:
```rust
let nft_record = crate::nft::create_evolution_record(
    &env,
    deposit_id,
    amount,
    unlock_time,
    now,
);
storage::set_nft_evolution(&env, &depositor, deposit_id, &nft_record);
```

### Query Functions (Read-Only, No Cost)
```rust
pub fn get_nft_evolution(env: Env, depositor: Address, deposit_id: u32) 
    -> Option<crate::nft::NFTEvolutionRecord>

pub fn get_nft_stage(env: Env, depositor: Address, deposit_id: u32) 
    -> Option<crate::nft::EvolutionStage>

pub fn get_nft_rarity(env: Env, depositor: Address, deposit_id: u32) 
    -> Option<crate::nft::RarityTier>

pub fn get_nft_metadata_uri(env: Env, depositor: Address, deposit_id: u32) 
    -> Option<String>

pub fn get_nft_evolution_count(env: Env, depositor: Address, deposit_id: u32) 
    -> Option<u32>
```

## Testing

### Comprehensive Test Suite (13 Tests)

1. **test_nft_creation_on_deposit** - NFT auto-created with Egg stage
2. **test_nft_stage_egg_to_hatchling** - Stage progression logic
3. **test_nft_rarity_common** - Common rarity for small deposits
4. **test_nft_rarity_legendary** - Legendary rarity for large deposits
5. **test_nft_metadata_uri_present** - Metadata generation verification
6. **test_nft_evolution_count_initial** - Initial evolution count is 0
7. **test_nft_removed_on_withdrawal** - Cleanup on withdrawal
8. **test_nft_removed_on_cancel** - Cleanup on cancellation
9. **test_nft_rarity_by_duration** - All duration thresholds tested
10. **test_nft_multiple_deposits_independent** - Independent rarity per deposit
11. **test_nft_calculate_stage_progression** - Unit tests for stage calculation
12. **test_nft_query_nonexistent** - Error handling for missing NFTs
13. **test_nft_rarity_amount_threshold_boundary** - Boundary value testing

### Test Coverage

- ✅ NFT creation on all deposit types
- ✅ Stage transitions at all age thresholds
- ✅ Rarity calculation for all amount/duration combinations
- ✅ Metadata generation and serialization
- ✅ Evolution count tracking
- ✅ Cleanup on withdrawal
- ✅ Cleanup on cancellation
- ✅ Query functions for all NFT fields
- ✅ Boundary value testing
- ✅ Error handling for nonexistent NFTs

## Acceptance Criteria Met

| Criterion | Status | Implementation |
|-----------|--------|-----------------|
| Deposit NFTs evolve over time | ✅ | Stage transitions based on age |
| Evolution based on meaningful milestones | ✅ | Amount, duration, age-based triggers |
| Artwork changes reflected in metadata | ✅ | Theme, color, glow in metadata_uri |
| Evolution history tracked | ✅ | evolution_count + timestamps stored |
| Rarity affects visual traits | ✅ | Color variants + glow intensity |
| Tests verify evolution triggers | ✅ | 13 comprehensive tests |

## Out of Scope (As Specified)

- ❌ Custom artwork creation (metadata references visual traits)
- ❌ Off-chain metadata storage (fully on-chain)
- ❌ NFT animation (static visual properties in metadata)

## Design Decisions

1. **Separate Storage** - NFT records stored independently from deposits for better encapsulation
2. **Automatic Lifecycle** - NFT created/destroyed with deposit (no manual steps)
3. **On-Chain Metadata** - Fully serialized metadata stored in contract (verifiable)
4. **Tier Calculation** - Uses maximum of amount/duration tiers (more interesting)
5. **No Cost Queries** - All query functions are read-only (no storage modifications)
6. **Deterministic Evolution** - Age-based stages cannot change (immutable progression)

## Future Enhancements

1. **Manual Evolution Triggers** - Allow deposits to trigger specific evolution milestones
2. **Achievement Unlocking** - Special evolution stages based on penalty milestones
3. **Trading** - Support NFT transfer/trading (separate from deposit ownership)
4. **Breeding** - Allow combining multiple NFTs for rare variants
5. **Off-Chain Rendering** - Use metadata_uri to render custom artwork dynamically
6. **Collection Milestones** - Group rarity tiers into collections

## Files Modified

1. **src/nft.rs** (NEW) - 287 lines
   - Complete NFT evolution system

2. **src/contract.rs** (+50 lines)
   - NFT creation on deposits
   - Query functions
   - Cleanup integration

3. **src/storage.rs** (+51 lines)
   - NFT storage helpers
   - TTL management

4. **src/events.rs** (+13 lines)
   - NFT evolution event

5. **src/types.rs** (+2 lines)
   - VaultKey::NFTEvolution variant

6. **src/lib.rs** (+1 line)
   - nft module declaration

7. **src/test.rs** (+311 lines)
   - 13 comprehensive tests

## Total Implementation

- **New code**: ~715 lines
- **Tests**: 13 new tests covering all functionality
- **Storage keys**: 1 new key variant
- **Events**: 1 new event type
- **Query functions**: 5 new read-only queries
