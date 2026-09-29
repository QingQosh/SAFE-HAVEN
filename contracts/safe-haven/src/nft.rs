// ============================================================
//  NFT Evolution System
//  Dynamic metadata generation and evolution tracking
// ============================================================

use soroban_sdk::{contracttype, String};

/// Rarity tier for evolved NFTs — affects visual traits and metadata attributes.
/// Higher tiers unlock more impressive evolution artwork.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum RarityTier {
    /// Common: deposits < 1000 units or locked < 7 days
    Common = 0,
    /// Uncommon: 1000–10000 units or locked 7–30 days
    Uncommon = 1,
    /// Rare: 10000–100000 units or locked 30–90 days
    Rare = 2,
    /// Epic: 100000–1000000 units or locked 90–365 days
    Epic = 3,
    /// Legendary: > 1000000 units or locked > 365 days
    Legendary = 4,
}

/// Evolution stage — deposit lifecycle expressed as a visual stage.
/// Each stage triggers automatic metadata updates.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum EvolutionStage {
    /// New deposit (0–14 days old)
    Egg = 0,
    /// Adolescent (14–60 days old)
    Hatchling = 1,
    /// Young adult (60–180 days old)
    Juvenile = 2,
    /// Mature (180–365 days old)
    Adult = 3,
    /// Ancient (365+ days old)
    Ancient = 4,
}

/// NFT metadata artifact — immutable record of evolution history on-chain.
/// Stored alongside the deposit and updated each time the NFT evolves.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NFTEvolutionRecord {
    /// Deposit ID tied to this NFT
    pub deposit_id: u32,
    /// Current evolution stage
    pub stage: EvolutionStage,
    /// Current rarity tier (based on amount and duration)
    pub rarity: RarityTier,
    /// Timestamp when this NFT was created (first deposit)
    pub created_at: u64,
    /// Timestamp of the most recent evolution (stage change)
    pub last_evolved_at: u64,
    /// Number of times this NFT has evolved (stage transitions)
    pub evolution_count: u32,
    /// Total value locked across all updates (in unit basis)
    pub total_value_locked: i128,
    /// Current deposit amount (snapshot at last update)
    pub current_amount: i128,
    /// Unlock timestamp (wall-clock or ledger-based marker)
    pub unlock_time: u64,
    /// True if this NFT has achieved carbon neutrality
    pub carbon_neutral: bool,
    /// True if this NFT has hit the high-renewable milestone
    pub high_renewable: bool,
    /// True if this NFT has achieved carbon-negative status
    pub carbon_negative: bool,
    /// Serialized metadata JSON string (UTF-8).
    /// Example: `{"name":"Vault #42","description":"...","attributes":[...]}`
    pub metadata_uri: String,
}

/// Summary of NFT traits for metadata generation.
/// Used internally to render the metadata_uri field.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NFTTraits {
    /// Deposit age in days (for evolution stage display)
    pub age_days: u32,
    /// Percentage of lock duration elapsed (0–100)
    pub maturity_percent: u32,
    /// Base visual theme (e.g., "seed", "sprout", "plant", "tree", "ancient_tree")
    pub theme: String,
    /// Color palette variant based on rarity (e.g., "gold", "purple", "blue", "green", "common")
    pub color_variant: String,
    /// Multiplier for glow intensity (1–5, higher = more evolved)
    pub glow_intensity: u32,
}

/// Event trigger for NFT evolution — emitted whenever a deposit evolves to a new stage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NFTEvolvedEvent {
    pub deposit_id: u32,
    pub old_stage: EvolutionStage,
    pub new_stage: EvolutionStage,
    pub rarity: RarityTier,
    pub evolution_count: u32,
    pub timestamp: u64,
}

// ================================================================
//  NFT Constants
// ================================================================

/// Thresholds for rarity tier assignment (in units — base amounts)
pub const RARITY_UNCOMMON_THRESHOLD: i128 = 1_000;
pub const RARITY_RARE_THRESHOLD: i128 = 10_000;
pub const RARITY_EPIC_THRESHOLD: i128 = 100_000;
pub const RARITY_LEGENDARY_THRESHOLD: i128 = 1_000_000;

/// Duration thresholds for rarity assignment (in seconds)
pub const RARITY_UNCOMMON_DAYS: u64 = 7 * 24 * 60 * 60; // 7 days
pub const RARITY_RARE_DAYS: u64 = 30 * 24 * 60 * 60; // 30 days
pub const RARITY_EPIC_DAYS: u64 = 90 * 24 * 60 * 60; // 90 days
pub const RARITY_LEGENDARY_DAYS: u64 = 365 * 24 * 60 * 60; // 365 days

/// Stage transition thresholds (deposit age in seconds)
pub const STAGE_HATCHLING_SECS: u64 = 14 * 24 * 60 * 60; // 14 days
pub const STAGE_JUVENILE_SECS: u64 = 60 * 24 * 60 * 60; // 60 days
pub const STAGE_ADULT_SECS: u64 = 180 * 24 * 60 * 60; // 180 days
pub const STAGE_ANCIENT_SECS: u64 = 365 * 24 * 60 * 60; // 365 days

/// NFT storage key variant — maps (depositor_address, deposit_id) to NFTEvolutionRecord
/// Added to VaultKey enum and used in storage module
pub const NFT_EVOLUTION_KEY_PREFIX: &str = "nft_evo";

// ================================================================
//  Rarity Calculation
// ================================================================

/// Determine rarity tier based on deposit amount and lock duration.
/// Rarity is determined by the maximum tier matched by either criteria.
///
/// # Arguments
/// * `amount` - Total amount locked
/// * `lock_duration_secs` - Time until unlock (or elapsed time if already unlocked)
///
/// # Returns
/// The highest applicable rarity tier
pub fn calculate_rarity(amount: i128, lock_duration_secs: u64) -> RarityTier {
    // Check amount-based tiers (descending order for max match)
    if amount >= RARITY_LEGENDARY_THRESHOLD {
        return RarityTier::Legendary;
    }
    if amount >= RARITY_EPIC_THRESHOLD {
        return RarityTier::Epic;
    }
    if amount >= RARITY_RARE_THRESHOLD {
        return RarityTier::Rare;
    }
    if amount >= RARITY_UNCOMMON_THRESHOLD {
        return RarityTier::Uncommon;
    }

    // Check duration-based tiers (descending order for max match)
    if lock_duration_secs >= RARITY_LEGENDARY_DAYS {
        return RarityTier::Legendary;
    }
    if lock_duration_secs >= RARITY_EPIC_DAYS {
        return RarityTier::Epic;
    }
    if lock_duration_secs >= RARITY_RARE_DAYS {
        return RarityTier::Rare;
    }
    if lock_duration_secs >= RARITY_UNCOMMON_DAYS {
        return RarityTier::Uncommon;
    }

    RarityTier::Common
}

/// Determine evolution stage based on deposit age.
///
/// # Arguments
/// * `age_secs` - Time elapsed since deposit creation (current_time - created_at)
///
/// # Returns
/// The appropriate evolution stage for the deposit's age
pub fn calculate_evolution_stage(age_secs: u64) -> EvolutionStage {
    if age_secs >= STAGE_ANCIENT_SECS {
        EvolutionStage::Ancient
    } else if age_secs >= STAGE_ADULT_SECS {
        EvolutionStage::Adult
    } else if age_secs >= STAGE_JUVENILE_SECS {
        EvolutionStage::Juvenile
    } else if age_secs >= STAGE_HATCHLING_SECS {
        EvolutionStage::Hatchling
    } else {
        EvolutionStage::Egg
    }
}

/// Generate color variant based on rarity tier.
/// Used for metadata traits to visually distinguish rarity levels.
pub fn rarity_to_color_variant(rarity: &RarityTier) -> String {
    match rarity {
        RarityTier::Common => String::from_slice(&soroban_sdk::Env::new(), "gray"),
        RarityTier::Uncommon => String::from_slice(&soroban_sdk::Env::new(), "green"),
        RarityTier::Rare => String::from_slice(&soroban_sdk::Env::new(), "blue"),
        RarityTier::Epic => String::from_slice(&soroban_sdk::Env::new(), "purple"),
        RarityTier::Legendary => String::from_slice(&soroban_sdk::Env::new(), "gold"),
    }
}

/// Generate visual theme based on evolution stage.
/// Represents the visual progression from seed → sprout → plant → tree → ancient_tree.
pub fn stage_to_theme(stage: &EvolutionStage) -> String {
    match stage {
        EvolutionStage::Egg => String::from_slice(&soroban_sdk::Env::new(), "seed"),
        EvolutionStage::Hatchling => String::from_slice(&soroban_sdk::Env::new(), "sprout"),
        EvolutionStage::Juvenile => String::from_slice(&soroban_sdk::Env::new(), "plant"),
        EvolutionStage::Adult => String::from_slice(&soroban_sdk::Env::new(), "tree"),
        EvolutionStage::Ancient => String::from_slice(&soroban_sdk::Env::new(), "ancient_tree"),
    }
}

/// Calculate glow intensity based on evolution stage.
/// Higher stages produce more intense glow effects.
pub fn stage_to_glow_intensity(stage: &EvolutionStage) -> u32 {
    match stage {
        EvolutionStage::Egg => 1,
        EvolutionStage::Hatchling => 2,
        EvolutionStage::Juvenile => 3,
        EvolutionStage::Adult => 4,
        EvolutionStage::Ancient => 5,
    }
}

// ================================================================
//  Metadata Generation
// ================================================================

/// Build NFT traits for metadata generation.
/// Assembles visual properties based on rarity, evolution stage, and deposit age.
pub fn build_nft_traits(
    rarity: &RarityTier,
    stage: &EvolutionStage,
    age_secs: u64,
    lock_duration_secs: u64,
) -> NFTTraits {
    let age_days = (age_secs / (24 * 60 * 60)) as u32;
    let maturity_percent = if lock_duration_secs == 0 {
        0
    } else {
        ((age_secs as u128 * 100) / lock_duration_secs as u128).min(100) as u32
    };

    NFTTraits {
        age_days,
        maturity_percent,
        theme: stage_to_theme(stage),
        color_variant: rarity_to_color_variant(rarity),
        glow_intensity: stage_to_glow_intensity(stage),
    }
}

/// Generate a JSON metadata URI string for the NFT.
/// This is a simple template — in production, this could be extended to include
/// full on-chain metadata or a reference to an off-chain metadata service.
pub fn generate_metadata_uri(
    env: &soroban_sdk::Env,
    deposit_id: u32,
    rarity: &RarityTier,
    stage: &EvolutionStage,
    traits: &NFTTraits,
    amount: i128,
    maturity_percent: u32,
) -> String {
    // Simple JSON-like format (not full JSON due to Soroban String constraints)
    // Format: {name:Vault #{id},stage:{stage},rarity:{rarity},theme:{theme},color:{color},glow:{glow},amount:{amount},maturity:{pct}%}
    let stage_name = match stage {
        EvolutionStage::Egg => "Egg",
        EvolutionStage::Hatchling => "Hatchling",
        EvolutionStage::Juvenile => "Juvenile",
        EvolutionStage::Adult => "Adult",
        EvolutionStage::Ancient => "Ancient",
    };

    let rarity_name = match rarity {
        RarityTier::Common => "Common",
        RarityTier::Uncommon => "Uncommon",
        RarityTier::Rare => "Rare",
        RarityTier::Epic => "Epic",
        RarityTier::Legendary => "Legendary",
    };

    // Build a compact metadata string
    let mut metadata = String::from_slice(env, "deposit_nft#");
    // Append deposit ID as decimal
    let id_str = if deposit_id == 0 {
        "0"
    } else {
        // Simple decimal conversion (Soroban limitation: no format! macro)
        "42" // placeholder for now; will be improved in real implementation
    };
    metadata = String::from_slice(env, &format!("deposit_nft#{}_{}_{}_{}_{}", 
        deposit_id, stage_name, rarity_name, traits.glow_intensity, maturity_percent));
    
    metadata
}

/// Create a new NFT evolution record for a deposit.
/// Called when a new deposit is created to initialize the NFT.
pub fn create_evolution_record(
    env: &soroban_sdk::Env,
    deposit_id: u32,
    amount: i128,
    unlock_time: u64,
    now: u64,
) -> NFTEvolutionRecord {
    let age_secs = 0; // Brand new deposit
    let lock_duration_secs = unlock_time.saturating_sub(now);
    
    let rarity = calculate_rarity(amount, lock_duration_secs);
    let stage = EvolutionStage::Egg;
    let traits = build_nft_traits(&rarity, &stage, age_secs, lock_duration_secs);
    let metadata_uri = generate_metadata_uri(
        env,
        deposit_id,
        &rarity,
        &stage,
        &traits,
        amount,
        0,
    );

    NFTEvolutionRecord {
        deposit_id,
        stage,
        rarity,
        created_at: now,
        last_evolved_at: now,
        evolution_count: 0,
        total_value_locked: amount,
        current_amount: amount,
        unlock_time,
        carbon_neutral: false,
        high_renewable: false,
        carbon_negative: false,
        metadata_uri,
    }
}

/// Check if an NFT should evolve and return the new stage (if any).
/// Returns None if no evolution is needed, otherwise returns the new stage.
pub fn check_evolution(
    record: &NFTEvolutionRecord,
    now: u64,
) -> Option<EvolutionStage> {
    let age_secs = now.saturating_sub(record.created_at);
    let new_stage = calculate_evolution_stage(age_secs);
    
    if new_stage > record.stage {
        Some(new_stage)
    } else {
        None
    }
}

/// Update an NFT evolution record with new metadata.
/// Called when the NFT evolves to a new stage.
pub fn evolve_nft(
    env: &soroban_sdk::Env,
    record: &mut NFTEvolutionRecord,
    new_stage: EvolutionStage,
    new_amount: i128,
    now: u64,
) {
    let lock_duration_secs = record.unlock_time.saturating_sub(record.created_at);
    let age_secs = now.saturating_sub(record.created_at);
    
    // Update rarity if amount changed
    let new_rarity = calculate_rarity(new_amount, lock_duration_secs);
    
    // Update the record
    record.stage = new_stage;
    record.rarity = new_rarity;
    record.last_evolved_at = now;
    record.evolution_count = record.evolution_count.saturating_add(1);
    record.current_amount = new_amount;
    
    // Regenerate metadata
    let traits = build_nft_traits(&new_rarity, &new_stage, age_secs, lock_duration_secs);
    let maturity_percent = if lock_duration_secs == 0 {
        0
    } else {
        ((age_secs as u128 * 100) / lock_duration_secs as u128).min(100) as u32
    };
    
    record.metadata_uri = generate_metadata_uri(
        env,
        record.deposit_id,
        &new_rarity,
        &new_stage,
        &traits,
        new_amount,
        maturity_percent,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_rarity_by_amount() {
        assert_eq!(calculate_rarity(500, 0), RarityTier::Common);
        assert_eq!(calculate_rarity(1_000, 0), RarityTier::Uncommon);
        assert_eq!(calculate_rarity(10_000, 0), RarityTier::Rare);
        assert_eq!(calculate_rarity(100_000, 0), RarityTier::Epic);
        assert_eq!(calculate_rarity(1_000_000, 0), RarityTier::Legendary);
    }

    #[test]
    fn test_calculate_rarity_by_duration() {
        assert_eq!(calculate_rarity(0, 6 * 24 * 60 * 60), RarityTier::Common);
        assert_eq!(
            calculate_rarity(0, 7 * 24 * 60 * 60),
            RarityTier::Uncommon
        );
        assert_eq!(
            calculate_rarity(0, 30 * 24 * 60 * 60),
            RarityTier::Rare
        );
        assert_eq!(
            calculate_rarity(0, 90 * 24 * 60 * 60),
            RarityTier::Epic
        );
        assert_eq!(
            calculate_rarity(0, 365 * 24 * 60 * 60),
            RarityTier::Legendary
        );
    }

    #[test]
    fn test_calculate_evolution_stage() {
        assert_eq!(calculate_evolution_stage(0), EvolutionStage::Egg);
        assert_eq!(
            calculate_evolution_stage(14 * 24 * 60 * 60),
            EvolutionStage::Hatchling
        );
        assert_eq!(
            calculate_evolution_stage(60 * 24 * 60 * 60),
            EvolutionStage::Juvenile
        );
        assert_eq!(
            calculate_evolution_stage(180 * 24 * 60 * 60),
            EvolutionStage::Adult
        );
        assert_eq!(
            calculate_evolution_stage(365 * 24 * 60 * 60),
            EvolutionStage::Ancient
        );
    }
}
