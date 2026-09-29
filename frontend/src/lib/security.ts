/**
 * Security checklist types and utilities
 * Tracks user's security practice recommendations
 */

export enum SecurityChecklistItem {
  // Wallet security
  SEED_PHRASE_BACKUP = 'seed_phrase_backup',
  HARDWARE_WALLET = 'hardware_wallet',
  PASSWORD_STRENGTH = 'password_strength',
  
  // Freighter security
  FREIGHTER_PASSWORD = 'freighter_password',
  FREIGHTER_BACKUP = 'freighter_backup',
  
  // Network security
  AVOID_PUBLIC_WIFI = 'avoid_public_wifi',
  HTTPS_ONLY = 'https_only',
  
  // Operational security
  VERIFY_ADDRESS = 'verify_address',
  DISCONNECT_WALLET = 'disconnect_wallet',
}

export interface SecurityChecklist {
  [key in SecurityChecklistItem]?: boolean
}

export interface SecurityTip {
  id: string
  title: string
  description: string
  category: 'wallet' | 'freighter' | 'network' | 'operational'
  priority: 'high' | 'medium' | 'low'
  actionable: boolean
}

/**
 * Comprehensive security tips database
 */
export const SECURITY_TIPS: SecurityTip[] = [
  // Wallet Security - High Priority
  {
    id: SECURITY_TIPS_ID.SEED_PHRASE,
    title: 'Backup Your Seed Phrase Securely',
    description:
      'Your seed phrase is the master key to all your crypto assets. Store it in multiple secure locations offline. Never share it or type it into a computer connected to the internet.',
    category: 'wallet',
    priority: 'high',
    actionable: true,
  },
  {
    id: SECURITY_TIPS_ID.HARDWARE_WALLET,
    title: 'Use a Hardware Wallet for Large Balances',
    description:
      'Hardware wallets (Ledger, Trezor) keep your keys offline and are the most secure way to store cryptocurrency. Recommended for holdings over $5,000.',
    category: 'wallet',
    priority: 'high',
    actionable: true,
  },
  {
    id: SECURITY_TIPS_ID.PASSWORD,
    title: 'Create a Strong, Unique Password',
    description:
      'Use a password manager to generate and store a strong password (16+ characters with mixed case, numbers, symbols). Never reuse passwords across sites.',
    category: 'wallet',
    priority: 'high',
    actionable: true,
  },

  // Freighter Security
  {
    id: SECURITY_TIPS_ID.FREIGHTER_PASSWORD,
    title: 'Secure Your Freighter Wallet Password',
    description:
      'Set a strong password for your Freighter extension. This is your first line of defense if someone gains access to your computer.',
    category: 'freighter',
    priority: 'high',
    actionable: true,
  },
  {
    id: SECURITY_TIPS_ID.FREIGHTER_BACKUP,
    title: 'Export and Back Up Your Freighter Secret Key',
    description:
      'In Freighter settings, export your account key and store it safely. This allows recovery if the extension is lost.',
    category: 'freighter',
    priority: 'medium',
    actionable: true,
  },

  // Network Security
  {
    id: SECURITY_TIPS_ID.PUBLIC_WIFI,
    title: 'Avoid Public WiFi for Transactions',
    description:
      'Public WiFi networks are vulnerable to interception. Avoid sending transactions or signing sensitive data on public networks. Use a VPN or cellular connection.',
    category: 'network',
    priority: 'high',
    actionable: false,
  },
  {
    id: SECURITY_TIPS_ID.HTTPS,
    title: 'Only Use HTTPS Connections',
    description:
      'Always verify the URL starts with "https://" before connecting your wallet. The padlock icon in your browser confirms a secure connection.',
    category: 'network',
    priority: 'high',
    actionable: false,
  },

  // Operational Security
  {
    id: SECURITY_TIPS_ID.VERIFY_ADDRESS,
    title: 'Always Verify Addresses',
    description:
      'Before sending tokens, verify the recipient address matches exactly. Paste addresses instead of typing them. Even one character difference sends funds to the wrong person permanently.',
    category: 'operational',
    priority: 'high',
    actionable: true,
  },
  {
    id: SECURITY_TIPS_ID.DISCONNECT,
    title: 'Disconnect Your Wallet When Done',
    description:
      'Disconnect from SAFE-HAVEN when you\'re finished. This minimizes the window of exposure if the page becomes compromised.',
    category: 'operational',
    priority: 'medium',
    actionable: true,
  },
  {
    id: SECURITY_TIPS_ID.CHECK_CONTRACT,
    title: 'Verify Contract Address',
    description:
      'Verify the contract address in SAFE-HAVEN matches the official one. Bookmark the official link and use it. Phishing pages may have similar URLs.',
    category: 'operational',
    priority: 'high',
    actionable: true,
  },
  {
    id: SECURITY_TIPS_ID.TEST_FIRST,
    title: 'Test with Small Amounts First',
    description:
      'Always test transactions with small amounts before depositing large sums. This verifies everything works correctly without risking significant funds.',
    category: 'operational',
    priority: 'medium',
    actionable: true,
  },

  // Additional Tips
  {
    id: SECURITY_TIPS_ID.TWO_FACTOR,
    title: 'Enable Two-Factor Authentication',
    description:
      'If you use an exchange for KYC, enable 2FA. Even though SAFE-HAVEN doesn\'t have 2FA, this protects your exchange account.',
    category: 'wallet',
    priority: 'medium',
    actionable: true,
  },
  {
    id: SECURITY_TIPS_ID.PHISHING,
    title: 'Be Aware of Phishing Attacks',
    description:
      'Attackers create fake websites and emails to steal credentials. Never click links from unsolicited emails. Always navigate directly to the official site.',
    category: 'operational',
    priority: 'medium',
    actionable: false,
  },
]

// Security tips ID mapping for easier reference
export const SECURITY_TIPS_ID = {
  SEED_PHRASE: 'seed_phrase',
  HARDWARE_WALLET: 'hardware_wallet',
  PASSWORD: 'password_strength',
  FREIGHTER_PASSWORD: 'freighter_password',
  FREIGHTER_BACKUP: 'freighter_backup',
  PUBLIC_WIFI: 'public_wifi',
  HTTPS: 'https_only',
  VERIFY_ADDRESS: 'verify_address',
  DISCONNECT: 'disconnect_wallet',
  CHECK_CONTRACT: 'check_contract',
  TEST_FIRST: 'test_first',
  TWO_FACTOR: 'two_factor',
  PHISHING: 'phishing',
}

/**
 * Get tips by category
 */
export function getTipsByCategory(
  category: SecurityTip['category']
): SecurityTip[] {
  return SECURITY_TIPS.filter((tip) => tip.category === category)
}

/**
 * Get tips by priority
 */
export function getTipsByPriority(
  priority: SecurityTip['priority']
): SecurityTip[] {
  return SECURITY_TIPS.filter((tip) => tip.priority === priority)
}

/**
 * Get high priority tips (security must-haves)
 */
export function getHighPriorityTips(): SecurityTip[] {
  return getTipsByPriority('high')
}

/**
 * Get tips that are immediately actionable
 */
export function getActionableTips(): SecurityTip[] {
  return SECURITY_TIPS.filter((tip) => tip.actionable)
}

/**
 * Get a random security tip
 */
export function getRandomSecurityTip(): SecurityTip {
  return SECURITY_TIPS[Math.floor(Math.random() * SECURITY_TIPS.length)]
}

/**
 * Format security tips for onboarding
 */
export function getOnboardingSecurityTips(): SecurityTip[] {
  // Top 3 high-priority items for new users
  return getHighPriorityTips().slice(0, 5)
}
