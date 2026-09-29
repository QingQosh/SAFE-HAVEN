# Price Oracle Quick Reference

## What Was Built

A real-time price oracle that fetches token prices from CoinGecko and displays USD equivalents throughout the UI.

## Key Components

| File | Purpose |
|------|---------|
| `priceService.ts` | CoinGecko API client + caching |
| `PriceContext.tsx` | React context for global state |
| `usePrice.ts` | Hook to access prices in components |
| `format.ts` | USD conversion utilities |

## How It Works

```
App Start
  ↓
PriceProvider initializes
  ↓
fetchTokenPrices() called → CoinGecko API
  ↓
Prices cached in React state
  ↓
Components use usePrice() hook
  ↓
Display prices next to amounts
  ↓
Auto-refresh every 60 seconds
```

## Display Examples

### DepositCard
```
100 XLM ($15.00)
updated 2m ago
```

### WithdrawPage Amount
```
100 XLM ($15.00)
```

### DepositPage Summary
```
Locking    100 XLM ($15.00)
updated 1m ago
Until      Dec 28, 2026 04:00 AM
```

## For Developers

### Adding USD to a New Component

1. Import the hook and formatters:
```tsx
import { usePrice } from '../hooks/usePrice'
import { formatTokenWithUsd, formatPriceUpdate } from '../lib/format'
```

2. Use in component:
```tsx
const { getPrice } = usePrice()
const priceData = getPrice('native')
const priceUsd = priceData?.usd

const display = formatTokenWithUsd(amount, 'XLM', priceUsd)
// "100 XLM ($15.00)" or "100 XLM" if price unavailable
```

### Adding a New Token

1. Edit `src/lib/priceService.ts`:
```typescript
const TOKEN_TO_COINGECKO_ID: Record<string, string> = {
  native: 'stellar',
  usdc_contract_id: 'usd-coin',  // Add this
}
```

2. Update `App.tsx`:
```tsx
<PriceProvider tokenIds={['native', 'usdc_contract_id']}>
```

3. In components:
```tsx
const priceData = getPrice('usdc_contract_id')
const display = formatTokenWithUsd(amount, 'USDC', priceData?.usd)
```

## API Details

- **Source**: CoinGecko free API
- **Endpoint**: `https://api.coingecko.com/api/v3/simple/price`
- **Rate Limit**: ~50 calls/min (free tier)
- **Timeout**: 5 seconds per request
- **Update Interval**: 60 seconds

## Error Handling

| Scenario | Behavior |
|----------|----------|
| API down | Uses last known price |
| Invalid token | Logs warning, shows amount without USD |
| Network timeout | Retries on next refresh cycle |
| Rate limit | Shows error in console, maintains price |

## Configuration

All settings are in `PriceContext.tsx`:

```typescript
// Refresh interval (default: 60000 ms)
refreshIntervalMs={60_000}

// Tokens to fetch (default: ['native'])
tokenIds={['native']}
```

## Testing Checklist

- [ ] Prices display on Dashboard deposits
- [ ] Prices display on DepositPage form
- [ ] Prices display on WithdrawPage details
- [ ] Price updates every 60 seconds (check DevTools)
- [ ] Penalty amounts show USD equivalents
- [ ] Graceful fallback when no price available
- [ ] Build completes without errors
- [ ] Dev server runs without errors

## Common Issues

**Q: Prices showing as NaN**  
A: Token is not in TOKEN_TO_COINGECKO_ID mapping

**Q: Prices won't update**  
A: Check browser console for API errors; verify CoinGecko is accessible

**Q: Build fails**  
A: Ensure vite-env.d.ts exists in src/

**Q: Performance issues**  
A: Price service makes only 1 API call per 60s; shouldn't impact performance

## Files to Review

1. **priceService.ts** — API integration logic
2. **PriceContext.tsx** — State management & refresh loop
3. **format.ts** — USD formatting functions
4. **DepositCard.tsx** — Example component integration
5. **App.tsx** — Provider setup

## Rollback Plan

If prices need to be disabled:

1. Remove PriceProvider from App.tsx:
```tsx
// Before
<PriceProvider tokenIds={['native']}>
  <WalletProvider>
    <AppInner />
  </WalletProvider>
</PriceProvider>

// After
<WalletProvider>
  <AppInner />
</WalletProvider>
```

2. Remove usePrice() calls from components
3. Remove formatTokenWithUsd() and use stroopsToXlm() instead
4. Revert package.json if needed (no new dependencies were added)

## Support

- **Questions about implementation?** See PRICE_ORACLE.md
- **CoinGecko API docs?** https://docs.coingecko.com/reference
- **Need higher rate limits?** Upgrade at https://www.coingecko.com/api
