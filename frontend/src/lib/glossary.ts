/**
 * Glossary of terms used in SAFE-HAVEN
 * Provides definitions and examples for all key concepts
 */

export interface GlossaryTerm {
  term: string
  definition: string
  example?: string
  relatedTerms?: string[]
  category: 'contract' | 'token' | 'time' | 'fee' | 'account' | 'operation'
}

export const GLOSSARY_TERMS: GlossaryTerm[] = [
  // Contract & Core Concepts
  {
    term: 'Vault',
    definition:
      'A secure storage location in the SAFE-HAVEN smart contract where your tokens are locked until a specified unlock time.',
    example: 'You create a vault to lock 100 XLM until January 1, 2025.',
    category: 'contract',
    relatedTerms: ['Deposit', 'Unlock Time', 'Smart Contract'],
  },
  {
    term: 'Deposit',
    definition:
      'The act of transferring tokens to the SAFE-HAVEN contract to be locked in a vault until the unlock time.',
    example: 'You deposit 50 USDC with an unlock time of 90 days from now.',
    category: 'contract',
    relatedTerms: ['Vault', 'Token', 'Unlock Time'],
  },
  {
    term: 'Withdraw',
    definition:
      'Claiming your tokens from a vault after the unlock time has passed. You can only withdraw if the unlock time is reached.',
    example: 'After 6 months, you withdraw your 100 XLM from the vault.',
    category: 'contract',
    relatedTerms: ['Deposit', 'Unlock Time', 'Vault'],
  },
  {
    term: 'Smart Contract',
    definition:
      'Automated code on the blockchain that manages deposits, enforces time locks, and handles withdrawals without intermediaries.',
    example: 'SAFE-HAVEN smart contract automatically releases your funds when the unlock time is reached.',
    category: 'contract',
    relatedTerms: ['Blockchain', 'Deposit', 'Vault'],
  },

  // Time-Related Terms
  {
    term: 'Unlock Time',
    definition:
      'The date and time when your deposited tokens will be available for withdrawal. Tokens cannot be withdrawn before this time.',
    example: 'If you deposit today with a 1-year unlock time, your funds unlock exactly one year from today.',
    category: 'time',
    relatedTerms: ['Deposit', 'Withdraw', 'Lock Duration'],
  },
  {
    term: 'Lock Duration',
    definition:
      'The length of time your tokens will be locked. Must be between 60 seconds and 5 years. SAFE-HAVEN calculates this as: Unlock Time - Current Time.',
    example: 'A 90-day lock duration means your tokens will be locked for 90 days from the deposit date.',
    category: 'time',
    relatedTerms: ['Unlock Time', 'Deposit', 'Time Remaining'],
  },
  {
    term: 'Time Remaining',
    definition:
      'The amount of time left until your vault unlocks and you can withdraw your tokens. Displayed as a countdown.',
    example: 'Your deposit shows "45 days remaining" until you can withdraw.',
    category: 'time',
    relatedTerms: ['Unlock Time', 'Lock Duration', 'Withdraw'],
  },

  // Token & Balance Terms
  {
    term: 'Token',
    definition:
      'A digital asset on the Stellar blockchain (like XLM, USDC, or other SEP-41 tokens). You can deposit any supported token into SAFE-HAVEN.',
    example: 'You deposit 100 XLM (Stellar Lumens) tokens into your vault.',
    category: 'token',
    relatedTerms: ['Deposit', 'Asset', 'Blockchain'],
  },
  {
    term: 'XLM',
    definition:
      'The native token of the Stellar blockchain, used for transactions and fees. Stands for "Stellar Lumens".',
    example: 'You deposit 50 XLM into a vault to lock it for 6 months.',
    category: 'token',
    relatedTerms: ['Token', 'Stellar', 'Stroops'],
  },
  {
    term: 'Stroops',
    definition:
      'The smallest unit of XLM. 1 XLM = 10,000,000 stroops. Used for precise balance calculations in smart contracts.',
    example: 'Your 1 XLM balance is stored as 10,000,000 stroops in the contract.',
    category: 'token',
    relatedTerms: ['XLM', 'Token', 'Balance'],
  },

  // Fee & Penalty Terms
  {
    term: 'Penalty',
    definition:
      'A fee (in percentage) deducted if you withdraw your tokens before the unlock time (early exit). The penalty goes to the fee recipient.',
    example: 'You set a 10% penalty and withdraw after 30 days. You lose 10% of your tokens; the rest are returned.',
    category: 'fee',
    relatedTerms: ['Early Exit', 'Basis Points', 'Fee Recipient'],
  },
  {
    term: 'Basis Points',
    definition:
      'A unit of measurement for percentages. 1 basis point = 0.01% = 1/100th of a percent. 10,000 basis points = 100%.',
    example: '500 basis points = 5%. If you set 500 basis points penalty, that equals 5% penalty fee.',
    category: 'fee',
    relatedTerms: ['Penalty', 'Percentage', 'Early Exit'],
  },
  {
    term: 'Early Exit',
    definition:
      'Withdrawing your tokens before the unlock time is reached. Results in a penalty fee being deducted from your withdrawal.',
    example: 'You planned to lock tokens for 1 year but need funds after 3 months. Early exit costs a 10% penalty.',
    category: 'fee',
    relatedTerms: ['Penalty', 'Withdraw', 'Cancel Deposit'],
  },
  {
    term: 'Cancel Deposit',
    definition:
      'Removing a locked deposit before the unlock time. Your tokens are returned minus the penalty fee you specified.',
    example: 'You cancel your deposit and pay a 5% penalty to get your remaining 95% of tokens back immediately.',
    category: 'fee',
    relatedTerms: ['Early Exit', 'Penalty', 'Withdraw'],
  },
  {
    term: 'Fee Recipient',
    definition:
      'The address that receives penalty fees when users cancel deposits early or exercise early exits. Set by the contract admin.',
    example: 'When you cancel a deposit with a 10% penalty, that 10% is sent to the fee recipient address.',
    category: 'fee',
    relatedTerms: ['Penalty', 'Admin', 'Early Exit'],
  },

  // Account & Admin Terms
  {
    term: 'Wallet Address',
    definition:
      'Your unique identifier on the Stellar blockchain. Looks like: GXXXXXXXXXXXXXX. Used to identify your account and receive tokens.',
    example: 'Your wallet address is GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA.',
    category: 'account',
    relatedTerms: ['Account', 'Freighter', 'Blockchain'],
  },
  {
    term: 'Admin',
    definition:
      'The administrator of SAFE-HAVEN with special powers: can pause deposits, perform emergency withdrawals, and manage contract settings.',
    example: 'The admin can pause the contract if a security issue is detected.',
    category: 'account',
    relatedTerms: ['Pause', 'Admin Panel', 'Contract'],
  },
  {
    term: 'Depositor',
    definition:
      'Any user who has created a deposit/vault in SAFE-HAVEN. The contract tracks all depositors.',
    example: 'When you create your first deposit, you become a depositor.',
    category: 'account',
    relatedTerms: ['Deposit', 'User', 'Account'],
  },

  // Operation Terms
  {
    term: 'Transaction',
    definition:
      'A record of an action on the blockchain (deposit, withdraw, etc.) that is signed by your wallet and permanently recorded.',
    example: 'Creating a new deposit is a transaction that gets recorded on the Stellar blockchain.',
    category: 'operation',
    relatedTerms: ['Blockchain', 'Sign', 'Confirm'],
  },
  {
    term: 'Pause',
    definition:
      'A feature that temporarily stops new deposits from being created. Existing deposits can still be withdrawn. Only the admin can pause.',
    example: 'The admin pauses deposits during maintenance; you can still withdraw existing vaults.',
    category: 'operation',
    relatedTerms: ['Admin', 'Emergency', 'Deposit'],
  },
  {
    term: 'Renounce Admin',
    definition:
      'Permanent removal of admin privileges by the current admin. Makes SAFE-HAVEN fully trustless with no admin control. This is irreversible.',
    example: 'The admin renounces admin rights, removing all special powers permanently.',
    category: 'operation',
    relatedTerms: ['Admin', 'Trustless', 'Contract'],
  },
  {
    term: 'Trustless',
    definition:
      'A system that requires no trust in a single entity or administrator. No one can steal or misuse your funds.',
    example: 'After admin renounces, SAFE-HAVEN becomes trustless because no one can perform emergency withdrawals.',
    category: 'operation',
    relatedTerms: ['Admin', 'Renounce Admin', 'Blockchain'],
  },

  // Form-Specific Terms
  {
    term: 'Deposit Amount',
    definition:
      'The quantity of tokens you want to lock in a vault. Must be greater than 0 and not exceed 10^15 units.',
    example: 'You deposit 100 XLM or 5000 USDC.',
    category: 'token',
    relatedTerms: ['Token', 'Deposit', 'Balance'],
  },
  {
    term: 'Unlock Date',
    definition:
      'The specific date and time when your vault will unlock and you can withdraw your tokens.',
    example: 'You set the unlock date to December 31, 2025, at 11:59 PM UTC.',
    category: 'time',
    relatedTerms: ['Unlock Time', 'Lock Duration', 'Withdraw'],
  },
  {
    term: 'Penalty Basis Points',
    definition:
      'The early-exit penalty expressed in basis points (0-10,000). Example: 500 = 5% penalty for early withdrawal.',
    example:
      'You set penalty basis points to 1000 (10%). If you exit early, 10% goes to the fee recipient, 90% returned to you.',
    category: 'fee',
    relatedTerms: ['Penalty', 'Basis Points', 'Early Exit'],
  },
]

/**
 * Get a single glossary term by name
 */
export function getGlossaryTerm(term: string): GlossaryTerm | undefined {
  return GLOSSARY_TERMS.find(
    (t) => t.term.toLowerCase() === term.toLowerCase()
  )
}

/**
 * Get all terms in a category
 */
export function getTermsByCategory(
  category: GlossaryTerm['category']
): GlossaryTerm[] {
  return GLOSSARY_TERMS.filter((t) => t.category === category)
}

/**
 * Search glossary by keyword
 */
export function searchGlossary(keyword: string): GlossaryTerm[] {
  const lowerKeyword = keyword.toLowerCase()
  return GLOSSARY_TERMS.filter(
    (t) =>
      t.term.toLowerCase().includes(lowerKeyword) ||
      t.definition.toLowerCase().includes(lowerKeyword)
  )
}
