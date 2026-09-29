# Fiat On-Ramp Integration Guide (Ramp Network)

This guide explains how to set up and use the Ramp Network fiat on-ramp integration in SAFE-HAVEN.

## Overview

Ramp Network is a fiat-to-crypto on-ramp provider that allows users to purchase cryptocurrency using fiat currency (credit/debit cards, bank transfers, etc.). The SAFE-HAVEN frontend integrates Ramp's embedded widget, enabling users to buy XLM directly from the app.

### Features

- **Buy Tokens Button** — Appears in the header when a wallet is connected
- **Pre-filled Address** — User's wallet address is automatically populated
- **Pre-selected Asset** — XLM (Stellar native token) is pre-selected
- **Embedded Widget** — Ramp widget displays in a modal without leaving the app
- **Easy KYC** — Ramp handles all KYC/AML compliance
- **Instant Settlement** — Tokens arrive in the user's wallet after purchase

### Scope

**In Scope:**
- Fiat to crypto purchases (USD → XLM)
- Pre-filled wallet address and token selection
- Exchange rates and fees displayed by Ramp
- Modal-based widget integration
- Testnet and production environments

**Out of Scope:**
- Multiple on-ramp providers (one for MVP)
- Crypto-to-crypto swaps
- KYC integration (handled by Ramp)

## Setup Instructions

### 1. Get a Ramp API Key

1. Visit [ramp.network](https://ramp.network/)
2. Sign up and create an account
3. Go to the **Dashboard → API Keys** section
4. Copy your **API Key**

For **testing/staging**, use the public key: `rampnetwork`

### 2. Configure Environment Variables

Copy `.env.example` to `.env` if you haven't already:

```bash
cp frontend/.env.example frontend/.env
```

Add the following variables to `frontend/.env`:

```bash
# Ramp Network API key for the embedded widget
VITE_RAMP_API_KEY=rampnetwork

# Ramp environment: "production" or "staging"
# Use "staging" for testing without KYC delays
VITE_RAMP_ENVIRONMENT=staging
```

**Options:**

- `VITE_RAMP_API_KEY`: Your Ramp API key
  - For testing: `rampnetwork` (public staging key)
  - For production: Your actual API key from ramp.network
  
- `VITE_RAMP_ENVIRONMENT`: Choose environment
  - `staging` — Test without real KYC (fastest for dev)
  - `production` — Real money transactions with full KYC

### 3. Start the Frontend

```bash
cd frontend
npm install
npm run dev
```

The app will be available at `http://localhost:5173`

### 4. Test the Integration

1. **Connect Wallet** — Click "Connect Wallet" and select Freighter
2. **Click "Buy Tokens"** — Button appears in the header after wallet connection
3. **Complete Purchase** — Ramp widget opens in a modal
4. **Verify** — After purchase, tokens should arrive in your wallet

## Architecture

### Components

#### `useRampOnramp` Hook (`src/hooks/useRampOnramp.ts`)

Manages Ramp SDK initialization and widget lifecycle.

```typescript
const { isSDKLoaded, isSDKError, openRampWidget, closeRampWidget } = useRampOnramp()

// Open the Ramp widget with a wallet address
openRampWidget(walletAddress)

// Close the widget
closeRampWidget()
```

**State:**
- `isSDKLoaded` — Ramp SDK is ready to use
- `isSDKError` — SDK failed to load (network error, etc.)

#### `BuyTokensModal` Component (`src/components/BuyTokensModal.tsx`)

Modal UI that wraps the Ramp widget. Handles:

- Wallet connection validation
- SDK loading state
- Widget initialization with pre-filled address
- Error messaging

```typescript
<BuyTokensModal isOpen={showBuyModal} onClose={() => setShowBuyModal(false)} />
```

#### Header Integration (`src/components/Header.tsx`)

"Buy Tokens" button that opens the modal.

- Button appears only when:
  - Wallet is connected
  - Ramp is enabled (API key configured)
  - No network mismatch
- Hidden on mobile (shown via `hidden sm:flex`)

### Configuration

#### `src/config.ts`

```typescript
export const CONFIG = {
  // ... other config ...
  RAMP_API_KEY: import.meta.env.VITE_RAMP_API_KEY ?? '',
  RAMP_ENVIRONMENT: import.meta.env.VITE_RAMP_ENVIRONMENT ?? 'staging',
  RAMP_ENABLED: !!import.meta.env.VITE_RAMP_API_KEY,
}
```

#### `index.html`

Ramp SDK script is loaded asynchronously in the `<head>`:

```html
<script src="https://ri-widget-staging.firebaseapp.com/iframe.js" async defer></script>
```

## Widget Configuration

The Ramp widget is configured with these defaults in `useRampOnramp.ts`:

```typescript
const rampConfig = {
  hostAppName: 'SAFE-HAVEN',
  variant: 'embedded',
  userAddress: address,           // Pre-filled with connected wallet
  defaultAsset: 'stellar_native', // XLM only
  assetFilter: 'stellar_native',
  fiatCurrency: 'USD',
  // ... callbacks ...
}
```

### Customization Options

To modify widget behavior, edit `useRampOnramp.ts`:

```typescript
// Change default fiat currency
fiatCurrency: 'EUR', // or 'GBP', 'CAD', etc.

// Set default fiat amount
fiatValue: 100,

// Allow multiple assets
assetFilter: undefined, // Show all assets

// Add custom webhooks
webhookStatusUrl: 'https://your-api.com/webhook',
```

## Environments

### Staging (Default)

```bash
VITE_RAMP_API_KEY=rampnetwork
VITE_RAMP_ENVIRONMENT=staging
```

**Benefits:**
- No real money transactions
- Instant KYC approval (for testing)
- Lower fees
- Perfect for development

**SDK URL:** `https://ri-widget-staging.firebaseapp.com/iframe.js`

### Production

```bash
VITE_RAMP_API_KEY=your_production_key
VITE_RAMP_ENVIRONMENT=production
```

**Requirements:**
- Registered Ramp account with production credentials
- Mainnet deployment recommended
- Real KYC/AML verification required
- Standard Ramp fees apply

**SDK URL:** `https://ri-widget.firebaseapp.com/iframe.js` (use in production)

## Testing

### Local Testing

1. Set `VITE_RAMP_API_KEY=rampnetwork` and `VITE_RAMP_ENVIRONMENT=staging`
2. Run `npm run dev` in the frontend directory
3. Connect wallet and click "Buy Tokens"
4. Complete test transaction in the Ramp widget

### Test Card Numbers

Use these in Ramp staging to test without real charges:

- **Visa:** `4111 1111 1111 1111`
- **Mastercard:** `5555 5555 5555 4444`
- **Expiry:** Any future date
- **CVC:** Any 3 digits

Ramp will not charge test cards.

### Debugging

Enable browser console logging to see Ramp events:

```typescript
onSuccess: (purchase) => {
  console.log('✓ Purchase successful:', purchase)
},
onError: (error) => {
  console.error('✗ Ramp error:', error)
},
onClose: () => {
  console.log('→ Widget closed')
},
```

## Troubleshooting

### "Buy Tokens" Button Not Appearing

**Possible causes:**
- `VITE_RAMP_API_KEY` not set in `.env`
- Wallet not connected
- Network mismatch (wallet on different network than app)

**Solution:**
1. Check `.env` has `VITE_RAMP_API_KEY=rampnetwork` (or your key)
2. Ensure wallet is connected
3. Verify Freighter is on the correct network (testnet/mainnet)

### Widget Not Loading

**Possible causes:**
- SDK script failed to load (network issue)
- JavaScript error in browser console
- API key invalid or expired

**Solution:**
1. Check browser console for errors
2. Verify `VITE_RAMP_API_KEY` is correct
3. Restart dev server: `npm run dev`
4. Hard refresh browser (Ctrl+Shift+R)

### Transaction Not Appearing

**Possible causes:**
- Transaction still processing (can take 5-30 mins)
- Wrong network selected in Freighter
- Wallet address typo

**Solution:**
1. Wait 5-10 minutes for blockchain confirmation
2. Verify address in Freighter matches what you see in the modal
3. Check transaction on [Stellar Expert](https://stellar.expert/explorer/testnet)

### API Key Issues

**Key expired or rate-limited:**
1. Regenerate key at [ramp.network dashboard](https://ramp.network)
2. Update `.env` with new key
3. Restart dev server

## Production Deployment

When deploying to production:

1. **Get production API key** from Ramp dashboard
2. **Update `.env` files:**
   ```bash
   VITE_RAMP_API_KEY=your_production_key
   VITE_RAMP_ENVIRONMENT=production
   ```
3. **Update SDK URL in `index.html`** (if not using staging):
   ```html
   <script src="https://ri-widget.firebaseapp.com/iframe.js" async defer></script>
   ```
4. **Update SDK URL in `useRampOnramp.ts`** (line ~68):
   ```typescript
   script.src = 'https://ri-widget.firebaseapp.com/iframe.js'
   ```
5. **Deploy frontend** with new configuration
6. **Monitor transactions** via Ramp dashboard

## Monitoring & Analytics

Ramp provides a dashboard where you can:

- Monitor transaction volume and revenue
- View KYC status for users
- Check payment method statistics
- Review settlement history

Visit your [Ramp Dashboard](https://ramp.network/dashboard) to access analytics.

## Security Considerations

- **API Keys** — Keep production keys secure; never commit to git
- **XLM-Only** — Widget is restricted to XLM to simplify UX
- **Pre-filled Address** — Requires wallet connection; address is user-controlled
- **No Backend** — Ramp handles all payment processing and compliance
- **Environment Variables** — Use `.env` for keys; add `.env` to `.gitignore`

## Support & Resources

- **Ramp Documentation:** [https://docs.ramp.network](https://docs.ramp.network)
- **Ramp Status:** [https://status.ramp.network](https://status.ramp.network)
- **Stellar Documentation:** [https://developers.stellar.org](https://developers.stellar.org)
- **Support Issues:** Report in the SAFE-HAVEN repository

## Future Enhancements

Potential improvements for future releases:

- [ ] Multiple on-ramp providers (Coinbase, Wyre, MoonPay)
- [ ] Real-time price feeds and exchange rates
- [ ] Transaction history and receipts
- [ ] Support for additional fiat currencies and payment methods
- [ ] Analytics and usage tracking
- [ ] Webhook integration for transaction confirmation
