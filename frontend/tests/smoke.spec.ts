import { test, expect } from '@playwright/test'

/**
 * Freighter wallet mock shape.
 *
 * Inject this object into `window.freighter` before the connect flow runs.
 * The mock must satisfy the subset of the Freighter API that WalletContext uses.
 */
interface MockFreighter {
  isConnected: () => Promise<{ isConnected: boolean }>
  getAddress: () => Promise<{ address: string }>
  signTransaction: (
    _xdr: string,
    _opts?: { networkPassphrase?: string }
  ) => Promise<{ signedTxXdr?: string; error?: string }>
}

test.describe('SAFE-HAVEN smoke test', () => {
  const FAKE_ADDRESS = 'GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3YYT5MOR3EGN2FHAA6OJIR2'

  test('loads app, shows connect-wallet prompt, mocks Freighter, connects, and shows Dashboard empty state', async ({
    page,
  }) => {
    // ── 1. Navigate to the app ──────────────────────────────────────────
    await page.goto('/')
    await page.waitForLoadState('domcontentloaded')

    // ── 2. Verify the title is present ───────────────────────────────────
    await expect(page.locator('header')).toContainText('SAFE-HAVEN')

    // ── 3. Verify the "Connect Wallet" button is visible ─────────────────
    const connectBtn = page.getByRole('button', { name: /Connect Wallet/i })
    await expect(connectBtn).toBeVisible()

    // ── 4. Verify the Dashboard shows the "Connect your wallet" empty state
    //    (since no wallet is connected, the Dashboard renders a prompt)
    await expect(page.getByText('Connect your wallet')).toBeVisible()

    // ── 5. Inject the mock Freighter before clicking connect ─────────────
    await page.evaluate((addr: string) => {
      const mock: MockFreighter = {
        isConnected: async () => ({ isConnected: true }),
        getAddress: async () => ({ address: addr }),
        signTransaction: async () => ({
          signedTxXdr: 'AAAAAAAAAAAAAAAAAAAAAMOCK_SIGNED_TXDR',
        }),
      }
      ;(window as unknown as Record<string, unknown>).freighter = mock
    }, FAKE_ADDRESS)

    // ── 6. Click the "Connect Wallet" button ────────────────────────────
    await connectBtn.click()

    // ── 7. Wait for the wallet to be connected ──────────────────────────
    //    After connection, the Header shows the wallet address and a "Disconnect" button
    await expect(page.getByRole('button', { name: /Disconnect/i })).toBeVisible({
      timeout: 10_000,
    })

    // ── 8. Verify the Dashboard now renders the connected state ──────────
    //    The empty wallet prompt should be gone
    await expect(page.getByText('Connect your wallet')).not.toBeVisible()

    // ── 9. Assert the Dashboard renders the "empty deposits" state ───────
    //    The truncated address is shown in the no-vaults section
    await expect(page.getByText('No active vaults for')).toBeVisible()
    //    The tip to use the Deposit tab should be visible
    await expect(page.getByText('Use the Deposit tab to lock your first tokens.')).toBeVisible()

    // ── 10. Verify the stats row is visible (4 stat cards) ───────────────
    await expect(page.getByText('Your Deposits')).toBeVisible()
    await expect(page.getByText('Unlocked')).toBeVisible()
    await expect(page.getByText('Locked')).toBeVisible()
    await expect(page.getByText('Total Depositors')).toBeVisible()
  })

  test('shows error toast when Freighter is not installed', async ({ page }) => {
    // Ensure freighter is NOT on window
    await page.goto('/')
    await page.waitForLoadState('domcontentloaded')

    // Ensure no mock freighter
    await page.evaluate(() => {
      delete (window as unknown as Record<string, unknown>).freighter
    })

    const connectBtn = page.getByRole('button', { name: /Connect Wallet/i })
    await connectBtn.click()

    // Should see an error toast about Freighter not being installed
    // react-hot-toast renders a div with role="status"
    const toast = page.getByText(/Freighter wallet not found/i)
    await expect(toast).toBeVisible({ timeout: 5_000 })
  })

  test('shows error message when wallet is locked', async ({ page }) => {
    await page.goto('/')
    await page.waitForLoadState('domcontentloaded')

    // Inject a mock Freighter that reports it is NOT connected (locked)
    await page.evaluate(() => {
      const mock: MockFreighter = {
        isConnected: async () => ({ isConnected: false }),
        getAddress: async () => ({ address: '' }),
        signTransaction: async () => ({ error: 'Wallet locked' }),
      }
      ;(window as unknown as Record<string, unknown>).freighter = mock
    })

    const connectBtn = page.getByRole('button', { name: /Connect Wallet/i })
    await connectBtn.click()

    // Should see a toast about needing to unlock the wallet
    const toast = page.getByText(/please unlock your freighter wallet/i)
    await expect(toast).toBeVisible({ timeout: 5_000 })
  })
})
