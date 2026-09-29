// ============================================================
//  Component tests for Header.tsx — balance sync UI
// ============================================================

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { WalletInfo, SigningResult } from '../types'

// ---------------------------------------------------------------------------
//  Mock heavy dependencies that Header pulls in transitively
// ---------------------------------------------------------------------------

// Mock the entire WalletContext module so we can inject any state we want
vi.mock('../context/WalletContext', () => ({
  useWallet: vi.fn(),
}))

// Mock NetworkContext (used by Header → NetworkSwitcher)
vi.mock('../context/NetworkContext', () => ({
  useNetwork: vi.fn(() => ({
    currentNetwork: 'testnet',
    envNetwork: 'testnet',
    isMismatched: false,
    switchNetwork: vi.fn(),
    resetToEnvNetwork: vi.fn(),
  })),
}))

// Stub child components that make fetch calls or require full context trees
vi.mock('../components/NetworkSwitcher', () => ({
  NetworkSwitcher: () => <div data-testid="network-switcher" />,
}))
vi.mock('../components/SmallBalanceWarning', () => ({
  SmallBalanceWarning: () => null,
}))
vi.mock('../components/PublicWifiWarning', () => ({
  PublicWifiWarning: () => null,
}))
vi.mock('../components/BuyTokensModal', () => ({
  BuyTokensModal: () => null,
}))
vi.mock('../components/HelpModal', () => ({
  HelpModal: () => null,
}))
vi.mock('../components/SecurityTipsModal', () => ({
  SecurityTipsModal: () => null,
}))

// ---------------------------------------------------------------------------
//  Helpers
// ---------------------------------------------------------------------------

import { useWallet } from '../context/WalletContext'
import { Header } from '../components/Header'

const mockUseWallet = vi.mocked(useWallet)

const STUB_WALLET: WalletInfo = {
  address: 'GABC1234567890123456789012345678901234567890123456789012',
  displayAddress: 'GABC…9012',
}

const defaultWalletContext = {
  wallet:               null as WalletInfo | null,
  wallets:              [],
  isConnecting:         false,
  isRestoringSession:   false,
  networkMismatch:      false,
  connect:              vi.fn<[], Promise<void>>(),
  disconnect:           vi.fn<[], void>(),
  signTransaction:      vi.fn<[string], Promise<SigningResult>>(),
  // balance sync defaults
  balance:              null as string | null,
  balanceError:         null as string | null,
  isBalanceStale:       false,
  lastBalanceUpdate:    null as Date | null,
  refreshBalance:       vi.fn<[], Promise<void>>(),
  isRefreshingBalance:  false,
}

function setupWallet(overrides: Partial<typeof defaultWalletContext> = {}) {
  mockUseWallet.mockReturnValue({ ...defaultWalletContext, ...overrides })
}

// ---------------------------------------------------------------------------
//  12.1 Refresh button rendered when wallet connected, absent when not
//  Requirements: 3.1, 3.5
// ---------------------------------------------------------------------------

describe('12.1 refresh button visibility', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders the refresh button when a wallet is connected', () => {
    setupWallet({ wallet: STUB_WALLET })
    render(<Header isPaused={false} />)

    const btn = screen.getByRole('button', { name: /refresh balance/i })
    expect(btn).toBeTruthy()
  })

  it('does NOT render the refresh button when no wallet is connected', () => {
    setupWallet({ wallet: null })
    render(<Header isPaused={false} />)

    const btn = screen.queryByRole('button', { name: /refresh balance/i })
    expect(btn).toBeNull()
  })
})

// ---------------------------------------------------------------------------
//  12.2 Refresh button disabled and spinning while isRefreshingBalance=true
//  Requirements: 3.3
// ---------------------------------------------------------------------------

describe('12.2 refresh button loading state', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('is disabled and its icon has animate-spin while isRefreshingBalance is true', () => {
    setupWallet({ wallet: STUB_WALLET, isRefreshingBalance: true })
    render(<Header isPaused={false} />)

    const btn = screen.getByRole('button', { name: /refresh balance/i })
    expect(btn.hasAttribute('disabled')).toBe(true)

    // The SVG inside the button should carry the animate-spin class
    const svg = btn.querySelector('svg')
    expect(svg?.classList.contains('animate-spin')).toBe(true)
  })

  it('is enabled and the icon has no animate-spin when isRefreshingBalance is false', () => {
    setupWallet({ wallet: STUB_WALLET, isRefreshingBalance: false })
    render(<Header isPaused={false} />)

    const btn = screen.getByRole('button', { name: /refresh balance/i })
    expect(btn.hasAttribute('disabled')).toBe(false)

    const svg = btn.querySelector('svg')
    expect(svg?.classList.contains('animate-spin')).toBe(false)
  })
})

// ---------------------------------------------------------------------------
//  12.3 "Updating…" placeholder when no balance yet
//  Requirements: 4.4
// ---------------------------------------------------------------------------

describe('12.3 "Updating…" placeholder before first fetch', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('shows "Updating…" when balance, lastBalanceUpdate, and balanceError are all null', () => {
    setupWallet({
      wallet:            STUB_WALLET,
      balance:           null,
      lastBalanceUpdate: null,
      balanceError:      null,
    })
    render(<Header isPaused={false} />)

    expect(screen.getByText('Updating…')).toBeTruthy()
  })
})

// ---------------------------------------------------------------------------
//  12.4 "—" placeholder when balance is null and error is set
//  Requirements: 5.6
// ---------------------------------------------------------------------------

describe('12.4 dash placeholder on error with no prior balance', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders "—" when balance is null and balanceError is set', () => {
    setupWallet({
      wallet:       STUB_WALLET,
      balance:      null,
      balanceError: 'fetch failed',
    })
    render(<Header isPaused={false} />)

    // The dash character as rendered
    expect(screen.getByText('—')).toBeTruthy()
  })
})

// ---------------------------------------------------------------------------
//  12.5 Amber styling applied when isBalanceStale=true
//  Requirements: 4.5
// ---------------------------------------------------------------------------

describe('12.5 amber styling when balance is stale', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('timestamp element has text-amber-500 class when isBalanceStale is true', () => {
    const lastBalanceUpdate = new Date()
    setupWallet({
      wallet:            STUB_WALLET,
      balance:           '42.0000000',
      isBalanceStale:    true,
      lastBalanceUpdate,
    })
    render(<Header isPaused={false} />)

    // The timestamp line says "Updated X ago" — find the element that contains it
    // and assert amber styling
    const timestampEl = screen.getByText(/updated .* ago/i)
    expect(timestampEl.className).toContain('text-amber-500')
  })

  it('timestamp element does NOT have text-amber-500 when balance is fresh', () => {
    const lastBalanceUpdate = new Date()
    setupWallet({
      wallet:            STUB_WALLET,
      balance:           '42.0000000',
      isBalanceStale:    false,
      balanceError:      null,
      lastBalanceUpdate,
    })
    render(<Header isPaused={false} />)

    const timestampEl = screen.getByText(/updated .* ago/i)
    expect(timestampEl.className).not.toContain('text-amber-500')
  })
})

// ---------------------------------------------------------------------------
//  12.6 Inline error hint visible when balanceError is set
//  Requirements: 5.3
// ---------------------------------------------------------------------------

describe('12.6 inline error hint', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('shows "Balance sync error" text when balanceError is set', () => {
    setupWallet({
      wallet:       STUB_WALLET,
      balance:      '10.0000000', // has a prior balance (stale scenario)
      balanceError: 'Network error',
      isBalanceStale: true,
    })
    render(<Header isPaused={false} />)

    expect(screen.getByText('Balance sync error')).toBeTruthy()
  })

  it('does NOT show "Balance sync error" when balanceError is null', () => {
    setupWallet({
      wallet:       STUB_WALLET,
      balance:      '10.0000000',
      balanceError: null,
    })
    render(<Header isPaused={false} />)

    expect(screen.queryByText('Balance sync error')).toBeNull()
  })
})
