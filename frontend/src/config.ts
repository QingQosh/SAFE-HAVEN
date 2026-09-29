// ============================================================
//  App Configuration
//  Update CONTRACT_ID and NETWORK after deploying your contract
// ============================================================

// Validate critical env vars at startup so misconfigured deployments fail fast
// rather than silently targeting the wrong network (#13).
if (!import.meta.env.VITE_NETWORK_PASSPHRASE) {
  throw new Error(
    '[SAFE-HAVEN] VITE_NETWORK_PASSPHRASE is not set. ' +
    'Add it to your .env file (e.g. "Test SDF Network ; September 2015" for testnet ' +
    'or "Public Global Stellar Network ; September 2015" for mainnet). ' +
    'Refusing to start to prevent transactions from being signed for the wrong network.'
  )
}

export const CONFIG = {
  /** Deployed contract ID — set via VITE_CONTRACT_ID env var or paste here */
  CONTRACT_ID: import.meta.env.VITE_CONTRACT_ID as string ?? 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4',

  /** Stellar network passphrase */
  NETWORK_PASSPHRASE: import.meta.env.VITE_NETWORK_PASSPHRASE as string,

  /** Horizon / Soroban RPC endpoint */
  RPC_URL: (import.meta.env.VITE_RPC_URL as string) ??
    'https://soroban-testnet.stellar.org',

  /** Horizon endpoint (for account info) */
  HORIZON_URL: (import.meta.env.VITE_HORIZON_URL as string) ??
    'https://horizon-testnet.stellar.org',

  /** Explorer base URL for transactions */
  EXPLORER_URL: (import.meta.env.VITE_EXPLORER_URL as string) ??
    'https://stellar.expert/explorer/testnet',

  /** Native XLM token contract on testnet */
  NATIVE_TOKEN: 'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC',

  /** Contract constants (mirror of Rust compile-time defaults) */
  MAX_DEPOSIT_AMOUNT: 1_000_000_000_000_000n,
  MAX_LOCK_DURATION_SECS: 157_788_000,
  MIN_LOCK_DURATION_SECS: 60,
  MAX_PENALTY_BPS: 10_000,

  /** Stroops per XLM */
  STROOPS_PER_XLM: 10_000_000,

  // ============================================================
  //  Fiat On-Ramp Configuration (Ramp Network)
  // ============================================================

  /** Ramp Network API key for embedded widget */
  RAMP_API_KEY: (import.meta.env.VITE_RAMP_API_KEY as string) ?? '',

  /** Ramp environment: "production" or "staging" (default: staging for testing) */
  RAMP_ENVIRONMENT: (import.meta.env.VITE_RAMP_ENVIRONMENT as 'production' | 'staging') ?? 'staging',

  /** Whether Ramp on-ramp is enabled (requires API key) */
  RAMP_ENABLED: !!(import.meta.env.VITE_RAMP_API_KEY as string),

  // ============================================================
  //  Version Pinning (#399)
  // ============================================================

  /** Expected contract version this frontend is pinned to (semver string) */
  EXPECTED_CONTRACT_VERSION: (import.meta.env.VITE_EXPECTED_CONTRACT_VERSION as string) ?? '0.1.0',
  /** Expected contract storage schema version */
  EXPECTED_STORAGE_VERSION: Number(import.meta.env.VITE_EXPECTED_STORAGE_VERSION ?? 1),
} as const

// ============================================================
//  Config Validation (#84)
// ============================================================

interface ConfigError {
  field: string
  message: string
  fix: string
}

/**
 * Validates all required config values at startup.
 * Returns an array of errors if validation fails.
 * Call this from main.tsx before rendering the app.
 */
export function validateConfig(): ConfigError[] {
  const errors: ConfigError[] = []

  // Validate CONTRACT_ID: must be a valid C-address (56 chars, starts with C)
  if (!CONFIG.CONTRACT_ID || CONFIG.CONTRACT_ID === 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4') {
    errors.push({
      field: 'VITE_CONTRACT_ID',
      message: 'Contract ID is not set or is using the placeholder value',
      fix: 'Set VITE_CONTRACT_ID in your .env file to your deployed contract address (starts with C, 56 characters)'
    })
  } else if (!/^C[A-Z2-7]{55}$/.test(CONFIG.CONTRACT_ID)) {
    errors.push({
      field: 'VITE_CONTRACT_ID',
      message: `Invalid contract ID format: "${CONFIG.CONTRACT_ID}"`,
      fix: 'Contract ID must start with "C" and be exactly 56 characters (uppercase letters and numbers 2-7)'
    })
  }

  // Validate NETWORK_PASSPHRASE: must be set (already checked above, but be explicit)
  if (!CONFIG.NETWORK_PASSPHRASE) {
    errors.push({
      field: 'VITE_NETWORK_PASSPHRASE',
      message: 'Network passphrase is not set',
      fix: 'Set VITE_NETWORK_PASSPHRASE in your .env file (e.g., "Test SDF Network ; September 2015" for testnet)'
    })
  }

  // Validate RPC_URL: must be a valid HTTP(S) URL
  if (!CONFIG.RPC_URL) {
    errors.push({
      field: 'VITE_RPC_URL',
      message: 'RPC URL is not set',
      fix: 'Set VITE_RPC_URL in your .env file (e.g., "https://soroban-testnet.stellar.org")'
    })
  } else if (!/^https?:\/\/.+/.test(CONFIG.RPC_URL)) {
    errors.push({
      field: 'VITE_RPC_URL',
      message: `Invalid RPC URL format: "${CONFIG.RPC_URL}"`,
      fix: 'RPC URL must start with http:// or https://'
    })
  }

  // Validate HORIZON_URL: must be a valid HTTP(S) URL
  if (!CONFIG.HORIZON_URL) {
    errors.push({
      field: 'VITE_HORIZON_URL',
      message: 'Horizon URL is not set',
      fix: 'Set VITE_HORIZON_URL in your .env file (e.g., "https://horizon-testnet.stellar.org")'
    })
  } else if (!/^https?:\/\/.+/.test(CONFIG.HORIZON_URL)) {
    errors.push({
      field: 'VITE_HORIZON_URL',
      message: `Invalid Horizon URL format: "${CONFIG.HORIZON_URL}"`,
      fix: 'Horizon URL must start with http:// or https://'
    })
  }

  // Validate EXPLORER_URL: must be a valid HTTP(S) URL
  if (!CONFIG.EXPLORER_URL) {
    errors.push({
      field: 'VITE_EXPLORER_URL',
      message: 'Explorer URL is not set',
      fix: 'Set VITE_EXPLORER_URL in your .env file (e.g., "https://stellar.expert/explorer/testnet")'
    })
  } else if (!/^https?:\/\/.+/.test(CONFIG.EXPLORER_URL)) {
    errors.push({
      field: 'VITE_EXPLORER_URL',
      message: `Invalid Explorer URL format: "${CONFIG.EXPLORER_URL}"`,
      fix: 'Explorer URL must start with http:// or https://'
    })
  }

  // Validate NATIVE_TOKEN: must be a valid C-address
  if (!CONFIG.NATIVE_TOKEN || !/^C[A-Z2-7]{55}$/.test(CONFIG.NATIVE_TOKEN)) {
    errors.push({
      field: 'NATIVE_TOKEN',
      message: `Invalid native token address: "${CONFIG.NATIVE_TOKEN}"`,
      fix: 'Native token address must be a valid Stellar contract address (starts with C, 56 characters)'
    })
  }

  // Validate EXPECTED_CONTRACT_VERSION: if set, must be a valid semver string (#399)
  if (
    CONFIG.EXPECTED_CONTRACT_VERSION &&
    !/^\d+\.\d+\.\d+$/.test(CONFIG.EXPECTED_CONTRACT_VERSION)
  ) {
    errors.push({
      field: 'VITE_EXPECTED_CONTRACT_VERSION',
      message: `Invalid semver format: "${CONFIG.EXPECTED_CONTRACT_VERSION}"`,
      fix: 'VITE_EXPECTED_CONTRACT_VERSION must follow the semver format MAJOR.MINOR.PATCH (e.g. "0.1.0")'
    })
  }

  return errors
}
