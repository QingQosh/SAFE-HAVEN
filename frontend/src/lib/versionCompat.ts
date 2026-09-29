// ============================================================
//  Version Compatibility Module
//
//  Defines the frontend/contract version pinning matrix and
//  provides a pure function to check compatibility at runtime.
//  Used by useVersionCheck() hook and VersionWarningBanner.
// ============================================================

/** Semver string for this frontend build. */
export const FRONTEND_VERSION = '0.1.0'

/** Contract semver version this frontend was built against. */
export const EXPECTED_CONTRACT_VERSION = '0.1.0'

/** Storage schema version this frontend understands. */
export const EXPECTED_STORAGE_VERSION = 1

// ----------------------------------------------------------------
//  Compatibility matrix
// ----------------------------------------------------------------

export interface CompatibilityRow {
  /** The frontend version this row describes. */
  frontendVersion: string
  /** Contract semver versions compatible with this frontend version. */
  compatibleContractVersions: string[]
  /** Storage schema versions compatible with this frontend version. */
  compatibleStorageVersions: number[]
}

/**
 * Compatibility matrix — add a new row whenever a breaking contract change
 * is released. Each row declares exactly which contract/storage versions a
 * given frontend build understands.
 *
 * Rules:
 *  - A frontend version may be compatible with more than one contract version
 *    (e.g. backward-compatible patch releases).
 *  - A mismatch on storage version is always treated as an error because
 *    on-chain data layout has changed.
 *  - A mismatch on contract version (but matching storage) is a warning —
 *    the UI may lack support for new features but existing data is readable.
 */
export const COMPATIBILITY_MATRIX: CompatibilityRow[] = [
  {
    frontendVersion: '0.1.0',
    compatibleContractVersions: ['0.1.0'],
    compatibleStorageVersions: [1],
  },
  {
    // Placeholder for the next release — update when 0.2.0 ships.
    // compatibleContractVersions includes 0.1.0 to allow a graceful
    // rolling-upgrade window where old contracts are still readable.
    frontendVersion: '0.2.0',
    compatibleContractVersions: ['0.1.0', '0.2.0'],
    compatibleStorageVersions: [1, 2],
  },
]

// ----------------------------------------------------------------
//  VersionCheckResult
// ----------------------------------------------------------------

export interface VersionCheckResult {
  /** True if all versions match the expected values. */
  compatible: boolean
  /** Semver string reported by the live contract, or null if unreachable. */
  contractVersion: string | null
  /** Storage schema version reported by the live contract, or null. */
  storageVersion: number | null
  /** Human-readable explanation of any incompatibility, or null when ok. */
  warning: string | null
  /** Severity level for banner rendering. */
  severity: 'ok' | 'warn' | 'error'
}

// ----------------------------------------------------------------
//  checkVersionCompatibility
// ----------------------------------------------------------------

/**
 * Pure function. Compares the live contract/storage versions against the
 * compatibility matrix and returns a VersionCheckResult.
 *
 * Degradation rules:
 *  - If both versions are null (network unreachable) → warn, not error.
 *  - If storage version is incompatible → error (data may be unreadable).
 *  - If only contract version is incompatible → warn (features may differ).
 *  - If both match the matrix for this frontend version → ok.
 */
export function checkVersionCompatibility(
  contractVersion: string | null,
  storageVersion: number | null,
): VersionCheckResult {
  // Graceful degradation: if we can't reach the contract at all, warn but
  // don't crash — the user might still be able to read cached state.
  if (contractVersion === null && storageVersion === null) {
    return {
      compatible: false,
      contractVersion: null,
      storageVersion: null,
      warning:
        'Could not retrieve contract version information. ' +
        'The contract may be unreachable or not yet initialized. ' +
        'Some features may not work correctly until the connection is restored.',
      severity: 'warn',
    }
  }

  // Find the row for the current frontend version.
  const row = COMPATIBILITY_MATRIX.find((r) => r.frontendVersion === FRONTEND_VERSION)

  // If contractVersion came back as 'unknown' (stellar.ts fallback), treat
  // as unreachable — same as null.
  const effectiveContractVersion =
    contractVersion === 'unknown' ? null : contractVersion

  const contractOk =
    effectiveContractVersion === null ||
    (row?.compatibleContractVersions.includes(effectiveContractVersion) ?? false)

  const storageOk =
    storageVersion === null ||
    (row?.compatibleStorageVersions.includes(storageVersion) ?? false)

  if (contractOk && storageOk) {
    return {
      compatible: true,
      contractVersion: effectiveContractVersion,
      storageVersion,
      warning: null,
      severity: 'ok',
    }
  }

  // Storage mismatch is the more severe case.
  if (!storageOk) {
    return {
      compatible: false,
      contractVersion: effectiveContractVersion,
      storageVersion,
      warning:
        `Storage schema mismatch: this frontend expects storage version ` +
        `${EXPECTED_STORAGE_VERSION} but the contract reports version ${storageVersion ?? 'unknown'}. ` +
        `The on-chain data layout may have changed. ` +
        `Please upgrade this frontend to a version compatible with storage v${storageVersion ?? '?'}, ` +
        `or contact the contract operator.`,
      severity: 'error',
    }
  }

  // Contract version mismatch only — warn, don't block.
  return {
    compatible: false,
    contractVersion: effectiveContractVersion,
    storageVersion,
    warning:
      `Contract version mismatch: this frontend is pinned to contract v${EXPECTED_CONTRACT_VERSION} ` +
      `but the deployed contract reports v${effectiveContractVersion ?? 'unknown'}. ` +
      `Most features should still work, but some functionality may be missing or behave unexpectedly. ` +
      `Update the frontend to v${effectiveContractVersion ?? '?'} for full compatibility.`,
    severity: 'warn',
  }
}

// ----------------------------------------------------------------
//  getMigrationGuide
// ----------------------------------------------------------------

/**
 * Returns a short step-by-step migration guide string for a given
 * version transition. Used in VersionWarningBanner's collapsible details.
 *
 * @param fromVersion - The current (old) contract version.
 * @param toVersion   - The target (new) contract version.
 */
export function getMigrationGuide(fromVersion: string, toVersion: string): string {
  // Specific known transitions with tailored instructions.
  if (fromVersion === '0.1.0' && toVersion === '0.2.0') {
    return [
      '=== Migration Guide: v0.1.0 → v0.2.0 ===',
      '',
      'Step 1 — Read the changelog',
      '  Review CHANGELOG.md for breaking changes between v0.1.0 and v0.2.0.',
      '',
      'Step 2 — Deploy the new contract',
      '  Run: make deploy-testnet   (or make deploy-mainnet)',
      '  Note the new CONTRACT_ID from the deployment output.',
      '',
      'Step 3 — Update frontend environment',
      '  In your .env file set:',
      '    VITE_CONTRACT_ID=<new contract ID>',
      '    VITE_EXPECTED_CONTRACT_VERSION=0.2.0',
      '    VITE_EXPECTED_STORAGE_VERSION=2   # if storage schema changed',
      '',
      'Step 4 — Verify compatibility',
      '  Reload the frontend. The version warning banner should disappear.',
      '  Run: npm run test  to confirm no regressions.',
      '',
      'Step 5 — Communicate to users',
      '  Announce the new contract address via your community channels.',
      '  Existing deposits on v0.1.0 remain locked in the old contract.',
      '  Users must withdraw from the old contract before re-depositing.',
    ].join('\n')
  }

  // Generic fallback guide for unknown transitions.
  return [
    `=== Migration Guide: v${fromVersion} → v${toVersion} ===`,
    '',
    'Step 1 — Review the CHANGELOG',
    '  Check CHANGELOG.md for a list of breaking changes.',
    '',
    'Step 2 — Deploy the updated contract',
    '  Run: make deploy-testnet   (or make deploy-mainnet)',
    '  Note the new CONTRACT_ID from the deploy output.',
    '',
    'Step 3 — Update your frontend .env',
    `  Set VITE_CONTRACT_ID to the new contract ID.`,
    `  Set VITE_EXPECTED_CONTRACT_VERSION=${toVersion}`,
    '',
    'Step 4 — Run smoke tests',
    '  make smoke-test-local  — verifies end-to-end against the new contract.',
    '',
    'Step 5 — Inform your users',
    '  Existing deposits stay in the old contract until withdrawn.',
  ].join('\n')
}
