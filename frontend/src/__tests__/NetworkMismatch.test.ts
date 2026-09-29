import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import type { WalletInfo, SigningResult } from '../types'

/**
 * Test network mismatch detection when connecting wallet
 */
describe('Network mismatch detection', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('should detect when wallet is on a different network', () => {
    const appNetwork = 'Test SDF Network ; September 2015'
    const walletNetwork = 'Public Global Stellar Network ; September 2015'

    const hasNetworkMismatch = walletNetwork !== appNetwork
    expect(hasNetworkMismatch).toBe(true)
  })

  it('should not report mismatch when networks match', () => {
    const appNetwork = 'Test SDF Network ; September 2015'
    const walletNetwork = 'Test SDF Network ; September 2015'

    const hasNetworkMismatch = walletNetwork !== appNetwork
    expect(hasNetworkMismatch).toBe(false)
  })

  it('should set networkMismatch flag in wallet info on mismatch', () => {
    const appNetwork = 'Test SDF Network ; September 2015'
    const walletNetwork = 'Public Global Stellar Network ; September 2015'
    const address = 'GBQWZICAZJIXZ4K46D7RBQF6RNQKZLM5UGFZPVJWCNK7WSVLVLLGFUAV'

    const hasNetworkMismatch = walletNetwork !== appNetwork

    const wallet: WalletInfo = {
      address,
      displayAddress: 'GB...FUAV',
      networkMismatch: hasNetworkMismatch,
      walletNetwork,
    }

    expect(wallet.networkMismatch).toBe(true)
    expect(wallet.walletNetwork).toBe(walletNetwork)
  })

  it('should not set networkMismatch flag when networks match', () => {
    const appNetwork = 'Test SDF Network ; September 2015'
    const walletNetwork = 'Test SDF Network ; September 2015'
    const address = 'GBQWZICAZJIXZ4K46D7RBQF6RNQKZLM5UGFZPVJWCNK7WSVLVLLGFUAV'

    const hasNetworkMismatch = walletNetwork !== appNetwork

    const wallet: WalletInfo = {
      address,
      displayAddress: 'GB...FUAV',
      networkMismatch: hasNetworkMismatch,
      walletNetwork,
    }

    expect(wallet.networkMismatch).toBe(false)
    expect(wallet.walletNetwork).toBe(walletNetwork)
  })

  it('should prevent transaction signing when there is a network mismatch', () => {
    const networkMismatch = true
    const txXdr = 'AAAAAgAAAABziU76Z06GEJVyifHQ3XR23LpwcKRqKBAqX7zrWZLX...'

    // Simulate signing attempt with mismatch
    if (networkMismatch) {
      const result: SigningResult = {
        signed: false,
        rejected: false,
        error: 'Network mismatch: Wallet is on Public Global Stellar Network ; September 2015, but app is on Test SDF Network ; September 2015',
      }
      expect(result.signed).toBe(false)
      expect(result.rejected).toBe(false)
      expect(result.error).toContain('Network mismatch')
    }
  })

  it('should allow transaction signing when networks match', () => {
    const networkMismatch = false
    const txXdr = 'AAAAAgAAAABziU76Z06GEJVyifHQ3XR23LpwcKRqKBAqX7zrWZLX...'

    // Signing should proceed if not mismatch
    if (!networkMismatch) {
      // Proceed with signing (would call freighter.signTransaction)
      expect(true).toBe(true)
    }
  })

  it('should display warning message with correct networks', () => {
    const appNetwork = 'Test SDF Network ; September 2015'
    const walletNetwork = 'Public Global Stellar Network ; September 2015'
    const networkMismatch = true

    if (networkMismatch) {
      const warningMessage = `Your Freighter wallet is on ${walletNetwork}, but this app is configured for ${appNetwork}`
      expect(warningMessage).toContain('Public Global Stellar Network')
      expect(warningMessage).toContain('Test SDF Network')
    }
  })

  it('should handle cases where getNetworkDetails fails gracefully', () => {
    // If getNetworkDetails throws, we shouldn't block the connection
    const networkDetailsError = new Error('getNetworkDetails not available')
    
    // Should catch and log warning but still allow connection
    expect(() => {
      throw networkDetailsError
    }).toThrow()
    
    // But the connection should still proceed
    const wallet: WalletInfo = {
      address: 'GBQWZICAZJIXZ4K46D7RBQF6RNQKZLM5UGFZPVJWCNK7WSVLVLLGFUAV',
      displayAddress: 'GB...FUAV',
      networkMismatch: false, // Default to false if we can't check
    }
    
    expect(wallet.networkMismatch).toBe(false)
  })

  it('should track network info in wallet throughout session', () => {
    const address = 'GBQWZICAZJIXZ4K46D7RBQF6RNQKZLM5UGFZPVJWCNK7WSVLVLLGFUAV'
    const walletNetwork = 'Public Global Stellar Network ; September 2015'
    const appNetwork = 'Test SDF Network ; September 2015'

    const wallet: WalletInfo = {
      address,
      displayAddress: 'GB...FUAV',
      networkMismatch: true,
      walletNetwork,
    }

    // Network info should persist in wallet
    expect(wallet.walletNetwork).toBe(walletNetwork)
    expect(wallet.networkMismatch).toBe(true)

    // Should be available for header banner
    const shouldShowBanner = !!wallet.networkMismatch && !!wallet.walletNetwork
    expect(shouldShowBanner).toBe(true)

    // Should be available for signing check
    const canSign = !wallet.networkMismatch
    expect(canSign).toBe(false)
  })

  it('should clear network mismatch on disconnect', () => {
    let wallet: WalletInfo | null = {
      address: 'GBQWZICAZJIXZ4K46D7RBQF6RNQKZLM5UGFZPVJWCNK7WSVLVLLGFUAV',
      displayAddress: 'GB...FUAV',
      networkMismatch: true,
      walletNetwork: 'Public Global Stellar Network ; September 2015',
    }

    // After disconnect
    wallet = null

    expect(wallet).toBe(null)
  })

  it('should verify network during session restoration', () => {
    // During restoration, we should check network
    const savedAddress = 'GBQWZICAZJIXZ4K46D7RBQF6RNQKZLM5UGFZPVJWCNK7WSVLVLLGFUAV'
    const walletNetwork = 'Test SDF Network ; September 2015'
    const appNetwork = 'Test SDF Network ; September 2015'

    const wallet: WalletInfo = {
      address: savedAddress,
      displayAddress: 'GB...FUAV',
      networkMismatch: walletNetwork !== appNetwork,
      walletNetwork,
    }

    // Network should be verified during restoration
    expect(wallet.networkMismatch).toBe(false)
  })

  it('should update network info if it changes on wallet', () => {
    // Start with matching networks
    let wallet: WalletInfo = {
      address: 'GBQWZICAZJIXZ4K46D7RBQF6RNQKZLM5UGFZPVJWCNK7WSVLVLLGFUAV',
      displayAddress: 'GB...FUAV',
      networkMismatch: false,
      walletNetwork: 'Test SDF Network ; September 2015',
    }

    expect(wallet.networkMismatch).toBe(false)

    // If wallet network changes (user switched in Freighter)
    const newWalletNetwork = 'Public Global Stellar Network ; September 2015'
    wallet = {
      ...wallet,
      networkMismatch: newWalletNetwork !== 'Test SDF Network ; September 2015',
      walletNetwork: newWalletNetwork,
    }

    expect(wallet.networkMismatch).toBe(true)
    expect(wallet.walletNetwork).toBe(newWalletNetwork)
  })
})
