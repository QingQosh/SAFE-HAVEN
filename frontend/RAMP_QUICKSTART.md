# Ramp Network On-Ramp — Quick Start Guide

Get up and running with fiat purchases in 2 minutes.

## 1. Configure Environment

Add these two lines to `frontend/.env`:

```bash
VITE_RAMP_API_KEY=rampnetwork
VITE_RAMP_ENVIRONMENT=staging
```

That's it! You're using Ramp's public staging key for testing.

## 2. Start the App

```bash
cd frontend
npm install
npm run dev
```

Visit [http://localhost:5173](http://localhost:5173)

## 3. Test It

1. Click "Connect Wallet" (Freighter)
2. Click "Buy Tokens" in the header
3. Complete the Ramp flow in the modal
4. Use this test card: `4111 1111 1111 1111` (any future date, any CVC)
5. Tokens arrive in your wallet ✨

## For Production

When deploying to production:

```bash
VITE_RAMP_API_KEY=your_production_api_key_from_ramp_network
VITE_RAMP_ENVIRONMENT=production
```

Get your production key at [ramp.network/dashboard](https://ramp.network/dashboard)

## What Changed?

**New Files:**
- `frontend/src/hooks/useRampOnramp.ts` — Ramp SDK management
- `frontend/src/components/BuyTokensModal.tsx` — Modal UI

**Modified Files:**
- `frontend/src/components/Header.tsx` — "Buy Tokens" button
- `frontend/src/config.ts` — Ramp configuration
- `frontend/index.html` — Ramp SDK script
- `frontend/.env.example` — Env variable docs

**Documentation:**
- `frontend/RAMP_ONRAMP.md` — Complete setup & troubleshooting
- `IMPLEMENTATION_SUMMARY.md` — Technical details

## Troubleshooting

**"Buy Tokens" button not showing?**
- Check `VITE_RAMP_API_KEY` is set in `.env`
- Restart dev server: `npm run dev`
- Ensure wallet is connected

**Widget not loading?**
- Check browser console for errors
- Verify API key is correct
- Hard refresh: `Ctrl+Shift+R`

**Tokens not arriving?**
- Wait 5-10 minutes for blockchain confirmation
- Check Stellar Expert: [stellar.expert/explorer/testnet](https://stellar.expert/explorer/testnet)
- Verify wallet address is correct

For more help, see [RAMP_ONRAMP.md](./RAMP_ONRAMP.md)

## Feature Summary

✅ Buy XLM with fiat (USD)  
✅ No backend needed (Ramp handles KYC)  
✅ Instant wallet integration  
✅ Test with staging mode  
✅ Production ready  

Enjoy! 🚀
